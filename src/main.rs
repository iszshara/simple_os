// kein Verwenden von der std-lib weil es OS-spezifische Calls braucht die es nicht gibt auf bare-metal
#![no_std]
// keine main, da sie auf bare-metal nicht den entry point für den Entwickler darstellt
#![no_main]
mod vga_buffer;

use core::panic::PanicInfo;

//static HELLO: &[u8] = b"Hello World!";

// eigener Panic Handler da Stack Unwinding OS-spezifisch ist
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    println!("{}", info);
    loop{}
}

// stellt so gesehen die main function dar
// no_mangle = Rust ist es möglich genau diesen Funktionsnamen auszugeben, da sonst ein unique Funktionsname draus wird, den der Linker dann nicht richtig erkennt
// stellt die entry_point function dar
// "-> !" bedeutet, dass die Funktion niemals erlaubt etwas zurückzugeben, weil der entry point von keiner Funktion aufgerufen wird, sondern direkt von dem OS oder dem Bootloader
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !
{
    // use core::fmt::Write;
    // vga_buffer::WRITER.lock().write_str("Hello again").unwrap();
    // write!(vga_buffer::WRITER.lock(), ", some numbers: {} {}", 42, 3.14).unwrap();
    println!("Hello World {}", "!");

    loop{}
}

