//! # simple_os
//! 
//! **simple_os** ist ein Übungsprojekt, um die Grundlagen von Betriebssystemen,
//! Bare-Metal-Programmierung und Low-Level-Rust zu verstehen.
//!
//! Dieses Projekt folgt dem Aufbau eines minimalistischen Kernels, basierend auf
//! dem Blog *Writing an OS in Rust* von Philipp Oppermann.  
//! Ziel ist es, Schritt für Schritt die wichtigsten Konzepte zu lernen:
//!
//! - Speicherzugriff und VGA-Ausgabe  
//! - Hardware-Interrupts und CPU-Kontext  
//! - Aufbau der GDT/IDT/TSS-Strukturen  
//! - Fehlerbehandlung über Panic Handler  
//! - Kernel-Tests im Bare-Metal-Umfeld (QEMU)
//!
//! # Aufbau
//! 
//! | Modul | Aufgabe |
//! |--------|----------|
//! | [serial] | Kommunikation über serielle Schnittstelle (z. B. für QEMU-Ausgabe) |
//! | [vga_buffer] | Textausgabe direkt im VGA-Speicher |
//! | [interrupts] | Verwaltung und Behandlung von CPU-Interrupts |
//! | [gdt] | Aufbau der Global Descriptor Table |
//!
//! Weitere Funktionen wie Paging, Speicherverwaltung oder Multitasking
//! können später ergänzt werden.
//!
//! # Testumgebung
//!
//! Das Projekt nutzt das **Custom Test Framework** von Rust,
//! um automatisierte Kernel-Tests ohne Standardbibliothek auszuführen.
//!
//! Tests werden beim Start in QEMU gesammelt und ausgeführt.
//! Ergebnisse werden über den seriellen Port an den Host ausgegeben.
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]


pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;

use core::panic::PanicInfo;
// use crate::interrupts::PIC_1_OFFSET;

/// ### Trait: Testable
///
/// Wird verwendet, um alle Kernel-Tests zu erfassen und in einer einheitlichen
/// Form auszuführen.
///
/// Dieses Trait abstrahiert Testfunktionen, sodass sie einheitlich
/// aufgerufen und über die serielle Schnittstelle ausgegeben werden können.
/// 
/// Führt den Test aus und meldet den Status über den seriellen Port.
pub trait Testable
{
    fn run(&self) -> ();
}


impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self)
    {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}


/// ### Test Runner
///
/// Führt alle Tests aus, die beim Build über das Custom Test Framework
/// registriert wurden.
///
/// Nach erfolgreicher Ausführung wird QEMU mit dem Statuscode Success beendet.
pub fn test_runner(tests: &[&dyn Testable])
{
    serial_println!("Running {} tests", tests.len());
    for test in tests 
    {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

/// ### Test Panic Handler
///
/// Wird aufgerufen, wenn ein Test fehlschlägt.
/// Gibt die Fehlermeldung über den seriellen Port aus und beendet QEMU
/// mit dem Statuscode Failed.
pub fn test_panic_handler(info: &PanicInfo) -> !
{
    serial_println!("[failed!]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}


/// ### Test Entry Point (nur bei #[cfg(test)])
///
/// Definiert den Einstiegspunkt für Testausführungen.
/// Dieser ersetzt den normalen Kernelstart (_start) während Tests.
#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !
{
    init();
    test_main();
    hlt_loop();
}

/// ### Test Panic Handler
///
/// Setzt den Panic Handler während Tests auf [test_panic_handler].
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    test_panic_handler(info)
}

/// ### QEMU Exit Codes
///
/// Stellt Exit-Codes bereit, mit denen QEMU beendet werden kann.
/// Dadurch können Testergebnisse automatisiert ausgewertet werden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode
{
    Success = 0x10,
    Failed = 0x11,
}

/// ### Beendet QEMU mit einem bestimmten Exit Code.
///
/// Nutzt Port 0xF4, um QEMU kontrolliert zu beenden.
/// Dieser Port ist in der Dokumentation von Qemu so vorgegeben
///
/// # Sicherheit
///
/// Der Aufruf greift direkt auf I/O-Ports der CPU zu und ist daher unsafe.
pub fn exit_qemu(exit_code: QemuExitCode)
{
    use x86_64::instructions::port::Port;

    unsafe 
    {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// ## Initialisierung des Kernels
///
/// Führt grundlegende Setup-Schritte aus:
/// - Initialisiert die [Global Descriptor Table](crate::gdt)
/// - Initialisiert die [Interrupt Descriptor Table](crate::interrupts)
/// - Initialisiert die 8259 PIC
/// - Aktiviert Interrupts in der CPU Konfiguration
pub fn init()
{
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> !
{
    loop
    {
        x86_64::instructions::hlt();
    }
}