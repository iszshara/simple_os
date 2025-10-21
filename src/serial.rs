//! # Modul: serial
//! 
//! Dieses Modul stellt die serielle Schnittstelle bereit, um Ausgaben vom Kernel
//! (z. B. Logmeldungen oder Testergebnisse) an den Host zu senden.
//!
//! Es implementiert eine einfache, thread-sichere [SerialPort]-Instanz, die über
//! [serial_print!] und [serial_println!] angesprochen werden kann.
//!
//! # Aufbau
//! 
//! | Komponente | Aufgabe |
//! |-------------|----------|
//! | [SERIAL1] | Globale, mutexgeschützte Instanz des UART-Ports |
//! | [serial_print!] / [serial_println!] | Eigene Makros zum Schreiben über die serielle Schnittstelle |
//!
//! # Hintergrund
//! 
//! Da in einem Bare-Metal-Umfeld keine Standardbibliothek (std) zur Verfügung steht,
//! können normale Print-Makros (println!, eprintln!, etc.) nicht verwendet werden.
//! Stattdessen werden die Ausgaben direkt an die UART-Schnittstelle (0x3F8) gesendet,
//! welche typischerweise als **COM1** genutzt wird.
//!
//! # Beispiel
//! ```rust,no_run
//! use simple_os::serial_println;
//! 
//! serial_println!("Hello from the kernel!");
//! ```

use core::fmt::Write;

use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static!
{
    /// ### Globale serielle Schnittstelle
    ///
    /// Initialisiert den ersten UART-Port (0x3F8, typischerweise **COM1**)
    /// und schützt ihn durch einen [Mutex], um konkurrierende Zugriffe zu verhindern.
    ///
    /// Der Port wird **lazy** initialisiert, d. h. erst beim ersten Zugriff während
    /// der Laufzeit, was Ressourcen spart und Initialisierungsprobleme vermeidet.
    pub static ref SERIAL1: Mutex<SerialPort> = 
    {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
/// Interne Hilfsfunktion, die Formatierungsargumente (fmt::Arguments) an den
/// globalen [SERIAL1]-Port weiterleitet.
///
/// Sollte **nicht direkt** verwendet werden – stattdessen die Makros
/// [serial_print!] oder [serial_println!] nutzen.
pub fn _print(args: ::core::fmt::Arguments)
{
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

/// ### serial_print!
///
/// Gibt formatierten Text über die serielle Schnittstelle aus.
///
/// Funktioniert ähnlich wie das Standard-Makro [print!], verwendet jedoch
/// den UART-Port statt der Standardausgabe.
///
/// # Beispiel
/// ```rust,no_run
/// use simple_os::serial_print;
///
/// serial_print!("Wert: {}", 42);
/// ```
#[macro_export]
macro_rules! serial_print 
{
    ($($arg:tt)*) => 
    {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// ### serial_println!
///
/// Gibt eine formatierte Zeile über die serielle Schnittstelle aus.
///
/// Entspricht funktional dem Standard-Makro [println!], ist aber für
/// Bare-Metal-Umgebungen implementiert.
///
/// Unterstützt drei Varianten:
/// 1. Ohne Argumente – gibt nur einen Zeilenumbruch aus  
/// 2. Mit Formatstring  
/// 3. Mit Formatstring und Argumenten
///
/// # Beispiel
/// ```rust,no_run
/// use simple_os::serial_println;
///
/// serial_println!("Hello Kernel!");
/// serial_println!("Wert: {}", 1337);
/// ```
#[macro_export]
macro_rules! serial_println 
{
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}
