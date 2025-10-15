// kein Verwenden von der std-lib weil es OS-spezifische Calls braucht die es nicht gibt auf bare-metal
#![no_std]
// keine main, da sie auf bare-metal nicht den entry point für den Entwickler darstellt
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(simple_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use simple_os::println;

////////////////////////////////////
//
// Entry Point
//
////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !
{
    println!("Hello World {}", "!");

    #[cfg(test)]
    test_main();

    loop{}
}

////////////////////////////////////
//
// Panic Handler
//
////////////////////////////////////
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    println!("{}", info);
    loop{}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    simple_os::test_panic_handler(info)
}

////////////////////////////////////
//
// Exit Qemu
//
////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode
{
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode)
{
    use x86_64::instructions::port::Port;

    unsafe 
    {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

////////////////////////////////////
//
// Test Section
//
////////////////////////////////////
// pub trait Testable
// {
//     fn run(&self) -> ();
// }

// impl<T> Testable for T
// where
//     T: Fn(),
// {
//     fn run(&self)
//     {
//         serial_println!("{}...\t", core::any::type_name::<T>());
//         self();
//         serial_println!("[ok]");
//     }
// }

// #[cfg(test)]
// pub fn test_runner(tests: &[&dyn Testable])
// {
//     serial_println!("Running {} tests", tests.len());
//     for test in tests 
//     {
//         test.run();
//     }
//     exit_qemu(QemuExitCode::Success);
// }

#[test_case]
fn trivial_assertion()
{
    assert_eq!(1, 1);
}