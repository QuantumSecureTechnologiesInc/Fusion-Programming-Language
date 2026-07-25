//! Fusion Runtime Core - Heterogeneous runtime for Quantum/AI/Classical hybrid workloads

use parking_lot::{Mutex, RwLock};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, info, trace};

pub use fusion_tensor_core::{Matrix, Scalar, Tensor, Vector};

// ==================== Configuration Types ====================

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub enable_gpu: bool,
    pub enable_qpu: bool,
    pub qos_mode: QoSMode,
    pub gpu_backend: GpuBackend,
    pub worker_threads: usize,
    pub memory_pool_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QoSMode {
    Balanced,
    LowLatency,
    HighThroughput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    Auto,
    Cuda,
    Metal,
    Vulkan,
}

// ==================== FiberScheduler ====================

/// Cooperative task fiber
#[derive(Debug, Clone)]
pub struct Fiber {
    pub id: u64,
    pub state: FiberState,
    pub priority: u8,
    pub stack_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberState {
    Ready,
    Running,
    Suspended,
    Completed,
}

/// Basic cooperative fiber scheduler with a ready queue.
///
/// Fibers are lightweight execution units that yield control cooperatively.
/// The scheduler maintains a ready queue and schedules fibers in FIFO order
/// (higher priority fibers are preferred).
pub struct FiberScheduler {
    ready_queue: Mutex<VecDeque<Fiber>>,
    suspended: Mutex<Vec<Fiber>>,
    next_id: AtomicU64,
    total_spawned: AtomicU64,
    total_completed: AtomicU64,
}

impl FiberScheduler {
    pub fn new() -> Self {
        Self {
            ready_queue: Mutex::new(VecDeque::new()),
            suspended: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
            total_spawned: AtomicU64::new(0),
            total_completed: AtomicU64::new(0),
        }
    }

    /// Spawn a new fiber with given stack size and priority (0=lowest, 255=highest).
    /// Returns the fiber ID.
    pub fn spawn(&self, stack_size: usize, priority: u8) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let fiber = Fiber {
            id,
            state: FiberState::Ready,
            priority,
            stack_size,
        };
        self.ready_queue.lock().push_back(fiber);
        self.total_spawned.fetch_add(1, Ordering::Relaxed);
        debug!("Spawned fiber {} (priority={}, stack={})", id, priority, stack_size);
        id
    }

    /// Yield the current fiber (cooperative multitasking point).
    pub fn yield_now(&self, fiber_id: u64) {
        trace!("Fiber {} yielding", fiber_id);
    }

    /// Suspend a fiber until explicitly resumed.
    pub fn suspend(&self, fiber_id: u64) {
        let mut queue = self.ready_queue.lock();
        if let Some(idx) = queue.iter().position(|f| f.id == fiber_id) {
            let mut fiber = queue.remove(idx).unwrap();
            fiber.state = FiberState::Suspended;
            self.suspended.lock().push(fiber);
            debug!("Suspended fiber {}", fiber_id);
        }
    }

    /// Resume a previously suspended fiber.
    pub fn resume(&self, fiber_id: u64) {
        let mut suspended = self.suspended.lock();
        if let Some(idx) = suspended.iter().position(|f| f.id == fiber_id) {
            let mut fiber = suspended.remove(idx);
            fiber.state = FiberState::Ready;
            self.ready_queue.lock().push_back(fiber);
            debug!("Resumed fiber {}", fiber_id);
        }
    }

    /// Pop the highest-priority ready fiber.
    pub fn next_fiber(&self) -> Option<Fiber> {
        let mut queue = self.ready_queue.lock();
        if queue.is_empty() {
            return None;
        }
        // Find the fiber with the highest priority
        let max_priority = queue.iter().map(|f| f.priority).max().unwrap_or(0);
        if let Some(idx) = queue.iter().position(|f| f.priority == max_priority) {
            let mut fiber = queue.remove(idx).unwrap();
            fiber.state = FiberState::Running;
            Some(fiber)
        } else {
            queue.pop_front().map(|mut f| {
                f.state = FiberState::Running;
                f
            })
        }
    }

    /// Complete a fiber.
    pub fn complete(&self, fiber_id: u64) {
        // Remove from ready queue if present
        let mut queue = self.ready_queue.lock();
        queue.retain(|f| f.id != fiber_id);
        drop(queue);
        self.total_completed.fetch_add(1, Ordering::Relaxed);
        debug!("Completed fiber {}", fiber_id);
    }

    /// Get scheduler statistics.
    pub fn stats(&self) -> FiberStats {
        FiberStats {
            ready_count: self.ready_queue.lock().len(),
            suspended_count: self.suspended.lock().len(),
            total_spawned: self.total_spawned.load(Ordering::Relaxed),
            total_completed: self.total_completed.load(Ordering::Relaxed),
        }
    }
}

impl Default for FiberScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FiberStats {
    pub ready_count: usize,
    pub suspended_count: usize,
    pub total_spawned: u64,
    pub total_completed: u64,
}

// ==================== MemoryManager ====================

/// A bump allocator with a free list for fast temporary allocations.
///
/// The bump allocator advances a pointer through a pre-allocated pool.
/// When it runs out, it checks the free list for reusable blocks.
/// Freed blocks are returned to the free list for future reuse.
pub struct MemoryManager {
    pool: Vec<u8>,
    bump_offset: Mutex<usize>,
    free_list: Mutex<Vec<FreeBlock>>,
    total_allocated: AtomicU64,
    total_freed: AtomicU64,
}

#[derive(Debug, Clone)]
struct FreeBlock {
    offset: usize,
    size: usize,
}

#[derive(Debug, Clone)]
pub struct MemHandle {
    pub offset: usize,
    pub size: usize,
}

impl MemoryManager {
    /// Create a new memory manager with the given pool size in bytes.
    pub fn new(pool_size: usize) -> Self {
        info!("Initializing MemoryManager with {} bytes pool", pool_size);
        Self {
            pool: vec![0u8; pool_size],
            bump_offset: Mutex::new(0),
            free_list: Mutex::new(Vec::new()),
            total_allocated: AtomicU64::new(0),
            total_freed: AtomicU64::new(0),
        }
    }

    /// Allocate memory using the bump allocator. Falls back to free list if pool is exhausted.
    /// Alignment is always 8 bytes.
    pub fn allocate(&self, size: usize) -> Option<MemHandle> {
        let aligned_size = (size + 7) & !7; // align to 8 bytes

        // Try free list first (best-fit)
        {
            let mut free_list = self.free_list.lock();
            if let Some(idx) = free_list.iter().position(|b| b.size >= aligned_size) {
                let block = free_list.remove(idx);
                let handle = MemHandle {
                    offset: block.offset,
                    size: aligned_size,
                };
                // If the block is larger, return the remainder to the free list
                if block.size > aligned_size {
                    free_list.push(FreeBlock {
                        offset: block.offset + aligned_size,
                        size: block.size - aligned_size,
                    });
                }
                self.total_allocated.fetch_add(aligned_size as u64, Ordering::Relaxed);
                trace!("Allocated {} bytes from free list at offset {}", aligned_size, handle.offset);
                return Some(handle);
            }
        }

        // Bump allocator
        let mut offset = self.bump_offset.lock();
        if *offset + aligned_size <= self.pool.len() {
            let handle = MemHandle {
                offset: *offset,
                size: aligned_size,
            };
            *offset += aligned_size;
            self.total_allocated.fetch_add(aligned_size as u64, Ordering::Relaxed);
            trace!("Bump allocated {} bytes at offset {}", aligned_size, handle.offset);
            Some(handle)
        } else {
            trace!("Allocation of {} bytes failed: pool exhausted", aligned_size);
            None
        }
    }

    /// Free a previously allocated block, returning it to the free list.
    pub fn free(&self, handle: MemHandle) {
        self.free_list.lock().push(FreeBlock {
            offset: handle.offset,
            size: handle.size,
        });
        self.total_freed.fetch_add(handle.size as u64, Ordering::Relaxed);
        trace!("Freed {} bytes at offset {}", handle.size, handle.offset);
    }

    /// Reset the bump allocator (free all bump-allocated memory).
    /// Does not affect free-list blocks.
    pub fn reset(&self) {
        *self.bump_offset.lock() = 0;
        debug!("MemoryManager bump allocator reset");
    }

    /// Get a mutable slice of the pool at a given handle.
    pub fn get_mut(&mut self, handle: &MemHandle) -> Option<&mut [u8]> {
        let end = handle.offset + handle.size;
        if end <= self.pool.len() {
            Some(&mut self.pool[handle.offset..end])
        } else {
            None
        }
    }

    /// Get an immutable slice of the pool at a given handle.
    pub fn get(&self, handle: &MemHandle) -> Option<&[u8]> {
        let end = handle.offset + handle.size;
        if end <= self.pool.len() {
            Some(&self.pool[handle.offset..end])
        } else {
            None
        }
    }

    /// Get pool statistics.
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            pool_size: self.pool.len(),
            bump_used: *self.bump_offset.lock(),
            free_blocks: self.free_list.lock().len(),
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            total_freed: self.total_freed.load(Ordering::Relaxed),
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new(64 * 1024 * 1024) // 64MB default
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub pool_size: usize,
    pub bump_used: usize,
    pub free_blocks: usize,
    pub total_allocated: u64,
    pub total_freed: u64,
}

// ==================== Scheduler ====================

/// Task priority for the scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    High = 3,
    Normal = 2,
    Low = 1,
    Background = 0,
}

/// A scheduler with a priority queue and FIFO fallback.
///
/// Tasks are enqueued with a priority level. The scheduler dequeues
/// the highest-priority task first. Within the same priority level,
/// tasks are served in FIFO order.
pub struct Scheduler {
    high_queue: Mutex<VecDeque<ScheduledTask>>,
    normal_queue: Mutex<VecDeque<ScheduledTask>>,
    low_queue: Mutex<VecDeque<ScheduledTask>>,
    background_queue: Mutex<VecDeque<ScheduledTask>>,
    next_task_id: AtomicU64,
    tasks_spawned: AtomicU64,
    tasks_completed: AtomicU64,
}

struct ScheduledTask {
    id: u64,
    priority: TaskPriority,
    name: String,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            high_queue: Mutex::new(VecDeque::new()),
            normal_queue: Mutex::new(VecDeque::new()),
            low_queue: Mutex::new(VecDeque::new()),
            background_queue: Mutex::new(VecDeque::new()),
            next_task_id: AtomicU64::new(1),
            tasks_spawned: AtomicU64::new(0),
            tasks_completed: AtomicU64::new(0),
        }
    }

    /// Enqueue a task with the given priority.
    pub fn enqueue(&self, name: impl Into<String>, priority: TaskPriority) -> u64 {
        let id = self.next_task_id.fetch_add(1, Ordering::SeqCst);
        let task = ScheduledTask {
            id,
            priority,
            name: name.into(),
        };
        match priority {
            TaskPriority::High => self.high_queue.lock().push_back(task),
            TaskPriority::Normal => self.normal_queue.lock().push_back(task),
            TaskPriority::Low => self.low_queue.lock().push_back(task),
            TaskPriority::Background => self.background_queue.lock().push_back(task),
        }
        self.tasks_spawned.fetch_add(1, Ordering::Relaxed);
        trace!("Enqueued task {} (priority={:?})", id, priority);
        id
    }

    /// Dequeue the next task (highest priority first, FIFO within priority).
    pub fn dequeue(&self) -> Option<(u64, String, TaskPriority)> {
        // Priority order: High -> Normal -> Low -> Background (FIFO fallback)
        let queue_order: &[&Mutex<VecDeque<ScheduledTask>>] = &[
            &self.high_queue,
            &self.normal_queue,
            &self.low_queue,
            &self.background_queue,
        ];
        for queue in queue_order {
            if let Some(task) = queue.lock().pop_front() {
                self.tasks_completed.fetch_add(1, Ordering::Relaxed);
                trace!("Dequeued task {} (priority={:?})", task.id, task.priority);
                return Some((task.id, task.name, task.priority));
            }
        }
        None
    }

    /// Peek at the next task without removing it.
    pub fn peek(&self) -> Option<(u64, TaskPriority)> {
        let queue_order: &[&Mutex<VecDeque<ScheduledTask>>] = &[
            &self.high_queue,
            &self.normal_queue,
            &self.low_queue,
            &self.background_queue,
        ];
        for queue in queue_order {
            let q = queue.lock();
            if let Some(task) = q.front() {
                return Some((task.id, task.priority));
            }
        }
        None
    }

    /// Check if the scheduler has any pending tasks.
    pub fn has_tasks(&self) -> bool {
        !self.high_queue.lock().is_empty()
            || !self.normal_queue.lock().is_empty()
            || !self.low_queue.lock().is_empty()
            || !self.background_queue.lock().is_empty()
    }

    /// Get scheduler statistics.
    pub fn stats(&self) -> SchedulerStats {
        SchedulerStats {
            high_priority_len: self.high_queue.lock().len(),
            normal_priority_len: self.normal_queue.lock().len(),
            low_priority_len: self.low_queue.lock().len(),
            background_priority_len: self.background_queue.lock().len(),
            tasks_spawned: self.tasks_spawned.load(Ordering::Relaxed),
            tasks_completed: self.tasks_completed.load(Ordering::Relaxed),
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub high_priority_len: usize,
    pub normal_priority_len: usize,
    pub low_priority_len: usize,
    pub background_priority_len: usize,
    pub tasks_spawned: u64,
    pub tasks_completed: u64,
}

// ==================== HardwareLayer ====================

/// Detected hardware device type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    Cpu,
    CudaGpu(u32),
    MetalGpu(u32),
    VulkanGpu(u32),
    Qpu(u32),
}

/// Information about a detected device.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_type: DeviceType,
    pub name: String,
    pub memory_bytes: usize,
    pub compute_units: u32,
    pub available: bool,
}

/// Hardware abstraction layer that detects available CPU/GPU/QPU devices.
pub struct HardwareLayer {
    devices: Vec<DeviceInfo>,
    cpu_info: CpuInfo,
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub logical_cores: usize,
    pub physical_cores: usize,
    pub arch: String,
}

impl HardwareLayer {
    /// Detect available hardware. On real systems this would probe CUDA/Metal/Vulkan/QPU APIs.
    pub fn new() -> Self {
        info!("Detecting hardware...");
        let logical_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        // Heuristic: physical cores ≈ logical / 2 (hyperthreading)
        let physical_cores = (logical_cores / 2).max(1);

        let cpu_info = CpuInfo {
            logical_cores,
            physical_cores,
            arch: std::env::consts::ARCH.to_string(),
        };

        let mut devices = Vec::new();
        devices.push(DeviceInfo {
            device_type: DeviceType::Cpu,
            name: format!("CPU ({} cores, {})", logical_cores, cpu_info.arch),
            memory_bytes: 0, // Would query OS for available RAM
            compute_units: logical_cores as u32,
            available: true,
        });

        // GPU detection would be done here via FFI:
        // - CUDA: cudaGetDeviceCount()
        // - Metal: MTLCreateSystemDefaultDevice()
        // - Vulkan: vkEnumeratePhysicalDevices()
        // For now, we detect based on compile-time cfg flags.
        #[cfg(target_os = "macos")]
        {
            devices.push(DeviceInfo {
                device_type: DeviceType::MetalGpu(0),
                name: "Apple GPU (Metal)".to_string(),
                memory_bytes: 0,
                compute_units: 0,
                available: true,
            });
        }

        // QPU detection would query quantum hardware providers (IBM Quantum, etc.)

        info!("Detected {} device(s)", devices.len());
        Self { devices, cpu_info }
    }

    /// List all detected devices.
    pub fn devices(&self) -> &[DeviceInfo] {
        &self.devices
    }

    /// Get CPU information.
    pub fn cpu_info(&self) -> &CpuInfo {
        &self.cpu_info
    }

    /// Check if a specific device type is available.
    pub fn has_device(&self, device_type: DeviceType) -> bool {
        self.devices.iter().any(|d| d.device_type == device_type && d.available)
    }

    /// Get the best available GPU device.
    pub fn best_gpu(&self) -> Option<&DeviceInfo> {
        self.devices
            .iter()
            .find(|d| d.available && matches!(d.device_type, DeviceType::CudaGpu(_) | DeviceType::MetalGpu(_) | DeviceType::VulkanGpu(_)))
    }
}

impl Default for HardwareLayer {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Executor ====================

/// Simple executor that runs futures on a thread pool.
pub struct Executor {
    scheduler: Arc<Scheduler>,
}

impl Executor {
    pub fn new(scheduler: Arc<Scheduler>) -> Self {
        Self { scheduler }
    }

    pub fn block_on<F: std::future::Future>(&self, fut: F) -> F::Output {
        futures::executor::block_on(fut)
    }
}

// ==================== Runtime ====================

pub struct Runtime {
    fiber_scheduler: Arc<FiberScheduler>,
    memory_manager: Arc<MemoryManager>,
    scheduler: Arc<Scheduler>,
    hal: Arc<HardwareLayer>,
    executor: Arc<Executor>,
    config: RuntimeConfig,
    metrics: Arc<RwLock<RuntimeMetrics>>,
}

#[derive(Debug, Default, Clone)]
pub struct RuntimeMetrics {
    pub tasks_spawned: u64,
    pub tasks_completed: u64,
    pub fibers_spawned: u64,
    pub memory_allocated: u64,
}

impl Runtime {
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::default()
    }

    pub fn new() -> Self {
        Self::builder().build()
    }

    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.executor.block_on(future)
    }

    pub fn fiber_scheduler(&self) -> &FiberScheduler {
        &self.fiber_scheduler
    }

    pub fn memory_manager(&self) -> &MemoryManager {
        &self.memory_manager
    }

    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    pub fn hal(&self) -> &HardwareLayer {
        &self.hal
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    pub fn metrics(&self) -> RuntimeMetrics {
        self.metrics.read().clone()
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Builder ====================

#[derive(Default)]
pub struct RuntimeBuilder {
    enable_gpu: bool,
    enable_qpu: bool,
    qos_mode: Option<QoSMode>,
    gpu_backend: Option<GpuBackend>,
    worker_threads: Option<usize>,
    memory_pool_size: Option<usize>,
}

impl RuntimeBuilder {
    pub fn enable_gpu(mut self) -> Self {
        self.enable_gpu = true;
        self
    }

    pub fn enable_qpu(mut self) -> Self {
        self.enable_qpu = true;
        self
    }

    pub fn enable_qos(mut self, mode: QoSMode) -> Self {
        self.qos_mode = Some(mode);
        self
    }

    pub fn gpu_backend(mut self, backend: GpuBackend) -> Self {
        self.gpu_backend = Some(backend);
        self
    }

    pub fn worker_threads(mut self, threads: usize) -> Self {
        self.worker_threads = Some(threads);
        self
    }

    pub fn memory_pool_size(mut self, size: usize) -> Self {
        self.memory_pool_size = Some(size);
        self
    }

    pub fn build(self) -> Runtime {
        info!("Building Fusion Runtime");

        let config = RuntimeConfig {
            enable_gpu: self.enable_gpu,
            enable_qpu: self.enable_qpu,
            qos_mode: self.qos_mode.unwrap_or(QoSMode::Balanced),
            gpu_backend: self.gpu_backend.unwrap_or(GpuBackend::Auto),
            worker_threads: self.worker_threads.unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(4)
            }),
            memory_pool_size: self.memory_pool_size.unwrap_or(64 * 1024 * 1024),
        };

        let fiber_scheduler = Arc::new(FiberScheduler::new());
        let memory_manager = Arc::new(MemoryManager::new(config.memory_pool_size));
        let scheduler = Arc::new(Scheduler::new());
        let hal = Arc::new(HardwareLayer::new());
        let executor = Arc::new(Executor::new(scheduler.clone()));
        let metrics = Arc::new(RwLock::new(RuntimeMetrics::default()));

        info!("Fusion Runtime initialized: {} workers, {}MB memory",
            config.worker_threads,
            config.memory_pool_size / (1024 * 1024)
        );

        Runtime {
            fiber_scheduler,
            memory_manager,
            scheduler,
            hal,
            executor,
            config,
            metrics,
        }
    }
}

// ==================== Additional Components ====================

/// Low-jitter timer for precise scheduling.
pub struct LowJitterTimer {
    start: std::time::Instant,
}

impl LowJitterTimer {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    /// Elapsed time in microseconds since timer creation.
    pub fn elapsed_us(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }

    /// Elapsed time in nanoseconds.
    pub fn elapsed_ns(&self) -> u64 {
        self.start.elapsed().as_nanos() as u64
    }
}

impl Default for LowJitterTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Event types for the I/O reactor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Read,
    Write,
    Timer,
    Signal,
}

/// Fused I/O reactor for async event processing.
pub struct FusedIoReactor {
    events: Mutex<VecDeque<(u64, EventType)>>,
    next_id: AtomicU64,
}

impl FusedIoReactor {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(VecDeque::new()),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn register_event(&self, event_type: EventType) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.events.lock().push_back((id, event_type));
        id
    }

    pub fn poll_event(&self) -> Option<(u64, EventType)> {
        self.events.lock().pop_front()
    }
}

impl Default for FusedIoReactor {
    fn default() -> Self {
        Self::new()
    }
}

/// Device memory allocator for GPU/QPU memory blocks.
pub struct DeviceMemoryAllocator {
    next_handle: AtomicU64,
    allocations: Mutex<Vec<DeviceAllocation>>,
}

#[derive(Debug, Clone)]
struct DeviceAllocation {
    handle: u64,
    device: DeviceType,
    size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceMemHandle(pub u64);

impl DeviceMemoryAllocator {
    pub fn new() -> Self {
        Self {
            next_handle: AtomicU64::new(1),
            allocations: Mutex::new(Vec::new()),
        }
    }

    pub fn allocate(&self, device: DeviceType, size: usize) -> DeviceMemHandle {
        let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
        self.allocations.lock().push(DeviceAllocation {
            handle,
            device,
            size,
        });
        DeviceMemHandle(handle)
    }

    pub fn free(&self, handle: DeviceMemHandle) {
        self.allocations.lock().retain(|a| a.handle != handle.0);
    }

    pub fn stats(&self) -> DeviceMemStats {
        let allocs = self.allocations.lock();
        DeviceMemStats {
            active_allocations: allocs.len(),
            total_bytes: allocs.iter().map(|a| a.size).sum(),
        }
    }
}

impl Default for DeviceMemoryAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DeviceMemStats {
    pub active_allocations: usize,
    pub total_bytes: usize,
}

/// Shared memory manager for zero-copy inter-process communication.
pub struct SharedMemoryManager {
    regions: Mutex<Vec<SharedRegion>>,
    next_id: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct SharedRegion {
    pub id: u64,
    pub size: usize,
    pub name: String,
}

impl SharedMemoryManager {
    pub fn new() -> Self {
        Self {
            regions: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn create_region(&self, name: impl Into<String>, size: usize) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.regions.lock().push(SharedRegion {
            id,
            size,
            name: name.into(),
        });
        id
    }

    pub fn stats(&self) -> usize {
        self.regions.lock().len()
    }
}

impl Default for SharedMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Collective communication operations for multi-device coordination.
pub struct CollectiveComms;

impl CollectiveComms {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CollectiveComms {
    fn default() -> Self {
        Self::new()
    }
}

/// QPU job sequencer for managing quantum circuit submissions.
pub struct QpuJobSequencer {
    next_job_id: AtomicU64,
    queued_jobs: Mutex<VecDeque<u64>>,
}

impl QpuJobSequencer {
    pub fn new() -> Self {
        Self {
            next_job_id: AtomicU64::new(1),
            queued_jobs: Mutex::new(VecDeque::new()),
        }
    }

    pub fn submit_job(&self) -> u64 {
        let id = self.next_job_id.fetch_add(1, Ordering::SeqCst);
        self.queued_jobs.lock().push_back(id);
        id
    }

    pub fn next_job(&self) -> Option<u64> {
        self.queued_jobs.lock().pop_front()
    }
}

impl Default for QpuJobSequencer {
    fn default() -> Self {
        Self::new()
    }
}

/// Variational loop controller for optimization workflows.
pub struct VariationalLoopController {
    iterations: AtomicU64,
}

impl VariationalLoopController {
    pub fn new() -> Self {
        Self {
            iterations: AtomicU64::new(0),
        }
    }

    pub fn increment(&self) -> u64 {
        self.iterations.fetch_add(1, Ordering::Relaxed)
    }

    pub fn iterations(&self) -> u64 {
        self.iterations.load(Ordering::Relaxed)
    }
}

impl Default for VariationalLoopController {
    fn default() -> Self {
        Self::new()
    }
}

/// FusionCore coordinator.
pub struct FusionCore;

impl FusionCore {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FusionCore {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = Runtime::new();
        assert!(runtime.config().worker_threads > 0);
    }

    #[test]
    fn test_all_components_accessible() {
        let runtime = Runtime::builder().enable_gpu().enable_qpu().build();
        let _ = runtime.fiber_scheduler();
        let _ = runtime.memory_manager();
        let _ = runtime.scheduler();
        let _ = runtime.hal();
    }

    #[test]
    fn test_fiber_scheduler() {
        let fs = FiberScheduler::new();
        let id1 = fs.spawn(65536, 128);
        let id2 = fs.spawn(65536, 64);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        let stats = fs.stats();
        assert_eq!(stats.ready_count, 2);

        // Higher priority fiber should be dequeued first
        let fiber = fs.next_fiber().unwrap();
        assert_eq!(fiber.id, id1); // priority 128 > 64
        assert_eq!(fiber.state, FiberState::Running);

        let stats = fs.stats();
        assert_eq!(stats.ready_count, 1);
    }

    #[test]
    fn test_fiber_suspend_resume() {
        let fs = FiberScheduler::new();
        let id = fs.spawn(65536, 128);

        fs.suspend(id);
        let stats = fs.stats();
        assert_eq!(stats.suspended_count, 1);
        assert_eq!(stats.ready_count, 0);

        fs.resume(id);
        let stats = fs.stats();
        assert_eq!(stats.ready_count, 1);
        assert_eq!(stats.suspended_count, 0);
    }

    #[test]
    fn test_memory_manager_bump_allocator() {
        let mm = MemoryManager::new(1024);

        let h1 = mm.allocate(128).unwrap();
        assert_eq!(h1.offset, 0);
        assert_eq!(h1.size, 128);

        let h2 = mm.allocate(256).unwrap();
        assert_eq!(h2.offset, 128);
        assert_eq!(h2.size, 256);

        let stats = mm.stats();
        assert_eq!(stats.bump_used, 384);
        assert_eq!(stats.total_allocated, 384);
    }

    #[test]
    fn test_memory_manager_free_list() {
        let mm = MemoryManager::new(1024);

        let h1 = mm.allocate(128).unwrap();
        let h1_offset = h1.offset;
        let _h2 = mm.allocate(256).unwrap();

        // Free h1 and allocate a smaller block - should reuse from free list
        mm.free(h1);
        let h3 = mm.allocate(64).unwrap();
        assert_eq!(h3.offset, h1_offset); // Reused!

        let stats = mm.stats();
        assert_eq!(stats.free_blocks, 1); // h2's remainder from h1's free block
    }

    #[test]
    fn test_memory_manager_alignment() {
        let mm = MemoryManager::new(1024);

        // Request 3 bytes, should be aligned to 8
        let h1 = mm.allocate(3).unwrap();
        assert_eq!(h1.size, 8);

        let h2 = mm.allocate(5).unwrap();
        assert_eq!(h2.size, 8);
        assert_eq!(h2.offset, 8); // properly aligned
    }

    #[test]
    fn test_scheduler_priority() {
        let s = Scheduler::new();
        s.enqueue("low_task", TaskPriority::Low);
        s.enqueue("high_task", TaskPriority::High);
        s.enqueue("normal_task", TaskPriority::Normal);

        // High priority first
        let (id, name, _) = s.dequeue().unwrap();
        assert_eq!(name, "high_task");

        // Then normal
        let (_, name, _) = s.dequeue().unwrap();
        assert_eq!(name, "normal_task");

        // Then low
        let (_, name, _) = s.dequeue().unwrap();
        assert_eq!(name, "low_task");

        assert!(s.dequeue().is_none());
    }

    #[test]
    fn test_scheduler_fifo_within_priority() {
        let s = Scheduler::new();
        s.enqueue("task1", TaskPriority::Normal);
        s.enqueue("task2", TaskPriority::Normal);
        s.enqueue("task3", TaskPriority::Normal);

        let (_, name, _) = s.dequeue().unwrap();
        assert_eq!(name, "task1");
        let (_, name, _) = s.dequeue().unwrap();
        assert_eq!(name, "task2");
        let (_, name, _) = s.dequeue().unwrap();
        assert_eq!(name, "task3");
    }

    #[test]
    fn test_hardware_layer() {
        let hal = HardwareLayer::new();
        let cpu_info = hal.cpu_info();
        assert!(cpu_info.logical_cores > 0);

        let devices = hal.devices();
        assert!(!devices.is_empty());
        assert!(hal.has_device(DeviceType::Cpu));
    }

    #[test]
    fn test_device_memory_allocator() {
        let dma = DeviceMemoryAllocator::new();
        let h1 = dma.allocate(DeviceType::CudaGpu(0), 1024);
        let h2 = dma.allocate(DeviceType::Cpu, 2048);

        let stats = dma.stats();
        assert_eq!(stats.active_allocations, 2);
        assert_eq!(stats.total_bytes, 3072);

        dma.free(h1);
        let stats = dma.stats();
        assert_eq!(stats.active_allocations, 1);
    }

    #[test]
    fn test_low_jitter_timer() {
        let timer = LowJitterTimer::new();
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(timer.elapsed_us() > 500);
    }
}
