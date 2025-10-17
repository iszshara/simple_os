//! vga_buffer.rs
//! 1. Erstellt einen VGA Buffer.
//! 2. Implementiert den WRITER.
//! 3. Stellt das print! und println! macro zur Verfügung.

use spin::Mutex;
use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;

////////////////////////////////////////////////////////////////////////////////
/// 
/// ## Enum
/// 
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)] 
/// -> Nun können Copy Semantics genutzt werden für den Type, sowie das man es 
/// printen und vergleichen kann.
/// 
/// #[repr(u8)]
/// -> dadurch werden alle Enum Variants als u8 gespeichert.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color
{
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

////////////////////////////////////////////////////////////////////////////////
/// 
/// ## ColorCode
/// 
/// Der Color Code wird in einer struct gespeichert da er mehrere Argumente
/// vereint. Weiterhin wird er als u8 gespeichert, da der Vordergrund 4 bit und 
/// der Hintergrund 4 bit groß ist und so zusammengesetzt wird, wie man in dem
/// impl-Block sehen kann.
/// 
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode 
{
    fn new(foreground: Color, background: Color) -> ColorCode
    {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

////////////////////////////////////////////////////////////////////////////////
/// 
/// ## ScreenChar
/// 
/// In der struct ScreenChar werden die Fields ascii_character und color_code
/// initialisiert, da ein Char, der auf den Bildschirm geprintet wird, aus einem 
/// Ascii Buchstaben besteht und dem dazugehörigen Color Code, um den 
/// Hintergrund und Vordergrund darstellen zu können.
/// 
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar
{
    ascii_character: u8,
    color_code: ColorCode,
}

////////////////////////////////////////////////////////////////////////////////
/// 
/// ## Buffer
/// 
/// Der Text wird in den Buffer geschrieben, aber in zukünftigen Rust Versionen
/// könnte der Rust Compiler aggressiver optimieren und den Schreibprozess 
/// auslassen da nur einmal in den Buffer geschrieben wird. Um das zu verhindern
/// wird Volatile genutzt. Volatile zeigt dem Compiler an, das der Schreibvorgang
/// Nebeneffekte haben kann und nicht wegoptimiert werden soll.
/// 
////////////////////////////////////////////////////////////////////////////////
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer
{
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

////////////////////////////////////////////////////////////////////////////////
/// 
/// ## Writer
/// 
/// Das Writer struct vereint nun wieder mit den Fields column_position,
/// color_code und buffer alle notwendigen Argumente die man braucht, um einen
/// Buchstaben auf den Bildschirm zu schreiben.
/// 
////////////////////////////////////////////////////////////////////////////////
pub struct Writer
{
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer 
{
    ////////////////////////////////////////////////////////////////////////////////
    /// 
    /// ## impl Writer - write_byte()
    /// 
    /// Schreibt Byte für Byte (Character für Character) in den Buffer hinein
    /// und trackt die ganze Zeit die aktuelle Position, sowie den vorgegebenen
    /// Abstand.
    /// 
    ////////////////////////////////////////////////////////////////////////////////
    pub fn write_byte(&mut self, byte: u8)
    {
        match byte
        {
            b'\n' => self.new_line(),   //wenn das byte das newline byte ist wird new_line() gerufen
            byte =>                 //in diesem match arm werden dann die bytes ausgegeben
            {
                if self.column_position >= BUFFER_WIDTH //checkt ob die Zeile voll ist, wenn ja wird new_line() gerufen
                {
                    self.new_line()
                }
                let row = BUFFER_HEIGHT -1;             //um zu wissen in welcher row man sich gerade befindet zum tracken
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar        //schreibt einen neuen Char in den Buffer an der aktuellen Position
                { 
                    ascii_character: byte, color_code 
                });
                self.column_position += 1;
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    /// 
    /// ## impl Writer - write_string()
    /// 
    /// Reiht die Bytes aus write_byte() hintereinander zu einem String zusammen und
    /// checkt ob der geschriebene Character ein gültiger Ascii Character ist. Wenn
    /// nicht wird ein weißes Viereck ausgegeben.
    /// 
    ////////////////////////////////////////////////////////////////////////////////
    pub fn write_string(&mut self, s: &str)
    {
        for byte in s.bytes()
        {
            match byte
            {
                //ascii byte oder newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                //nicht in der ausgebbaren ascii range
                _ =>  self.write_byte(0xfe),
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    /// 
    /// ## impl-Writer new_line()
    /// 
    /// Wenn ein Character über die vorgegebene Breite der Zeile hinausgeschrieben
    /// werden würde, springt die Funktion in die nächste Zeile in dem die aktuelle
    /// row minus 1 gerechnet wird.
    /// 
    ////////////////////////////////////////////////////////////////////////////////
    fn new_line(&mut self)
    {
        for row in 1..BUFFER_HEIGHT
        {
            for col in 0..BUFFER_WIDTH
            {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT -1);
        self.column_position = 0;
    }

    ////////////////////////////////////////////////////////////////////////////////
    /// 
    /// ## impl-Writer clear_row()
    /// 
    /// Setzt den Wert der Spalte auf 0 und somit auf den Anfang zurück und leert
    /// die aktuelle Zeile.
    /// 
    ////////////////////////////////////////////////////////////////////////////////
    fn clear_row(&mut self, row: usize)
    {
        let blank = ScreenChar
        {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH
        {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// 
/// ## impl fmt::Write for Writer
/// 
/// Formatiert den zusammengesetzten String aus write_string() in die 
/// standardisierte UTF-8 Encodierung. 
///  
////////////////////////////////////////////////////////////////////////////////
impl fmt::Write for Writer
{
    fn write_str(&mut self, s: &str) -> fmt::Result
    {
        self.write_string(s);
        Ok(())
    }
}

lazy_static!
{
    ////////////////////////////////////////////////////////////////////////////////
    /// 
    /// ## WRITER
    /// 
    /// pub static ref bedeutet das eine statische Referenz(&'static T) erzeugt wird
    /// , die beim ersten Zugriff initialisiert wird (durch lazy) und dann für 
    /// den Rest des Programms unveränderlich bleibt. Mutex stellt währenddessen
    /// sicher das immer nur ein Prozess auf WRITER zugreifen kann und wenn dieser 
    /// fertig ist der lock() gelöst wird und andere Prozesse wieder auf WRITER 
    /// zugreifen können.
    /// 
    ////////////////////////////////////////////////////////////////////////////////
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer
    {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }, //what the helly🤨 => mit einem raw pointer auf die Speicheradresse für VGA zeigen (es ist sicher das es dort liegt)
    });
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
/// ### Erklärung zu print!
/// 
/// print -> printed Character auf die aktuelle Zeile   
/// println -> dasselbe wie print + neue Zeile
/// 
////////////////////////////////////////////////////////////////////////////////
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

////////////////////////////////////////////////////////////////////////////////
/// 
/// ## Tests   
/// 
/// ### test_println_simple()   
/// -> testet ob println funktioniert und nicht panicked
/// 
/// ### test_println_many()   
/// -> testet das Schreiben vieler Zeilen und checkt ob 
/// der vga buffer panicked wenn die Zeilen außerhalb des Bildschirmes 
/// geshifted werden
/// 
/// ### test_println_output()   
/// -> Testet ob der string wirklich geprinted wird auf dem Bildschirm. In der 
/// for-Schleife wird die Anzahl der Iterationen der Variable 'i' gezählt, 
/// mittels enumerate und dann mittels assert_eq! abgeglichen ob dieselbe 
/// Anzahl an Chars auf dem Bildschirm geprinted werden.
/// 
////////////////////////////////////////////////////////////////////////////////
#[test_case]
fn test_println_simple()
{
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many()
{
    for _ in 0..200
    {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output()
{
    let string = "Some test string that fits on a single line";
    println!("{}", string);
    for (i, c) in string.chars().enumerate()
    {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}