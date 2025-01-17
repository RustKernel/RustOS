#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)] // to allow x86_interrupt to run in our OS
#![feature(custom_test_frameworks)] // Custom test framework provided by Rust
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use core::panic::PanicInfo;

pub mod allocator;
pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod keyboard;
pub mod shell;
pub mod fs;

pub fn init() {
    // new gdt with our custom tss in it loaded
    gdt::init();
    interrupts::init_idt();
    // init PIC (Programmable Interrupt Controller)
    unsafe {interrupts::PICS.lock().initialize()};
    // change CPU config for CPU to listen to PIC
    x86_64::instructions::interrupts::enable();
}

// function to prevent continuous loop, hlt puts CPU to sleep until next interrupt
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

// Testing code blocks
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t",core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

// Test exit code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init(); // init initiates the IDT when test environment is started
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

// a breakpoint exception testing test case
#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3(); // invoke a breakpoint exception
}