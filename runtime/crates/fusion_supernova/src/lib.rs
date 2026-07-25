//! # Fusion Supernova
//!
//! Tribrid execution engine that dispatches work across CPU, GPU, and QPU
//! devices. Provides a work-stealing scheduler with per-device work queues,
//! zero-copy memory transfers, and automatic fault tolerance with QPU-to-simulator
//! fallback.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────┐
//! │              SupernovaEngine                  │
//! │  ┌────────┐  ┌────────┐  ┌────────┐          │
//! │  │ CPU Q  │  │ GPU Q  │  │ QPU Q  │  ← Per-device queues
//! │  └───┬────┘  └───┬────┘  └───┬────┘          │
//! │      │           │           │                 │
//! │  ┌───▼───────────▼───────────▼────┐           │
//! │  │      Work-Stealing Scheduler   │           │
//! │  └────────────────────────────────┘           │
//! │  ┌────────────────────────────────┐           │
//! │  │     Zero-Copy Memory Layer     │           │
//! │  └────────────────────────────────┘           │
//! │  ┌────────────────────────────────┐           │
//! │  │   Fault Tolerance (QPU→Sim)    │           │
//! │  └────────────────────────────────┘           │
//! └──────────────────────────────────────────────┘
//! ```

use crossbeam::deque::{Steal, Stealer, Worker};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tracing::{debug, info, warn};

// ─── Device Types ──────────────────────────────────────────────

/// Hardware device target for task dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Device {
    Cpu,
    Gpu(u32),
    Qpu(u32),
}

impl Device {
    pub fn is_cpu(&self) -> bool { matches!(self, Device::Cpu) }
    pub fn is_gpu(&self) -> bool { matches!(self, Device::Gpu(_)) }
    pub fn is_qpu(&self) -> bool { matches!(self, Device::Qpu(_)) }

    pub fn device_index(&self) -> Option<u32> {
        match self {
            Device::Cpu => None,
            Device::Gpu(i) | Device::Qpu(i) => Some(*i),
        }
    }
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Device::Cpu => write!(f, "CPU"),
            Device::Gpu(i) => write!(f, "GPU:{}", i),
            Device::Qpu(i) => write!(f, "QPU:{}", i),
        }
    }
}

// ─── Task Representation ───────────────────────────────────────

/// A unique task identifier.
pub type TaskId = u64;

/// The priority level of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// A unit of work dispatched to a device.
#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub priority: Priority,
    pub estimated_ops: u64,
    pub memory_bytes: usize,
    pub preferred_device: Option<Device>,
}

impl Task {
    pub fn new(id: TaskId, priority: Priority, estimated_ops: u64, memory_bytes: usize) -> Self {
        Self {
            id,
            priority,
            estimated_ops,
            memory_bytes,
            preferred_device: None,
        }
    }

    pub fn with_device(mut self, device: Device) -> Self {
        self.preferred_device = Some(device);
        self
    }
}

// ─── Work Queue ────────────────────────────────────────────────

/// Per-device work queue backed by a crossbeam `Worker`.
pub struct DeviceQueue {
    worker: Worker<Task>,
    stealer: Stealer<Task>,
    device: Device,
    task_count: AtomicU64,
}

impl DeviceQueue {
    fn new(device: Device) -> Self {
        let worker = Worker::new_lifo();
        let stealer = worker.stealer();
        Self {
            worker,
            stealer,
            device,
            task_count: AtomicU64::new(0),
        }
    }

    /// Push a task onto this device's queue.
    pub fn push(&self, task: Task) {
        self.worker.push(task);
        self.task_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Pop a task from this device's queue (LIFO).
    pub fn pop(&self) -> Option<Task> {
        let task = self.worker.pop();
        if task.is_some() {
            self.task_count.fetch_sub(1, Ordering::Relaxed);
        }
        task
    }

    /// Attempt to steal a task from this queue (FIFO steal order).
    pub fn steal_from(&self) -> Option<Task> {
        match self.stealer.steal() {
            Steal::Success(task) => {
                self.task_count.fetch_sub(1, Ordering::Relaxed);
                Some(task)
            }
            _ => None,
        }
    }

    /// Number of tasks currently in this queue.
    pub fn len(&self) -> u64 {
        self.task_count.load(Ordering::Relaxed)
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn device(&self) -> Device {
        self.device
    }
}

// ─── Zero-Copy Memory Transfer ─────────────────────────────────

/// A region of shared memory that can be accessed by multiple devices
/// without copying. Tracks which devices currently hold references.
#[derive(Debug, Clone)]
pub struct SharedBuffer {
    id: u64,
    size: usize,
    ptr: *mut u8,
    holders: Vec<Device>,
    is_cpu_backed: bool,
}

// SAFETY: SharedBuffer is used within a single-threaded executor context
// and protected by the SupernovaEngine's RwLock.
unsafe impl Send for SharedBuffer {}
unsafe impl Sync for SharedBuffer {}

impl SharedBuffer {
    /// Allocate a new shared buffer backed by host (CPU) memory.
    pub fn new_cpu(id: u64, size: usize) -> Self {
        let layout = std::alloc::Layout::array::<u8>(size).unwrap();
        // SAFETY: Layout is non-zero; we handle deallocation in Drop.
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        Self {
            id,
            size,
            ptr,
            holders: vec![Device::Cpu],
            is_cpu_backed: true,
        }
    }

    pub fn id(&self) -> u64 { self.id }
    pub fn size(&self) -> usize { self.size }

    /// Get a raw pointer to the buffer data.
    pub fn as_ptr(&self) -> *const u8 { self.ptr }

    /// Get a mutable raw pointer to the buffer data.
    pub fn as_mut_ptr(&mut self) -> *mut u8 { self.ptr }

    /// Record that a device now holds a reference to this buffer.
    /// In a real system this would trigger a GPU/CPU mapping.
    pub fn attach_device(&mut self, device: Device) {
        if !self.holders.contains(&device) {
            debug!("SharedBuffer:{} attaching {:?} (zero-copy map)", self.id, device);
            self.holders.push(device);
        }
    }

    /// Record that a device has released its reference.
    pub fn detach_device(&mut self, device: Device) {
        self.holders.retain(|d| d != &device);
        debug!("SharedBuffer:{} detached {:?}", self.id, device);
    }

    /// Check if the buffer is currently shared across devices.
    pub fn is_shared(&self) -> bool {
        self.holders.len() > 1
    }

    /// Read bytes from the buffer.
    pub fn read_bytes(&self, offset: usize, len: usize) -> Vec<u8> {
        assert!(offset + len <= self.size, "read out of bounds");
        // SAFETY: we checked bounds above; ptr is valid for `size` bytes.
        unsafe { std::slice::from_raw_parts(self.ptr.add(offset), len).to_vec() }
    }

    /// Write bytes into the buffer.
    pub fn write_bytes(&mut self, offset: usize, data: &[u8]) {
        assert!(offset + data.len() <= self.size, "write out of bounds");
        // SAFETY: we checked bounds; ptr is valid for `size` bytes.
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.ptr.add(offset), data.len());
        }
    }
}

impl Drop for SharedBuffer {
    fn drop(&mut self) {
        if self.is_cpu_backed && !self.ptr.is_null() {
            let layout = std::alloc::Layout::array::<u8>(self.size).unwrap();
            // SAFETY: ptr was allocated with the same layout in new_cpu.
            unsafe { std::alloc::dealloc(self.ptr, layout) };
        }
    }
}

// ─── Fault Tolerance ───────────────────────────────────────────

/// QPU execution status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QpuStatus {
    Available,
    Degraded,
    Unavailable,
}

/// Tracks QPU health and provides simulator fallback.
pub struct FaultTolerance {
    qpu_status: RwLock<HashMap<u32, QpuStatus>>,
    fallback_to_sim: AtomicBool,
    failure_count: AtomicU64,
}

impl FaultTolerance {
    pub fn new() -> Self {
        Self {
            qpu_status: RwLock::new(HashMap::new()),
            fallback_to_sim: AtomicBool::new(true),
            failure_count: AtomicU64::new(0),
        }
    }

    /// Check if a specific QPU is available.
    pub fn qpu_available(&self, qpu_id: u32) -> bool {
        let status = self.qpu_status.read();
        matches!(status.get(&qpu_id), Some(QpuStatus::Available))
    }

    /// Check if fallback to CPU simulator is enabled.
    pub fn fallback_enabled(&self) -> bool {
        self.fallback_to_sim.load(Ordering::Relaxed)
    }

    /// Report a QPU failure and potentially trigger fallback.
    pub fn report_qpu_failure(&self, qpu_id: u32) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        warn!("QPU:{} failure reported, total failures: {}", qpu_id, self.failure_count.load(Ordering::Relaxed));

        let mut status = self.qpu_status.write();
        status.insert(qpu_id, QpuStatus::Degraded);
    }

    /// Mark a QPU as recovered.
    pub fn report_qpu_recovery(&self, qpu_id: u32) {
        info!("QPU:{} recovered", qpu_id);
        let mut status = self.qpu_status.write();
        status.insert(qpu_id, QpuStatus::Available);
    }

    /// Get the total failure count.
    pub fn failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::Relaxed)
    }

    /// Select the best device, falling back from QPU to CPU if needed.
    pub fn select_device(&self, preferred: Device) -> Device {
        match preferred {
            Device::Qpu(id) => {
                if self.qpu_available(id) {
                    preferred
                } else if self.fallback_enabled() {
                    debug!("QPU:{} unavailable, falling back to CPU simulator", id);
                    Device::Cpu
                } else {
                    preferred
                }
            }
            _ => preferred,
        }
    }
}

impl Default for FaultTolerance {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Supernova Engine ──────────────────────────────────────────

/// Global task ID counter.
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

/// The Supernova tribrid execution engine.
///
/// Manages per-device work queues, dispatches tasks via work-stealing,
/// handles zero-copy memory, and provides QPU fault tolerance.
pub struct SupernovaEngine {
    cpu_queue: DeviceQueue,
    gpu_queues: Vec<DeviceQueue>,
    qpu_queues: Vec<DeviceQueue>,
    buffers: RwLock<Vec<SharedBuffer>>,
    fault_tolerance: FaultTolerance,
    total_dispatched: AtomicU64,
    num_gpus: u32,
    num_qpus: u32,
}

impl SupernovaEngine {
    /// Create a new SupernovaEngine with the given device counts.
    pub fn new(num_gpus: u32, num_qpus: u32) -> Self {
        info!(
            "Initializing SupernovaEngine: 1 CPU, {} GPU(s), {} QPU(s)",
            num_gpus, num_qpus
        );

        let mut gpu_queues = Vec::with_capacity(num_gpus as usize);
        for i in 0..num_gpus {
            gpu_queues.push(DeviceQueue::new(Device::Gpu(i)));
        }

        let mut qpu_queues = Vec::with_capacity(num_qpus as usize);
        for i in 0..num_qpus {
            qpu_queues.push(DeviceQueue::new(Device::Qpu(i)));
        }

        Self {
            cpu_queue: DeviceQueue::new(Device::Cpu),
            gpu_queues,
            qpu_queues,
            buffers: RwLock::new(Vec::new()),
            fault_tolerance: FaultTolerance::new(),
            total_dispatched: AtomicU64::new(0),
            num_gpus,
            num_qpus,
        }
    }

    /// Create a minimal engine with CPU only.
    pub fn cpu_only() -> Self {
        Self::new(0, 0)
    }

    /// Generate a unique task ID.
    pub fn next_task_id(&self) -> TaskId {
        NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed)
    }

    /// Dispatch a task to the appropriate device queue.
    ///
    /// If the task has a preferred device, it goes there. Otherwise the engine
    /// selects based on task characteristics and fault tolerance state.
    pub fn dispatch(&self, task: Task) -> Device {
        let task_id = task.id;
        let mut device = self.select_device(&task);

        match device {
            Device::Cpu => {
                self.cpu_queue.push(task);
                debug!("Task {} dispatched to CPU", task_id);
            }
            Device::Gpu(idx) => {
                if (idx as usize) < self.gpu_queues.len() {
                    self.gpu_queues[idx as usize].push(task);
                    debug!("Task {} dispatched to GPU:{}", task_id, idx);
                } else {
                    warn!("GPU:{} not available, falling back to CPU", idx);
                    self.cpu_queue.push(task);
                    device = Device::Cpu;
                }
            }
            Device::Qpu(idx) => {
                let resolved = self.fault_tolerance.select_device(Device::Qpu(idx));
                match resolved {
                    Device::Qpu(q) => {
                        if (q as usize) < self.qpu_queues.len() {
                            self.qpu_queues[q as usize].push(task);
                            debug!("Task {} dispatched to QPU:{}", task_id, q);
                        } else {
                            self.cpu_queue.push(task);
                            debug!("Task {} QPU:{} queue unavailable, fell back to CPU", task_id, q);
                            device = Device::Cpu;
                        }
                    }
                    Device::Cpu => {
                        self.cpu_queue.push(task);
                        debug!("Task {} fell back to CPU from QPU:{}", task_id, idx);
                        device = Device::Cpu;
                    }
                    _ => unreachable!(),
                }
            }
        }

        self.total_dispatched.fetch_add(1, Ordering::Relaxed);
        device
    }

    /// Select the best device for a task based on its characteristics.
    fn select_device(&self, task: &Task) -> Device {
        // Respect explicit preference
        if let Some(pref) = task.preferred_device {
            return pref;
        }

        // Critical tasks always go to CPU for lowest latency
        if task.priority == Priority::Critical {
            return Device::Cpu;
        }

        // Large memory tasks prefer GPU (high bandwidth)
        if task.memory_bytes > 100 * 1024 * 1024 && self.num_gpus > 0 {
            return Device::Gpu(0);
        }

        // High compute tasks prefer GPU
        if task.estimated_ops > 1_000_000_000 && self.num_gpus > 0 {
            return Device::Gpu(0);
        }

        // Small tasks stay on CPU
        Device::Cpu
    }

    /// Try to steal a task from another device's queue.
    ///
    /// This is used when a device's own queue is empty and it wants more work.
    pub fn steal_task(&self, from: Device) -> Option<Task> {
        match from {
            Device::Cpu => self.cpu_queue.steal_from(),
            Device::Gpu(idx) => {
                if (idx as usize) < self.gpu_queues.len() {
                    self.gpu_queues[idx as usize].steal_from()
                } else {
                    None
                }
            }
            Device::Qpu(idx) => {
                if (idx as usize) < self.qpu_queues.len() {
                    self.qpu_queues[idx as usize].steal_from()
                } else {
                    None
                }
            }
        }
    }

    /// Try to steal work from any device's queue to balance load.
    pub fn steal_any(&self, current: Device) -> Option<(Task, Device)> {
        // Try stealing from other device types in order
        let targets: Vec<Device> = match current {
            Device::Cpu => {
                let mut t = Vec::new();
                for i in 0..self.num_gpus { t.push(Device::Gpu(i)); }
                for i in 0..self.num_qpus { t.push(Device::Qpu(i)); }
                t
            }
            Device::Gpu(_) => {
                let mut t = vec![Device::Cpu];
                for i in 0..self.num_qpus { t.push(Device::Qpu(i)); }
                t
            }
            Device::Qpu(_) => {
                let mut t = vec![Device::Cpu];
                for i in 0..self.num_gpus { t.push(Device::Gpu(i)); }
                t
            }
        };

        for target in targets {
            if let Some(task) = self.steal_task(target) {
                return Some((task, target));
            }
        }
        None
    }

    /// Allocate a shared memory buffer.
    pub fn alloc_buffer(&self, size: usize) -> u64 {
        let id = self.buffers.read().len() as u64;
        let buf = SharedBuffer::new_cpu(id, size);
        self.buffers.write().push(buf);
        id
    }

    /// Attach a device to an existing buffer (zero-copy mapping).
    pub fn attach_buffer(&self, buffer_id: u64, device: Device) {
        let mut buffers = self.buffers.write();
        if let Some(buf) = buffers.get_mut(buffer_id as usize) {
            buf.attach_device(device);
        }
    }

    /// Read data from a shared buffer.
    pub fn read_buffer(&self, buffer_id: u64, offset: usize, len: usize) -> Vec<u8> {
        let buffers = self.buffers.read();
        buffers
            .get(buffer_id as usize)
            .map(|buf| buf.read_bytes(offset, len))
            .unwrap_or_default()
    }

    /// Write data into a shared buffer.
    pub fn write_buffer(&self, buffer_id: u64, offset: usize, data: &[u8]) {
        let mut buffers = self.buffers.write();
        if let Some(buf) = buffers.get_mut(buffer_id as usize) {
            buf.write_bytes(offset, data);
        }
    }

    /// Report a QPU failure.
    pub fn report_qpu_failure(&self, qpu_id: u32) {
        self.fault_tolerance.report_qpu_failure(qpu_id);
    }

    /// Report QPU recovery.
    pub fn report_qpu_recovery(&self, qpu_id: u32) {
        self.fault_tolerance.report_qpu_recovery(qpu_id);
    }

    /// Get the total number of tasks dispatched.
    pub fn total_dispatched(&self) -> u64 {
        self.total_dispatched.load(Ordering::Relaxed)
    }

    /// Get queue lengths per device.
    pub fn queue_lengths(&self) -> HashMap<Device, u64> {
        let mut map = HashMap::new();
        map.insert(Device::Cpu, self.cpu_queue.len());
        for (i, q) in self.gpu_queues.iter().enumerate() {
            map.insert(Device::Gpu(i as u32), q.len());
        }
        for (i, q) in self.qpu_queues.iter().enumerate() {
            map.insert(Device::Qpu(i as u32), q.len());
        }
        map
    }
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_display() {
        assert_eq!(format!("{}", Device::Cpu), "CPU");
        assert_eq!(format!("{}", Device::Gpu(0)), "GPU:0");
        assert_eq!(format!("{}", Device::Qpu(1)), "QPU:1");
    }

    #[test]
    fn test_device_checks() {
        assert!(Device::Cpu.is_cpu());
        assert!(!Device::Cpu.is_gpu());
        assert!(Device::Gpu(0).is_gpu());
        assert!(Device::Qpu(0).is_qpu());
        assert_eq!(Device::Gpu(2).device_index(), Some(2));
        assert_eq!(Device::Cpu.device_index(), None);
    }

    #[test]
    fn test_engine_creation() {
        let engine = SupernovaEngine::new(2, 1);
        let lengths = engine.queue_lengths();
        assert_eq!(lengths[&Device::Cpu], 0);
        assert_eq!(lengths[&Device::Gpu(0)], 0);
        assert_eq!(lengths[&Device::Gpu(1)], 0);
        assert_eq!(lengths[&Device::Qpu(0)], 0);
    }

    #[test]
    fn test_dispatch_to_cpu() {
        let engine = SupernovaEngine::new(1, 1);
        let task = Task::new(1, Priority::Normal, 100, 1024);
        let device = engine.dispatch(task);
        assert_eq!(device, Device::Cpu);
        assert_eq!(engine.total_dispatched(), 1);
    }

    #[test]
    fn test_dispatch_preferred_device() {
        let engine = SupernovaEngine::new(1, 1);
        let task = Task::new(1, Priority::Normal, 100, 1024).with_device(Device::Gpu(0));
        let device = engine.dispatch(task);
        assert_eq!(device, Device::Gpu(0));
    }

    #[test]
    fn test_dispatch_large_memory_to_gpu() {
        let engine = SupernovaEngine::new(2, 0);
        let task = Task::new(1, Priority::Normal, 100, 200 * 1024 * 1024); // 200MB
        let device = engine.dispatch(task);
        assert_eq!(device, Device::Gpu(0));
    }

    #[test]
    fn test_dispatch_high_compute_to_gpu() {
        let engine = SupernovaEngine::new(1, 0);
        let task = Task::new(1, Priority::Normal, 5_000_000_000, 1024); // 5B ops
        let device = engine.dispatch(task);
        assert_eq!(device, Device::Gpu(0));
    }

    #[test]
    fn test_dispatch_critical_always_cpu() {
        let engine = SupernovaEngine::new(2, 2);
        let task = Task::new(1, Priority::Critical, 10_000_000_000, 1024 * 1024 * 1024);
        let device = engine.dispatch(task);
        assert_eq!(device, Device::Cpu);
    }

    #[test]
    fn test_dispatch_gpu_out_of_range_fallback() {
        let engine = SupernovaEngine::new(1, 0);
        let task = Task::new(1, Priority::Normal, 100, 1024).with_device(Device::Gpu(5));
        let device = engine.dispatch(task);
        assert_eq!(device, Device::Cpu); // Falls back to CPU
    }

    #[test]
    fn test_work_stealing() {
        let engine = SupernovaEngine::new(1, 0);
        // Dispatch a task to GPU
        let task = Task::new(1, Priority::Normal, 100, 200 * 1024 * 1024);
        engine.dispatch(task);

        // Steal from GPU
        let stolen = engine.steal_task(Device::Gpu(0));
        assert!(stolen.is_some());
        assert_eq!(stolen.unwrap().id, 1);
    }

    #[test]
    fn test_steal_any() {
        let engine = SupernovaEngine::new(1, 0);
        let task = Task::new(42, Priority::Normal, 5_000_000_000, 1024);
        engine.dispatch(task);

        let result = engine.steal_any(Device::Cpu);
        assert!(result.is_some());
        let (task, from) = result.unwrap();
        assert_eq!(task.id, 42);
        assert_eq!(from, Device::Gpu(0));
    }

    #[test]
    fn test_shared_buffer() {
        let mut buf = SharedBuffer::new_cpu(0, 256);
        assert_eq!(buf.size(), 256);
        assert_eq!(buf.id(), 0);

        // Write data
        buf.write_bytes(0, b"hello world");
        let data = buf.read_bytes(0, 11);
        assert_eq!(&data, b"hello world");

        // Attach devices
        buf.attach_device(Device::Gpu(0));
        assert!(buf.is_shared());
        buf.detach_device(Device::Cpu);
        assert!(!buf.is_shared());
    }

    #[test]
    fn test_buffer_out_of_bounds_panics() {
        let buf = SharedBuffer::new_cpu(0, 16);
        let result = std::panic::catch_unwind(|| {
            buf.read_bytes(0, 32);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_alloc_buffer() {
        let engine = SupernovaEngine::new(0, 0);
        let id = engine.alloc_buffer(1024);
        assert_eq!(id, 0);

        engine.write_buffer(id, 0, b"test data");
        let data = engine.read_buffer(id, 0, 9);
        assert_eq!(&data, b"test data");
    }

    #[test]
    fn test_engine_attach_buffer() {
        let engine = SupernovaEngine::new(1, 0);
        let id = engine.alloc_buffer(64);
        engine.attach_buffer(id, Device::Gpu(0));

        let buffers = engine.buffers.read();
        assert!(buffers[id as usize].is_shared());
    }

    #[test]
    fn test_fault_tolerance() {
        let ft = FaultTolerance::new();
        assert!(ft.fallback_enabled());
        assert_eq!(ft.failure_count(), 0);

        ft.report_qpu_failure(0);
        assert_eq!(ft.failure_count(), 1);
        assert!(!ft.qpu_available(0));

        // Should fall back to CPU
        let device = ft.select_device(Device::Qpu(0));
        assert_eq!(device, Device::Cpu);

        // Recover
        ft.report_qpu_recovery(0);
        assert!(ft.qpu_available(0));
        let device = ft.select_device(Device::Qpu(0));
        assert_eq!(device, Device::Qpu(0));
    }

    #[test]
    fn test_qpu_fallback_on_dispatch() {
        let engine = SupernovaEngine::new(0, 1);
        engine.report_qpu_failure(0);

        let task = Task::new(1, Priority::Normal, 100, 1024).with_device(Device::Qpu(0));
        let device = engine.dispatch(task);
        assert_eq!(device, Device::Cpu); // Fell back from QPU
    }

    #[test]
    fn test_cpu_only_engine() {
        let engine = SupernovaEngine::cpu_only();
        let task = Task::new(1, Priority::Normal, 10_000_000_000, 1024 * 1024 * 1024);
        let device = engine.dispatch(task);
        assert_eq!(device, Device::Cpu); // No GPU available
    }

    #[test]
    fn test_multiple_dispatches() {
        let engine = SupernovaEngine::new(1, 0);
        for i in 0..100 {
            let task = Task::new(i, Priority::Normal, 10, 64);
            engine.dispatch(task);
        }
        assert_eq!(engine.total_dispatched(), 100);
        assert_eq!(engine.cpu_queue.len(), 100);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }
}
