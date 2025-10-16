#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(simple_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use simple_os::println;

////////////////////////////////////////////////////////////////////////////////
/// 
/// ## Eingangspunkt
/// 
/// Eigener Eingangspunkt und Panic Handler , da basic_boot.rs nicht abhängig 
/// ist von main.rs.
/// 
////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !
{
    test_main();

    loop{}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    simple_os::test_panic_handler(info)
}

////////////////////////////////////////////////////////////////////////////////
/// 
/// Tests
/// 
/// Testet ob der Boot Vorgang erfolgreich ist.
///  
////////////////////////////////////////////////////////////////////////////////
#[test_case]
fn test_println()
{
    println!("test_println output");
}
