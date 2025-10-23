//! Definiert den Kernel-Einstiegspunkt und den Panic-Handler.
//!
//! Dieses Modul stellt den **Einstiegspunkt** für den Kernel bereit,
//! wie er vom Linker und Bootloader erwartet wird.  
//! Da es sich um ein Bare-Metal-Umfeld handelt, wird auf die
//! Standardbibliothek (std) verzichtet und stattdessen ausschließlich
//! auf die **Kernbibliothek (core)** zurückgegriffen.
//!
//! # Merkmale
//!
//! - #![no_std]: Deaktiviert die Standardbibliothek, da sie Betriebssystem-Funktionen (z. B. Speicherverwaltung, I/O) voraussetzt, die hier nicht verfügbar sind.
//! - #![no_main]: Unterdrückt die Generierung der Standard-main()-Funktion, da der Kernel stattdessen den eigenen Einstiegspunkt [_start] verwendet.
//! - #![feature(custom_test_frameworks)]: Aktiviert das **Custom Test Framework**, das Kernel-Tests ohne std ermöglicht.
//! - #![test_runner(simple_os::test_runner)]: Legt die benutzerdefinierte Testlauf-Funktion fest.
//! - #![reexport_test_harness_main = "test_main"]: Exportiert die generierte Test-Hauptfunktion unter dem Namen test_main.
//!
//! # Hintergrund
//!
//! In Bare-Metal-Umgebungen (z. B. bei Betriebssystem-Kernen) existiert
//! **kein Betriebssystem**, das eine main()-Funktion aufruft oder
//! Panics handhabt.  
//! Deshalb definiert dieses Modul den tatsächlichen Einstiegspunkt _start
//! und eigene Mechanismen für Panic-Handling und Testunterstützung.
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(simple_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use simple_os::println;

/// Einstiegspunkt des Betriebssystems.
///
/// Diese Funktion entspricht dem **Kernel-Einstiegspunkt** (_start),
/// der vom Bootloader nach dem Laden des Kernels aufgerufen wird.
///
/// Sie ersetzt in einem Betriebssystem den üblichen Einstiegspunkt main().
/// Der Rückgabetyp [!] bedeutet, dass diese Funktion **niemals zurückkehren darf**.
/// Wenn _start terminieren würde, käme es zu einem **Systemabsturz (Triple Fault)**.
///
/// Innerhalb dieser Funktion wird:
/// - eine Begrüßungsnachricht auf die Konsole ausgegeben,
/// - die **Hardware- und Interrupt-Initialisierung** über [simple_os::init()] durchgeführt,
/// - optional (#[cfg(test)]) die **Testsuite** aufgerufen,
/// - und anschließend in eine **Endlosschleife** übergegangen.
///
/// # ABI
///
/// Es wird das **C-ABI** (extern "C") verwendet.  
/// Dieses garantiert ein wohldefiniertes, plattformkonstantes Speicherlayout,  
/// sodass der Bootloader die Funktion zuverlässig aufrufen kann.
///
/// # Sicherheit
///
/// Die Funktion ist mit #[no_mangle] gekennzeichnet, damit der Compiler
/// den Symbolnamen _start **nicht verändert**.  
/// Der Bootloader erwartet diesen genauen Funktionsnamen, um den Kernel zu starten.
///
/// # Ablauf
///
/// ```text
/// Bootloader --> _start() --> simple_os::init() --> Endlosschleife
/// ```
///
/// # Beispielausgabe
///
/// ```text
/// Hello World !
/// It did not crash!
/// ```
///
/// [!]: https://doc.rust-lang.org/std/primitive.never.html
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !
{
    println!("Hello World {}", "!");

    simple_os::init();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    simple_os::hlt_loop();
}

/// Panic-Handler für das Betriebssystem.
///
/// Es werden zwei Varianten des Panic-Handlers bereitgestellt, abhängig
/// davon, ob der Code **im Testmodus** oder **im normalen Kernelbetrieb**
/// ausgeführt wird.
///
/// # Varianten
///
/// - **Normalbetrieb (#[cfg(not(test))])**  
///   Gibt die Panic-Nachricht über [println!] auf der Konsole aus  
///   und bleibt anschließend in einer Endlosschleife, um das System
///   im sicheren Zustand zu halten.
///
/// - **Testmodus (#[cfg(test)])**  
///   Ruft den [simple_os::test_panic_handler] auf, der die Panic
///   an das benutzerdefinierte Test-Framework weiterleitet.
///
/// # Parameter
///
/// * info – Enthält Informationen über die Ursache der Panic, z. B.
///   Dateiname, Zeilennummer und die Nachricht.
///
/// # Hintergrund
///
/// Da in einem Bare-Metal-Umfeld kein Betriebssystem existiert, das eine
/// Panic abfangen könnte, muss der Kernel selbst entscheiden, wie er
/// darauf reagiert.  
/// Der Standard-Handler von Rust (std::panic) steht hier nicht zur Verfügung.
///
/// [PanicInfo]: core::panic::PanicInfo
/// [println!]: crate::println
/// [#[panic_handler]]: https://doc.rust-lang.org/reference/attributes/codegen.html#the-panic_handler-attribute
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    println!("{}", info);
    simple_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    simple_os::test_panic_handler(info)
}

/// Führt einen trivialen Test aus.
///
/// Dient als Minimaltest, um sicherzustellen, dass das
/// **Custom Test Framework** korrekt funktioniert.
///
/// # Zweck
///
/// - Verifiziert, dass das Attribut [#[test_case]] funktioniert.  
/// - Stellt sicher, dass das Test-Framework Panics korrekt verarbeitet.
///
/// # Beispiel
///
/// ```
/// // Dieser Test sollte immer erfolgreich sein.
/// trivial_assertion();
/// ```
#[test_case]
fn trivial_assertion()
{
    assert_eq!(1, 1);
}