use core::fmt::Write;

use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static!
{
    pub static ref SERIAL1: Mutex<SerialPort> = 
    {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

////////////////////////////////////
//
// Adding Macros: serial_print(ln)!
//
////////////////////////////////////
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments)
{
    //use core::fmt::Arguments;
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

// Printed auf den Host über das Serial Interface
#[macro_export]
macro_rules! serial_print 
{
    ($($arg:tt)*) => 
    {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

// Printed auf den Host über das Serial Interface und fügt eine neue Zeile an
// 1. Arm: keine Argumente übergeben -> Printed eine neue Zeile
// 2. Arm: nur einen format String übergeben -> Printed String + neue Zeile
// 3. Arm: wenn zusätzliche Formatierungsargumente übergeben werden -> Printed String mit den Formatierungargs. + neue Zeile
#[macro_export]
macro_rules! serial_println 
{
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}
