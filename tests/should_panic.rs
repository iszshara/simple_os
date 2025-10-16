#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use simple_os::{QemuExitCode, exit_qemu, serial_print, serial_println};


////////////////////////////////////////////////////////////////////////////////
/// 
/// ## Eingangspunkt
/// 
/// Eigener Eingangspunkt und Panic Handler , da should_panic.rs nicht abhängig 
/// ist von main.rs.
/// 
////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !
{
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);

    loop{}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> !
{
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop{}
}

////////////////////////////////////////////////////////////////////////////////
/// 
/// Tests
/// 
/// Testet ob panic funktioniert.
/// 
////////////////////////////////////////////////////////////////////////////////
fn should_fail()
{
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}