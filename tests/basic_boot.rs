//! # basic_boot.rs
//!
//! Dieses Modul definiert den Einstiegspunkt (_start) für den Kernel im Testmodus.  
//! Es verwendet das in [lib.rs](../lib.rs.html) definierte benutzerdefinierte Test-Framework
//! von simple_os.
//!
//! ## Übersicht
//!
//! - **Kein std:** Da sich das Projekt im Bare-Metal-Modus befindet, steht
//!   die Standardbibliothek (std) nicht zur Verfügung.
//! - **Kein main:** Der Einstiegspunkt wird direkt vom Linker aufgerufen
//!   und ersetzt die main()-Funktion.
//! - **Testmodus:** Diese Datei wird beim Testen genutzt, um sicherzustellen,
//!   dass grundlegende Kernel-Funktionalität (z. B. [println!]) funktioniert.
//!
//! ## Enthaltene Komponenten
//!
//! - [_start()]: Einstiegspunkt des Kernels im Testmodus  
//! - [panic()]: Panic Handler, der auf [simple_os::test_panic_handler] verweist  
//! - [test_println()]: Beispieltest, der die VGA-Ausgabe testet
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(simple_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use simple_os::println;


/// ## Einstiegspunkt (_start)
///
/// Diese Funktion ist der Startpunkt für QEMU im Testmodus.
/// Sie ruft die Testharness-Funktion [test_main()] auf, die alle
/// definierten Tests ausführt.
///
/// Da auf Bare-Metal kein Betriebssystem aktiv ist, bleibt die
/// Funktion anschließend in einer Endlosschleife, um den Kernel
/// am Laufen zu halten.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !
{
    test_main();

    loop{}
}

/// ## Panic Handler
///
/// Dieser Panic Handler wird im Testmodus verwendet.
/// Er leitet alle Panic-Informationen an
/// [simple_os::test_panic_handler] weiter, damit QEMU
/// den Fehlerstatus korrekt auswerten kann.
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    simple_os::test_panic_handler(info)
}


/// ## Test: test_println
///
/// Überprüft, ob das [println!]-Makro korrekt funktioniert.
/// Wenn der Test erfolgreich abgeschlossen wird, meldet das
/// Test-Framework [ok] über den seriellen Port.
///
/// ```text
/// test_println output
/// ```
#[test_case]
fn test_println()
{
    println!("test_println output");
}
