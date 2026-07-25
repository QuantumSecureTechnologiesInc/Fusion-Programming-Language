# Chapter 22: Systems Programming

Fusion's low-level control and memory safety make it ideal for systems programming. This chapter covers OS development, device drivers, embedded systems, and real-time systems.

## OS Development

### Bootloader

```fusion
// Minimal bootloader
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Setup stack
    unsafe {
        asm!(
            "mov rsp, {stack_top}",
            stack_top = const STACK_TOP,
        );
    }
    
    // Clear BSS
    unsafe {
        extern "C" {
            static __bss_start: u8;
            static __bss_end: u8;
        }
        
        let start = &__bss_start as *const u8 as usize;
        let end = &__bss_end as *const u8 as usize;
        
        for addr in start..end {
            unsafe {
                core::ptr::write_volatile(addr as *mut u8, 0);
            }
        }
    }
    
    // Call kernel main
    kernel_main();
    
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

// Kernel entry point
#[no_mangle]
pub extern "C" fn kernel_main() {
    // Initialize VGA buffer
    let vga = unsafe { &mut *(0xB8000 as *mut VgaBuffer) };
    vga.clear();
    vga.write_string("Hello from Fusion OS!");
    
    // Setup GDT, IDT, etc.
    gdt::init();
    idt::init();
    
    // Enable interrupts
    unsafe {
        asm!("sti");
    }
    
    // Start scheduler
    scheduler::init();
}
```

### Memory Management

```fusion
// Physical memory manager
struct PhysicalMemoryManager {
    bitmap: Vec<u64>,
    total_pages: usize,
    used_pages: usize,
}

impl PhysicalMemoryManager {
    fn new(total_memory: usize) -> Self {
        let total_pages = total_memory / PAGE_SIZE;
        let bitmap_size = (total_pages + 63) / 64;
        
        Self {
            bitmap: vec![0; bitmap_size],
            total_pages,
            used_pages: 0,
        }
    }
    
    fn allocate_page(&mut self) -> Option<usize> {
        for (i, entry) in self.bitmap.iter_mut().enumerate() {
            if *entry != u64::MAX {
                let bit = entry.trailing_ones() as usize;
                *entry |= 1 << bit;
                self.used_pages += 1;
                
                return Some(i * 64 + bit);
            }
        }
        
        None
    }
    
    fn free_page(&mut self, page: usize) {
        let index = page / 64;
        let bit = page % 64;
        
        self.bitmap[index] &= !(1 << bit);
        self.used_pages -= 1;
    }
}

// Virtual memory
struct VirtualMemoryManager {
    page_tables: Vec<PageTable>,
    kernel_offset: usize,
}

impl VirtualMemoryManager {
    fn map_page(&mut self, virtual_addr: usize, physical_addr: usize, flags: PageFlags) {
        let pml4_index = (virtual_addr >> 39) & 0x1FF;
        let pml3_index = (virtual_addr >> 30) & 0x1FF;
        let pml2_index = (virtual_addr >> 21) & 0x1FF;
        let pml1_index = (virtual_addr >> 12) & 0x1FF;
        
        // Walk page tables and create as needed
        let pml4 = &mut self.page_tables[0];
        let pml3 = pml4.get_or_create_entry(pml4_index);
        let pml2 = pml3.get_or_create_entry(pml3_index);
        let pml1 = pml2.get_or_create_entry(pml2_index);
        
        pml1.entries[pml1_index] = physical_addr as u64 | flags.bits();
    }
    
    fn translate(&self, virtual_addr: usize) -> Option<usize> {
        let pml4_index = (virtual_addr >> 39) & 0x1FF;
        let pml3_index = (virtual_addr >> 30) & 0x1FF;
        let pml2_index = (virtual_addr >> 21) & 0x1FF;
        let pml1_index = (virtual_addr >> 12) & 0x1FF;
        
        let pml4 = &self.page_tables[0];
        let pml3 = pml4.get_entry(pml4_index)?;
        let pml2 = pml3.get_entry(pml3_index)?;
        let pml1 = pml2.get_entry(pml2_index)?;
        
        let physical = pml1.entries[pml1_index] & 0x000FFFFFFFFFF000;
        let offset = virtual_addr & 0xFFF;
        
        Some(physical as usize + offset)
    }
}
```

### Process Management

```fusion
// Process structure
struct Process {
    pid: u32,
    name: String,
    state: ProcessState,
    page_table: usize,
    kernel_stack: usize,
    user_stack: usize,
    instruction_pointer: usize,
    registers: Registers,
}

enum ProcessState {
    Running,
    Ready,
    Blocked,
    Zombie,
}

struct Registers {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rbp: u64,
    rsp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rflags: u64,
    rip: u64,
}

// Scheduler
struct Scheduler {
    processes: Vec<Process>,
    current_process: Option<usize>,
    ready_queue: VecDeque<usize>,
}

impl Scheduler {
    fn schedule(&mut self) {
        if let Some(current) = self.current_process {
            if self.processes[current].state == ProcessState::Running {
                self.processes[current].state = ProcessState::Ready;
                self.ready_queue.push_back(current);
            }
        }
        
        if let Some(next) = self.ready_queue.pop_front() {
            self.processes[next].state = ProcessState::Running;
            self.current_process = Some(next);
            
            // Context switch
            self.context_switch(&self.processes[next]);
        }
    }
    
    fn context_switch(&self, process: &Process) {
        unsafe {
            // Load page table
            asm!(
                "mov cr3, {pt}",
                pt = in(reg) process.page_table,
            );
            
            // Switch stacks
            asm!(
                "mov rsp, {rsp}",
                rsp = in(reg) process.kernel_stack,
            );
            
            // Jump to process instruction pointer
            asm!(
                "jmp {rip}",
                rip = in(reg) process.instruction_pointer,
            );
        }
    }
}
```

## Device Drivers

### Character Device

```fusion
// UART driver
struct UartDriver {
    port: u16,
}

impl UartDriver {
    fn new(port: u16) -> Self {
        Self { port }
    }
    
    fn init(&self) {
        // Disable interrupts
        self.write_byte(0, 0x00);
        
        // Enable DLAB
        self.write_byte(3, 0x80);
        
        // Set baud rate divisor to 3 (38400 baud)
        self.write_byte(0, 0x03);
        self.write_byte(1, 0x00);
        
        // 8 bits, no parity, one stop bit
        self.write_byte(3, 0x03);
        
        // Enable FIFO
        self.write_byte(2, 0xC7);
        
        // Enable interrupts
        self.write_byte(1, 0x01);
    }
    
    fn write_byte(&self, offset: u16, value: u8) {
        unsafe {
            let port = self.port + offset;
            asm!(
                "outb {port}, {value}",
                port = in(reg) port,
                value = in(reg) value,
            );
        }
    }
    
    fn read_byte(&self, offset: u16) -> u8 {
        unsafe {
            let port = self.port + offset;
            let value: u8;
            asm!(
                "inb {value}, {port}",
                value = out(reg) value,
                port = in(reg) port,
            );
            value
        }
    }
    
    fn send(&self, data: &[u8]) {
        for &byte in data {
            // Wait for transmit buffer empty
            while self.read_byte(5) & 0x20 == 0 {}
            self.write_byte(0, byte);
        }
    }
    
    fn receive(&self) -> Option<u8> {
        if self.read_byte(5) & 0x01 != 0 {
            Some(self.read_byte(0))
        } else {
            None
        }
    }
}

// Implement Write trait for stdio
impl core::fmt::Write for UartDriver {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.send(s.as_bytes());
        Ok(())
    }
}
```

### Block Device

```fusion
// ATA/IDE driver
struct AtaDriver {
    base_port: u16,
    is_slave: bool,
}

impl AtaDriver {
    fn new(base_port: u16, is_slave: bool) -> Self {
        Self { base_port, is_slave }
    }
    
    fn identify(&self) -> Result<DiskInfo, AtaError> {
        // Select drive
        self.write_port(6, if self.is_slave { 0xB0 } else { 0xA0 });
        
        // Send IDENTIFY command
        self.write_port(7, 0xEC);
        
        // Wait for result
        let status = self.read_port(7);
        if status == 0 {
            return Err(AtaError::NoDisk);
        }
        
        // Read 256 words
        let mut info = [0u16; 256];
        for word in &mut info {
            *word = self.read_port_data();
        }
        
        Ok(DiskInfo {
            sectors: (info[60] as u32) << 16 | info[61] as u32,
            model: String::from_utf16_lossy(&info[27..47]),
        })
    }
    
    fn read_sector(&self, lba: u32, buffer: &mut [u8; 512]) -> Result<(), AtaError> {
        // Select drive and set LBA
        self.write_port(6, 0xE0 | ((lba >> 24) as u8) | if self.is_slave { 0x10 } else { 0 });
        self.write_port(2, 1);  // Sector count
        self.write_port(3, lba as u8);
        self.write_port(4, (lba >> 8) as u8);
        self.write_port(5, (lba >> 16) as u8);
        self.write_port(7, 0x20);  // READ SECTORS
        
        // Wait for data
        self.wait_ready()?;
        
        // Read 256 words
        for i in 0..256 {
            let word = self.read_port_data();
            buffer[i * 2] = word as u8;
            buffer[i * 2 + 1] = (word >> 8) as u8;
        }
        
        Ok(())
    }
    
    fn write_sector(&self, lba: u32, buffer: &[u8; 512]) -> Result<(), AtaError> {
        // Select drive and set LBA
        self.write_port(6, 0xE0 | ((lba >> 24) as u8) | if self.is_slave { 0x10 } else { 0 });
        self.write_port(2, 1);  // Sector count
        self.write_port(3, lba as u8);
        self.write_port(4, (lba >> 8) as u8);
        self.write_port(5, (lba >> 16) as u8);
        self.write_port(7, 0x30);  // WRITE SECTORS
        
        // Wait for data
        self.wait_ready()?;
        
        // Write 256 words
        for i in 0..256 {
            let word = buffer[i * 2] as u16 | (buffer[i * 2 + 1] as u16) << 8;
            self.write_port_data(word);
        }
        
        // Flush
        self.write_port(7, 0xE7);
        
        Ok(())
    }
    
    fn read_port(&self, offset: u16) -> u8 {
        unsafe {
            let port = self.base_port + offset;
            let value: u8;
            asm!(
                "inb {value}, {port}",
                value = out(reg) value,
                port = in(reg) port,
            );
            value
        }
    }
    
    fn write_port(&self, offset: u16, value: u8) {
        unsafe {
            let port = self.base_port + offset;
            asm!(
                "outb {port}, {value}",
                port = in(reg) port,
                value = in(reg) value,
            );
        }
    }
    
    fn read_port_data(&self) -> u16 {
        unsafe {
            let port = self.base_port;
            let value: u16;
            asm!(
                "inw {value}, {port}",
                value = out(reg) value,
                port = in(reg) port,
            );
            value
        }
    }
    
    fn write_port_data(&self, value: u16) {
        unsafe {
            let port = self.base_port;
            asm!(
                "outw {port}, {value}",
                port = in(reg) port,
                value = in(reg) value,
            );
        }
    }
    
    fn wait_ready(&self) -> Result<(), AtaError> {
        for _ in 0..1000 {
            let status = self.read_port(7);
            if status & 0x08 != 0 {
                return Ok(());
            }
            if status & 0x01 != 0 {
                return Err(AtaError::DiskError);
            }
        }
        Err(AtaError::Timeout)
    }
}
```

### Network Device

```fusion
// E1000 NIC driver
struct E1000Driver {
    mmio_base: usize,
    tx_ring: Box<[TxDescriptor; 32]>,
    rx_ring: Box<[RxDescriptor; 32]>,
    tx_index: usize,
    rx_index: usize,
}

#[repr(C)]
struct TxDescriptor {
    address: u64,
    length: u16,
    checksum: u16,
    cmd: u8,
    status: u8,
}

#[repr(C)]
struct RxDescriptor {
    address: u64,
    length: u16,
    checksum: u16,
    status: u8,
    errors: u8,
}

impl E1000Driver {
    fn new(mmio_base: usize) -> Self {
        Self {
            mmio_base,
            tx_ring: Box::new(unsafe { core::mem::zeroed() }),
            rx_ring: Box::new(unsafe { core::mem::zeroed() }),
            tx_index: 0,
            rx_index: 0,
        }
    }
    
    fn init(&mut self) {
        // Reset device
        self.write_reg(CTRL, self.read_reg(CTRL) | CTRL_RST);
        
        // Wait for reset
        while self.read_reg(CTRL) & CTRL_RST != 0 {}
        
        // Setup TX/RX rings
        self.init_tx();
        self.init_rx();
        
        // Enable interrupts
        self.write_reg(IMASK, IMS_RXT0 | IMS_TXDW);
        
        // Enable device
        self.write_reg(CTRL, CTRL_SLU | CTRL_FD);
    }
    
    fn send_packet(&mut self, data: &[u8]) {
        let desc = &mut self.tx_ring[self.tx_index];
        desc.address = data.as_ptr() as u64;
        desc.length = data.len() as u16;
        desc.cmd = TX_CMD_EOP | TX_CMD_IFCS;
        desc.status = 0;
        
        // Ring doorbell
        self.write_reg(TDT, (self.tx_index + 1) as u32 % 32);
        
        self.tx_index = (self.tx_index + 1) % 32;
    }
    
    fn receive_packet(&mut self) -> Option<Vec<u8>> {
        let desc = &self.rx_ring[self.rx_index];
        
        if desc.status & RX_STATUS_DD == 0 {
            return None;
        }
        
        let data = unsafe {
            core::slice::from_raw_parts(
                desc.address as *const u8,
                desc.length as usize,
            )
        }.to_vec();
        
        // Reset descriptor
        self.rx_ring[self.rx_index].status = 0;
        
        self.rx_index = (self.rx_index + 1) % 32;
        
        // Update tail pointer
        self.write_reg(RDT, self.rx_index as u32);
        
        Some(data)
    }
    
    fn read_reg(&self, reg: u32) -> u32 {
        unsafe {
            let ptr = (self.mmio_base + reg as usize) as *const u32;
            core::ptr::read_volatile(ptr)
        }
    }
    
    fn write_reg(&self, reg: u32, value: u32) {
        unsafe {
            let ptr = (self.mmio_base + reg as usize) as *mut u32;
            core::ptr::write_volatile(ptr, value);
        }
    }
}
```

## Embedded Systems

### Bare Metal Programming

```fusion
// Embedded target (ARM Cortex-M)
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use stm32f1xx_hal::{prelude::*, pac};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut gpioc = dp.GPIOC.split();
    
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    
    loop {
        led.set_high();
        cortex_m::asm::delay(8_000_000);
        
        led.set_low();
        cortex_m::asm::delay(8_000_000);
    }
}
```

### Interrupt Handling

```fusion
// Interrupt service routines
#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut nvic = dp.NVIC;
    
    // Enable UART interrupt
    unsafe {
        nvic.set_priority(pac::Interrupt::USART1, 1);
        nvic.unmask(pac::Interrupt::USART1);
    }
    
    loop {
        cortex_m::asm::wfi();
    }
}

#[interrupt]
fn USART1() {
    static mut COUNT: u32 = 0;
    
    // Handle interrupt
    unsafe {
        *COUNT += 1;
    }
}
```

### Power Management

```fusion
// Low-power modes
fn enter_sleep_mode() {
    unsafe {
        cortex_m::asm::wfi();  // Wait For Interrupt
    }
}

fn enter_stop_mode() {
    let dp = unsafe { pac::Peripherals::steal() };
    
    // Configure power control
    dp.PWR.cr.modify(|_, w| w.pdds().stop_mode());
    
    // Set SLEEPDEEP bit
    unsafe {
        cortex_m::asm::dsb();
        cortex_m::asm::wfi();
    }
}

fn enter_standby_mode() {
    let dp = unsafe { pac::Peripherals::steal() };
    
    // Configure power control
    dp.PWR.cr.modify(|_, w| w.pdds().standby_mode());
    
    // Set SLEEPDEEP bit
    unsafe {
        cortex_m::asm::dsb();
        cortex_m::asm::wfi();
    }
}
```

## Real-Time Systems

### Real-Time Scheduler

```fusion
// Rate-monotonic scheduler
struct RtmScheduler {
    tasks: Vec<Task>,
    current_time: u64,
}

struct Task {
    id: u32,
    period: u64,
    worst_case_execution_time: u64,
    priority: u32,
    next_deadline: u64,
    state: TaskState,
}

enum TaskState {
    Ready,
    Running,
    Blocked,
}

impl RtmScheduler {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current_time: 0,
        }
    }
    
    fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
        self.tasks.sort_by(|a, b| a.period.cmp(&b.period));
    }
    
    fn schedule(&mut self) -> Option<u32> {
        self.current_time += 1;
        
        // Update deadlines
        for task in &mut self.tasks {
            if self.current_time >= task.next_deadline {
                task.next_deadline += task.period;
                task.state = TaskState::Ready;
            }
        }
        
        // Find highest priority ready task
        for task in &self.tasks {
            if task.state == TaskState::Ready {
                return Some(task.id);
            }
        }
        
        None
    }
    
    fn utilization_bound(&self) -> f64 {
        let total: f64 = self.tasks.iter()
            .map(|t| t.worst_case_execution_time as f64 / t.period as f64)
            .sum();
        
        let n = self.tasks.len() as f64;
        total <= n * (2.0_f64.powf(1.0 / n) - 1.0)
    }
}
```

### Priority Inversion Prevention

```fusion
// Priority inheritance mutex
struct PriorityInheritanceMutex {
    owner: Option<u32>,
    original_priority: Option<u32>,
    waiting_tasks: VecDeque<u32>,
}

impl PriorityInheritanceMutex {
    fn lock(&mut self, task_id: u32, task_priority: u32) -> Result<(), LockError> {
        if self.owner.is_none() {
            self.owner = Some(task_id);
            self.original_priority = Some(task_priority);
            Ok(())
        } else {
            // Priority inheritance
            if task_priority > self.original_priority.unwrap() {
                self.original_priority = Some(task_priority);
            }
            
            self.waiting_tasks.push_back(task_id);
            Err(LockError::WouldBlock)
        }
    }
    
    fn unlock(&mut self, task_id: u32) -> Result<(), UnlockError> {
        if self.owner != Some(task_id) {
            return Err(UnlockError::NotOwner);
        }
        
        if let Some(next_task) = self.waiting_tasks.pop_front() {
            self.owner = Some(next_task);
        } else {
            self.owner = None;
            self.original_priority = None;
        }
        
        Ok(())
    }
}
```

### Time-Triggered Architecture

```fusion
// Time-triggered scheduler
struct TtScheduler {
    tasks: Vec<TtTask>,
    cycle_time: u64,
    current_slot: usize,
}

struct TtTask {
    id: u32,
    slot: usize,
    duration: u64,
    offset: u64,
    state: TaskState,
}

impl TtScheduler {
    fn new(cycle_time: u64, num_slots: usize) -> Self {
        Self {
            tasks: Vec::new(),
            cycle_time,
            current_slot: 0,
        }
    }
    
    fn add_task(&mut self, task: TtTask) {
        self.tasks.push(task);
    }
    
    fn tick(&mut self) -> Option<u32> {
        self.current_slot = (self.current_slot + 1) % (self.cycle_time as usize);
        
        for task in &self.tasks {
            if task.slot == self.current_slot && task.state == TaskState::Ready {
                return Some(task.id);
            }
        }
        
        None
    }
}
```

## Summary

Fusion's systems programming capabilities include:

1. **OS Development**: Bootloaders, memory management, process scheduling
2. **Device Drivers**: Character, block, and network device drivers
3. **Embedded Systems**: Bare metal programming, interrupts, power management
4. **Real-Time Systems**: Deterministic scheduling, priority inheritance, time-triggered architectures

Fusion's memory safety and zero-cost abstractions make it suitable for building reliable systems software without sacrificing performance.

In the next chapter, we'll explore network programming with Fusion.