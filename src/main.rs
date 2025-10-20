//! Stellt den Eingangspunkt bereit für den Linker, sowie einen Panic Handler
//! kein Verwenden von der std-lib weil es OS-spezifische Calls braucht die es nicht gibt auf bare-metal
//! keine main, da sie auf bare-metal nicht den entry point für den Entwickler darstellt
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
/// pub extern "C" fn _start() stellt den Eingangspunkt des ganzen Programmes
/// dar. Sie gibt ein -> ! zurück was bedeutet, dass diese Funktion niemals terminieren darf, da sonst das 
/// Betriebssystem abstürzt. In _start() wird weiter hin die Hardware-Initialisierung durchgeführt.
/// 
////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !
{
    println!("Hello World {}", "!");

    simple_os::init();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop{}
}

////////////////////////////////////////////////////////////////////////////////
///
/// ## Panic Handler
///
/// Es werden zwei verschiedene Panic Handler bereitgestellt, die jeweils genutzt
/// werden wenn entweder der Code getestet wird oder normal ausgeführt wird.
/// 
////////////////////////////////////////////////////////////////////////////////
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

////////////////////////////////////////////////////////////////////////////////
///
/// Test Sektion
///
/// Testet ob Tests funktionieren.
/// 
////////////////////////////////////////////////////////////////////////////////
#[test_case]
fn trivial_assertion()
{
    assert_eq!(1, 1);
}