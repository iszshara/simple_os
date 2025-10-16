//! serial.rs
//! 
//! 1. Stellt das serial_print! macro zur Verfügung
//! 2. Initialisiert einen Port als Serial Port, um auf die serielle Konsole zu schreiben

use core::fmt::Write;

use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static!
{
    ////////////////////////////////////////////////////////////////////////////////
    /// 
    /// ## Serial Port
    /// Hier wird der Serial Port initialisiert, aber lazy.
    /// Das bedeutet, dass der Port erst initialisiert wird, wenn er angesprochen
    /// wird und das dann auch erst zur Runtime.
    /// 
    ////////////////////////////////////////////////////////////////////////////////
    pub static ref SERIAL1: Mutex<SerialPort> = 
    {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

////////////////////////////////////////////////////////////////////////////////
///
/// ## Custom Macro
///
/// Festlegen von eigenen Macros, da man nicht einfach so die normalen nutzen 
/// kann. Das liegt daran das man die Bibliothek std::io nutzen müsste, welche
/// aber 1.) von der std-Bibliothek abhängig ist und 2.) somit von einem 
/// darunterliegenden OS abhängig ist.
/// 
/// ### Erklärung zu serial_print!
/// 
/// Printed auf den Host über das Serial Interface und fügt eine neue Zeile an
/// 1. Arm: keine Argumente übergeben -> Printed eine neue Zeile
/// 2. Arm: nur einen format String übergeben -> Printed String + neue Zeile
/// 3. Arm: wenn zusätzliche Formatierungsargumente übergeben werden 
///         -> Printed String mit den Formatierungargs. + neue Zeile
/// 
////////////////////////////////////////////////////////////////////////////////
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments)
{
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

#[macro_export]
macro_rules! serial_print 
{
    ($($arg:tt)*) => 
    {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println 
{
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}
