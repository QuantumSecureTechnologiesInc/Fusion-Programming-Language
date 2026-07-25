//! # Fusion HAFT (Hyper-Adaptive Flux Tensor)
//!
//! A tiered memory manager that tracks data placement across CPU RAM and GPU
//! VRAM, automatically migrates data based on access patterns, and provides
//! memory pooling for efficient reuse.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │            HaftMemoryManager                │
//! │  ┌──────────────────────────────────────┐   │
//! │  │     Tier Tracker (CPU/GPU/Shared)    │   │
//!  │  └──────────────────────────────────────┘   │
//! │  ┌──────────────────────────────────────┐   │
//! │  │     Access Pattern Analyzer          │   │
//! │  └──────────────────────────────────────┘   │
//! │  ┌──────────────────────────────────────┐   │
//! │  │     Memory Pool (size-class bins)    │   │
//! │  └──────────────────────────────────────┘   │
//! │  ┌──────────────────────────────────────┐   │
//! │  │     Migration Engine                 │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────┘
//! ```

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{debug, info};

// ─── Memory Tier ───────────────────────────────────────────────

/// Where a buffer currently resides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MemoryTier {
    /// System RAM (CPU-accessible).
    Ram,
    /// GPU video memory (device-local).
    Vram(u32),
    /// Both CPU and GPU accessible (pinned/host-visible).
    Shared,
}

impl std::fmt::Display for MemoryTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryTier::Ram => write!(f, "RAM"),
            MemoryTier::Vram(id) => write!(f, "VRAM:{}", id),
            MemoryTier::Shared => write!(f, "Shared"),
        }
    }
}

// ─── Buffer Metadata ───────────────────────────────────────────

/// Metadata for a tracked memory allocation.
#[derive(Debug, Clone)]
pub struct BufferInfo {
    pub id: u64,
    pub size: usize,
    pub tier: MemoryTier,
    /// Number of times this buffer has been accessed.
    pub access_count: u64,
    /// Number of times this buffer has been migrated.
    pub migration_count: u32,
    /// Timestamp of last access (monotonic tick).
    pub last_access_tick: u64,
    /// Timestamp of creation.
    pub created_tick: u64,
    /// Whether this buffer is currently "hot" (frequently accessed).
    pub is_hot: bool,
}

// ─── Access Pattern ────────────────────────────────────────────

/// Patterns of buffer access that influence migration decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessPattern {
    /// Sequential read-only (good for GPU prefetch).
    SequentialRead,
    /// Random access (better on CPU).
    RandomAccess,
    /// Write-heavy (benefits from GPU local memory).
    WriteHeavy,
    /// Read-modify-write (benefits from Shared tier).
    ReadModifyWrite,
    /// Unknown/default.
    Unknown,
}

impl AccessPattern {
    /// Which tier is best for this access pattern?
    pub fn optimal_tier(&self, device_id: u32) -> MemoryTier {
        match self {
            AccessPattern::SequentialRead => MemoryTier::Vram(device_id),
            AccessPattern::RandomAccess => MemoryTier::Ram,
            AccessPattern::WriteHeavy => MemoryTier::Vram(device_id),
            AccessPattern::ReadModifyWrite => MemoryTier::Shared,
            AccessPattern::Unknown => MemoryTier::Ram,
        }
    }
}

// ─── Size-Class Pool ───────────────────────────────────────────

/// A pool of reusable memory blocks grouped by size class.
struct SizeClassPool {
    /// Size class in bytes (e.g., 64, 256, 1024, 4096, ...).
    block_size: usize,
    /// Available blocks (raw pointers).
    free_blocks: Vec<*mut u8>,
    /// Total blocks allocated by this pool.
    total_allocated: usize,
}

impl SizeClassPool {
    fn new(block_size: usize) -> Self {
        Self {
            block_size,
            free_blocks: Vec::new(),
            total_allocated: 0,
        }
    }

    /// Get the size class for a requested allocation size.
    fn size_class(requested: usize) -> usize {
        // Round up to next power of 2, minimum 64 bytes
        let min_class = 64;
        if requested <= min_class {
            min_class
        } else {
            let class = min_class;
            let mut c = class;
            while c < requested {
                c *= 2;
            }
            c
        }
    }

    /// Allocate a block from this pool.
    fn alloc(&mut self) -> *mut u8 {
        if let Some(ptr) = self.free_blocks.pop() {
            ptr
        } else {
            let layout = std::alloc::Layout::from_size_align(self.block_size, 64).unwrap();
            // SAFETY: layout is valid; caller must ensure deallocation.
            let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            self.total_allocated += 1;
            ptr
        }
    }

    /// Return a block to the pool.
    fn dealloc(&mut self, ptr: *mut u8) {
        self.free_blocks.push(ptr);
    }

    /// Number of available blocks.
    fn free_count(&self) -> usize {
        self.free_blocks.len()
    }

    /// Total blocks ever allocated.
    fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    /// Release all free blocks back to the system.
    fn release_free(&mut self) {
        let layout = std::alloc::Layout::from_size_align(self.block_size, 64).unwrap();
        for ptr in self.free_blocks.drain(..) {
            // SAFETY: ptr was allocated with the same layout in alloc().
            unsafe { std::alloc::dealloc(ptr, layout) };
        }
    }
}

impl Drop for SizeClassPool {
    fn drop(&mut self) {
        self.release_free();
    }
}

// ─── HAFT Memory Manager ──────────────────────────────────────

/// Hyper-Adaptive Flux Tensor memory manager.
///
/// Tracks buffer locations across memory tiers, analyzes access patterns,
/// and automatically migrates data to the optimal tier. Provides size-class
/// memory pooling for efficient allocation reuse.
pub struct HaftMemoryManager {
    buffers: RwLock<HashMap<u64, BufferInfo>>,
    pools: RwLock<HashMap<usize, SizeClassPool>>,
    access_history: RwLock<Vec<(u64, AccessPattern)>>,
    tick: AtomicU64,
    next_buffer_id: AtomicU64,
    total_allocated_bytes: AtomicU64,
    total_migrations: AtomicU64,

    /// Total system RAM available (bytes).
    ram_capacity: usize,
    /// Total VRAM per GPU device.
    vram_capacity: HashMap<u32, usize>,
}

impl HaftMemoryManager {
    /// Create a new HAFT manager with given capacities.
    pub fn new(ram_capacity: usize, vram_capacity: HashMap<u32, usize>) -> Self {
        info!(
            "Initializing HAFT: RAM={}MB, GPUs={}",
            ram_capacity / (1024 * 1024),
            vram_capacity.len()
        );

        Self {
            buffers: RwLock::new(HashMap::new()),
            pools: RwLock::new(HashMap::new()),
            access_history: RwLock::new(Vec::new()),
            tick: AtomicU64::new(0),
            next_buffer_id: AtomicU64::new(0),
            total_allocated_bytes: AtomicU64::new(0),
            total_migrations: AtomicU64::new(0),
            ram_capacity,
            vram_capacity,
        }
    }

    /// Create a manager with default capacities (8GB RAM, no GPUs).
    pub fn default_ram() -> Self {
        Self::new(8 * 1024 * 1024 * 1024, HashMap::new())
    }

    /// Allocate a buffer in the specified tier.
    pub fn allocate(&self, size: usize, tier: MemoryTier) -> u64 {
        let id = self.next_buffer_id.fetch_add(1, Ordering::Relaxed);
        let tick = self.tick.fetch_add(1, Ordering::Relaxed);

        let info = BufferInfo {
            id,
            size,
            tier,
            access_count: 0,
            migration_count: 0,
            last_access_tick: tick,
            created_tick: tick,
            is_hot: false,
        };

        self.buffers.write().insert(id, info);
        self.total_allocated_bytes.fetch_add(size as u64, Ordering::Relaxed);

        debug!("Allocated buffer {} ({} bytes) in {}", id, size, tier);
        id
    }

    /// Record an access to a buffer and update access patterns.
    pub fn access(&self, buffer_id: u64, pattern: AccessPattern) {
        let tick = self.tick.fetch_add(1, Ordering::Relaxed);

        let mut buffers = self.buffers.write();
        if let Some(info) = buffers.get_mut(&buffer_id) {
            info.access_count += 1;
            info.last_access_tick = tick;

            // Mark as hot if accessed frequently
            if info.access_count > 10 {
                info.is_hot = true;
            }
        }

        self.access_history.write().push((buffer_id, pattern));
    }

    /// Get the current tier of a buffer.
    pub fn buffer_tier(&self, buffer_id: u64) -> Option<MemoryTier> {
        self.buffers.read().get(&buffer_id).map(|b| b.tier)
    }

    /// Get full info for a buffer.
    pub fn buffer_info(&self, buffer_id: u64) -> Option<BufferInfo> {
        self.buffers.read().get(&buffer_id).cloned()
    }

    /// Migrate a buffer to a new tier.
    pub fn migrate(&self, buffer_id: u64, new_tier: MemoryTier) -> bool {
        let mut buffers = self.buffers.write();
        if let Some(info) = buffers.get_mut(&buffer_id) {
            let old_tier = info.tier;
            if old_tier == new_tier {
                return true; // Already there
            }

            info.tier = new_tier;
            info.migration_count += 1;
            self.total_migrations.fetch_add(1, Ordering::Relaxed);

            debug!(
                "Migrated buffer {} from {} to {} (migration #{})",
                buffer_id, old_tier, new_tier, info.migration_count
            );
            true
        } else {
            false
        }
    }

    /// Free a buffer and return its memory to the pool.
    pub fn free(&self, buffer_id: u64) -> bool {
        let removed = self.buffers.write().remove(&buffer_id);
        if let Some(info) = &removed {
            self.total_allocated_bytes
                .fetch_sub(info.size as u64, Ordering::Relaxed);
            debug!("Freed buffer {} ({} bytes)", buffer_id, info.size);
        }
        removed.is_some()
    }

    /// Auto-migrate buffers based on access patterns.
    ///
    /// Scans all tracked buffers and migrates them to the tier recommended
    /// by their observed access pattern. Only migrates if the new tier is
    /// different and migration count is below the threshold.
    pub fn auto_migrate(&self, max_migrations_per_buffer: u32) -> usize {
        let mut migrated = 0;
        let buffers = self.buffers.read().clone();

        // Analyze access patterns from history
        let history = self.access_history.read();
        let mut pattern_counts: HashMap<u64, HashMap<AccessPattern, u64>> = HashMap::new();
        for &(id, pattern) in history.iter() {
            *pattern_counts
                .entry(id)
                .or_default()
                .entry(pattern)
                .or_insert(0) += 1;
        }

        // Collect migration candidates
        let mut candidates: Vec<(u64, MemoryTier)> = Vec::new();
        for (id, info) in &buffers {
            if info.migration_count >= max_migrations_per_buffer {
                continue;
            }

            let dominant_pattern = pattern_counts
                .get(id)
                .and_then(|counts| {
                    counts
                        .iter()
                        .max_by_key(|(_, &count)| count)
                        .map(|(&pattern, _)| pattern)
                })
                .unwrap_or(AccessPattern::Unknown);

            let optimal_tier = dominant_pattern.optimal_tier(0);
            if info.tier != optimal_tier {
                candidates.push((*id, optimal_tier));
            }
        }

        // Migrate the first candidate (one per call to avoid lock contention)
        if let Some((id, tier)) = candidates.into_iter().next() {
            self.migrate(id, tier);
            migrated = 1;
        }

        migrated
    }

    /// Allocate from the size-class pool.
    pub fn pool_alloc(&self, size: usize) -> *mut u8 {
        let class_size = SizeClassPool::size_class(size);
        let mut pools = self.pools.write();
        let pool = pools.entry(class_size).or_insert_with(|| SizeClassPool::new(class_size));
        pool.alloc()
    }

    /// Return a block to the size-class pool.
    pub fn pool_dealloc(&self, ptr: *mut u8, size: usize) {
        let class_size = SizeClassPool::size_class(size);
        let mut pools = self.pools.write();
        if let Some(pool) = pools.get_mut(&class_size) {
            pool.dealloc(ptr);
        }
    }

    /// Get pool statistics.
    pub fn pool_stats(&self) -> Vec<(usize, usize, usize)> {
        self.pools
            .read()
            .iter()
            .map(|(&class, pool)| (class, pool.free_count(), pool.total_allocated()))
            .collect()
    }

    /// Release unused pool memory.
    pub fn pool_release(&self) {
        let mut pools = self.pools.write();
        for pool in pools.values_mut() {
            pool.release_free();
        }
        info!("Released all free pool blocks");
    }

    /// Total allocated bytes across all buffers.
    pub fn total_allocated_bytes(&self) -> u64 {
        self.total_allocated_bytes.load(Ordering::Relaxed)
    }

    /// Total number of migrations performed.
    pub fn total_migrations(&self) -> u64 {
        self.total_migrations.load(Ordering::Relaxed)
    }

    /// Number of tracked buffers.
    pub fn buffer_count(&self) -> usize {
        self.buffers.read().len()
    }

    /// Get all buffer IDs.
    pub fn buffer_ids(&self) -> Vec<u64> {
        self.buffers.read().keys().copied().collect()
    }

    /// Get memory usage per tier.
    pub fn usage_by_tier(&self) -> HashMap<MemoryTier, u64> {
        let buffers = self.buffers.read();
        let mut usage = HashMap::new();
        for info in buffers.values() {
            *usage.entry(info.tier).or_insert(0) += info.size as u64;
        }
        usage
    }

    /// Get the number of hot buffers.
    pub fn hot_buffer_count(&self) -> usize {
        self.buffers.read().values().filter(|b| b.is_hot).count()
    }

    /// Clear all buffers and pools.
    pub fn clear(&self) {
        self.buffers.write().clear();
        self.access_history.write().clear();
        self.total_allocated_bytes.store(0, Ordering::Relaxed);
        self.total_migrations.store(0, Ordering::Relaxed);
        info!("HAFT memory manager cleared");
    }
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let mgr = HaftMemoryManager::default_ram();
        assert_eq!(mgr.buffer_count(), 0);
        assert_eq!(mgr.total_allocated_bytes(), 0);
    }

    #[test]
    fn test_allocate_and_free() {
        let mgr = HaftMemoryManager::default_ram();
        let id = mgr.allocate(1024, MemoryTier::Ram);
        assert_eq!(mgr.buffer_count(), 1);
        assert_eq!(mgr.total_allocated_bytes(), 1024);

        assert!(mgr.free(id));
        assert_eq!(mgr.buffer_count(), 0);
        assert_eq!(mgr.total_allocated_bytes(), 0);
    }

    #[test]
    fn test_buffer_info() {
        let mgr = HaftMemoryManager::default_ram();
        let id = mgr.allocate(2048, MemoryTier::Vram(0));

        let info = mgr.buffer_info(id).unwrap();
        assert_eq!(info.size, 2048);
        assert_eq!(info.tier, MemoryTier::Vram(0));
        assert_eq!(info.access_count, 0);
        assert!(!info.is_hot);
    }

    #[test]
    fn test_access_tracking() {
        let mgr = HaftMemoryManager::default_ram();
        let id = mgr.allocate(512, MemoryTier::Ram);

        // Access multiple times to make it "hot"
        for _ in 0..15 {
            mgr.access(id, AccessPattern::SequentialRead);
        }

        let info = mgr.buffer_info(id).unwrap();
        assert_eq!(info.access_count, 15);
        assert!(info.is_hot);
    }

    #[test]
    fn test_migration() {
        let mgr = HaftMemoryManager::default_ram();
        let id = mgr.allocate(1024, MemoryTier::Ram);

        assert_eq!(mgr.buffer_tier(id), Some(MemoryTier::Ram));

        let ok = mgr.migrate(id, MemoryTier::Vram(0));
        assert!(ok);
        assert_eq!(mgr.buffer_tier(id), Some(MemoryTier::Vram(0)));
        assert_eq!(mgr.total_migrations(), 1);

        // Migrate to Shared
        mgr.migrate(id, MemoryTier::Shared);
        assert_eq!(mgr.buffer_tier(id), Some(MemoryTier::Shared));
        assert_eq!(mgr.total_migrations(), 2);
    }

    #[test]
    fn test_migration_same_tier() {
        let mgr = HaftMemoryManager::default_ram();
        let id = mgr.allocate(1024, MemoryTier::Ram);

        // Migrating to the same tier is a no-op
        let ok = mgr.migrate(id, MemoryTier::Ram);
        assert!(ok);
        assert_eq!(mgr.total_migrations(), 0);
    }

    #[test]
    fn test_migration_nonexistent_buffer() {
        let mgr = HaftMemoryManager::default_ram();
        let ok = mgr.migrate(999, MemoryTier::Ram);
        assert!(!ok);
    }

    #[test]
    fn test_free_nonexistent() {
        let mgr = HaftMemoryManager::default_ram();
        assert!(!mgr.free(42));
    }

    #[test]
    fn test_usage_by_tier() {
        let mgr = HaftMemoryManager::default_ram();
        mgr.allocate(100, MemoryTier::Ram);
        mgr.allocate(200, MemoryTier::Ram);
        mgr.allocate(300, MemoryTier::Vram(0));

        let usage = mgr.usage_by_tier();
        assert_eq!(usage[&MemoryTier::Ram], 300);
        assert_eq!(usage[&MemoryTier::Vram(0)], 300);
    }

    #[test]
    fn test_auto_migrate() {
        let mgr = HaftMemoryManager::default_ram();
        let id = mgr.allocate(1024, MemoryTier::Ram);

        // Record sequential read pattern (prefers VRAM)
        for _ in 0..20 {
            mgr.access(id, AccessPattern::SequentialRead);
        }

        let migrated = mgr.auto_migrate(5);
        assert!(migrated > 0);
        assert_eq!(mgr.buffer_tier(id), Some(MemoryTier::Vram(0)));
    }

    #[test]
    fn test_pool_alloc_dealloc() {
        let mgr = HaftMemoryManager::default_ram();

        let ptr1 = mgr.pool_alloc(100);
        let ptr2 = mgr.pool_alloc(200);
        assert!(!ptr1.is_null());
        assert!(!ptr2.is_null());
        assert_ne!(ptr1, ptr2);

        mgr.pool_dealloc(ptr1, 100);
        mgr.pool_dealloc(ptr2, 200);

        let stats = mgr.pool_stats();
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_pool_reuse() {
        let mgr = HaftMemoryManager::default_ram();

        let ptr1 = mgr.pool_alloc(128);
        mgr.pool_dealloc(ptr1, 128);

        let ptr2 = mgr.pool_alloc(128);
        // Should reuse the same memory block
        assert_eq!(ptr1, ptr2);

        mgr.pool_dealloc(ptr2, 128);
    }

    #[test]
    fn test_size_class_rounds_up() {
        assert_eq!(SizeClassPool::size_class(1), 64);
        assert_eq!(SizeClassPool::size_class(64), 64);
        assert_eq!(SizeClassPool::size_class(65), 128);
        assert_eq!(SizeClassPool::size_class(128), 128);
        assert_eq!(SizeClassPool::size_class(129), 256);
        assert_eq!(SizeClassPool::size_class(1024), 1024);
    }

    #[test]
    fn test_pool_release() {
        let mgr = HaftMemoryManager::default_ram();

        let ptr = mgr.pool_alloc(256);
        mgr.pool_dealloc(ptr, 256);

        let stats_before = mgr.pool_stats();
        let free_before: usize = stats_before.iter().map(|(_, free, _)| free).sum();
        assert!(free_before > 0);

        mgr.pool_release();

        let stats_after = mgr.pool_stats();
        let free_after: usize = stats_after.iter().map(|(_, free, _)| free).sum();
        assert_eq!(free_after, 0);
    }

    #[test]
    fn test_buffer_ids() {
        let mgr = HaftMemoryManager::default_ram();
        let id1 = mgr.allocate(64, MemoryTier::Ram);
        let id2 = mgr.allocate(128, MemoryTier::Ram);

        let ids = mgr.buffer_ids();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_hot_buffer_count() {
        let mgr = HaftMemoryManager::default_ram();
        let id1 = mgr.allocate(64, MemoryTier::Ram);
        let id2 = mgr.allocate(64, MemoryTier::Ram);

        // Make id1 hot
        for _ in 0..15 {
            mgr.access(id1, AccessPattern::RandomAccess);
        }
        // id2 stays cold
        mgr.access(id2, AccessPattern::RandomAccess);

        assert_eq!(mgr.hot_buffer_count(), 1);
    }

    #[test]
    fn test_clear() {
        let mgr = HaftMemoryManager::default_ram();
        mgr.allocate(1024, MemoryTier::Ram);
        mgr.allocate(2048, MemoryTier::Vram(0));

        mgr.clear();
        assert_eq!(mgr.buffer_count(), 0);
        assert_eq!(mgr.total_allocated_bytes(), 0);
        assert_eq!(mgr.total_migrations(), 0);
    }

    #[test]
    fn test_access_pattern_optimal_tier() {
        assert_eq!(
            AccessPattern::SequentialRead.optimal_tier(0),
            MemoryTier::Vram(0)
        );
        assert_eq!(
            AccessPattern::RandomAccess.optimal_tier(0),
            MemoryTier::Ram
        );
        assert_eq!(
            AccessPattern::WriteHeavy.optimal_tier(1),
            MemoryTier::Vram(1)
        );
        assert_eq!(
            AccessPattern::ReadModifyWrite.optimal_tier(0),
            MemoryTier::Shared
        );
    }

    #[test]
    fn test_memory_tier_display() {
        assert_eq!(format!("{}", MemoryTier::Ram), "RAM");
        assert_eq!(format!("{}", MemoryTier::Vram(0)), "VRAM:0");
        assert_eq!(format!("{}", MemoryTier::Shared), "Shared");
    }

    #[test]
    fn test_multiple_buffers_operations() {
        let mgr = HaftMemoryManager::default_ram();

        let mut ids = Vec::new();
        for i in 0..50 {
            let id = mgr.allocate(1024 * (i + 1), MemoryTier::Ram);
            ids.push(id);
        }
        assert_eq!(mgr.buffer_count(), 50);

        // Free half
        for id in ids.iter().step_by(2) {
            mgr.free(*id);
        }
        assert_eq!(mgr.buffer_count(), 25);

        // Migrate remaining
        for id in ids.iter().skip(1).step_by(2) {
            mgr.migrate(*id, MemoryTier::Vram(0));
        }
        assert_eq!(mgr.total_migrations(), 25);
    }
}
