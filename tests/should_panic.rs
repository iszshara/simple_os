//! # should_panic.rs
//!
//! Dieses Modul testet das **Fehlerverhalten** des Kernels,
//! indem absichtlich eine panic! ausgelöst wird.
//!
//! Der Test wird in QEMU ausgeführt und validiert,
//! ob der Panic-Handler korrekt reagiert und QEMU
//! mit dem richtigen Exit-Code beendet.
//!
//! ## Übersicht
//!
//! - Kein std und kein main, da Bare-Metal-Umgebung  
//! - Nutzt QemuExitCode, um den Teststatus an QEMU zu übermitteln  
//! - Prüft, ob der Panic Handler wie erwartet greift
//!
//! ## Enthaltene Komponenten
//!
//! - [`_start()`]: Einstiegspunkt, der die Testfunktion [`should_fail()`] ausführt  
//! - [`panic()`]: Panic Handler, der einen erfolgreichen Test markiert  
//! - [`should_fail()`]: Funktion, die absichtlich fehlschlägt
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use simple_os::{QemuExitCode, exit_qemu, serial_print, serial_println};



/// ## Einstiegspunkt (_start)
///
/// Diese Funktion startet den Testlauf und ruft [should_fail()] auf,
/// um eine absichtliche Panic auszulösen.
///
/// Wenn die Panic **nicht** auftritt, gilt der Test als fehlgeschlagen.
/// In diesem Fall wird eine Fehlermeldung über die serielle Schnittstelle
/// ausgegeben und QEMU mit [QemuExitCode::Failed] beendet.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !
{
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);

    loop{}
}

/// ## Panic Handler
///
/// Wird ausgelöst, wenn während des Tests eine Panic auftritt.
/// In diesem Fall wird [ok] ausgegeben und QEMU mit
/// [QemuExitCode::Success] beendet, um einen **erfolgreichen Test**
/// zu signalisieren.
#[panic_handler]
fn panic(_info: &PanicInfo) -> !
{
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop{}
}

/// ## Test: should_fail
///
/// Führt absichtlich einen fehlschlagenden Test aus, um den Panic Handler
/// zu überprüfen.
///
/// Gibt über die serielle Schnittstelle aus:
///
/// ```text
/// should_panic::should_fail...    [ok]
/// ```
///
/// Erwartetes Verhalten:
/// - Der `assert_eq!` schlägt fehl → löst Panic aus  
/// - Der Panic Handler wird aufgerufen → Test gilt als bestanden
fn should_fail()
{
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}