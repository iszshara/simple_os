//! # Modul: vga_buffer
//!
//! Dieses Modul implementiert eine einfache Textausgabe über den
//! VGA-Textmodus (Speicheradresse 0xb8000).
//!
//! Es stellt den [WRITER] sowie die Makros [print!] und [println!] bereit,
//! um Text direkt auf den Bildschirm zu schreiben – **ohne Standardbibliothek**.
//!
//! # Aufbau
//!
//! | Komponente | Beschreibung |
//! |-------------|--------------|
//! | [Color] | Enthält alle 16 VGA-Farben |
//! | [Writer] | Schreibt Text in den VGA-Puffer |
//! | [WRITER] | Globale, mutexgeschützte Writer-Instanz |
//! | [print!], [println!] | Makros für formatierte Textausgabe |
//!
//! # Hintergrund
//!
//! Der VGA-Textmodus speichert Zeichen und Farbwerte in einem 80×25-Grid im Speicher
//! (0xb8000). Jedes Zeichen besteht aus einem ASCII-Byte und einem Farbbyte.
//!
//! Da Bare-Metal-Umgebungen keine std::io-Funktionen bieten,
//! müssen Ein- und Ausgaben direkt über Speicherzugriffe erfolgen.

use spin::Mutex;
use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;

/// Repräsentiert die 16 verfügbaren VGA-Farben.
///
/// Durch #[repr(u8)] werden die Varianten direkt als u8 gespeichert,
/// was der tatsächlichen Speicherrepräsentation im VGA-Puffer entspricht.
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

/// Kombination aus Vorder- und Hintergrundfarbe.
///
/// Die unteren 4 Bits repräsentieren die Vordergrundfarbe,
/// die oberen 4 Bits die Hintergrundfarbe.
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

/// Repräsentiert ein einzelnes Zeichen im VGA-Puffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar
{
    ascii_character: u8,
    color_code: ColorCode,
}

/// Der VGA-Textpuffer mit BUFFER_HEIGHT Zeilen und BUFFER_WIDTH Spalten.
///
/// Jedes Feld ist als [Volatile] markiert, damit der Compiler keine
/// Speicherzugriffe entfernt, da sie **sichtbare Nebeneffekte** haben.
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer
{
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// ## Writer
/// 
/// Schreibt Zeichen in den VGA-Puffer.
///
/// Der [Writer] hält:
/// - die aktuelle Spaltenposition,
/// - den aktuellen [ColorCode],
/// - eine mutable Referenz auf den [Buffer].
pub struct Writer
{
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer 
{
    /// Schreibt ein einzelnes Byte in den VGA-Puffer.
    ///
    /// - Bei \n wird eine neue Zeile begonnen.
    /// - Wenn die Zeile voll ist, wird automatisch nach unten gescrollt.
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

    /// Schreibt einen String in den VGA-Puffer.
    ///
    /// Nicht druckbare ASCII-Zeichen werden als ■ (0xfe) dargestellt.
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

    /// Scrollt den Puffer um eine Zeile nach oben.
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

    /// Löscht den Inhalt einer bestimmten Zeile.
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


/// Ermöglicht das Verwenden von format_args! mit [Writer].
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
    /// Globale Writer-Instanz.
    ///
    /// Wird **lazy** initialisiert, um Reihenfolgeprobleme während des
    /// Programmstarts zu vermeiden.  
    /// Der [Mutex] stellt sicher, dass immer nur ein Thread gleichzeitig
    /// auf den Writer zugreift.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer
    {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }, //what the helly🤨 => mit einem raw pointer auf die Speicheradresse für VGA zeigen (es ist sicher das es dort liegt)
    });
}

/// Gibt Text auf den VGA-Puffer aus.
///
/// Funktioniert analog zu [print!], schreibt aber direkt auf den Bildschirm.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}


/// Gibt eine Zeile auf den VGA-Puffer aus.
///
/// Funktioniert analog zu [println!], schreibt aber direkt auf den Bildschirm.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
/// Interne Hilfsfunktion zum Schreiben formatierten Textes.
/// without_interrupts nimmt eine Closure
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(||
    {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

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
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let string = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| 
        {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", string).expect("writeln failed");
        for (i, c) in string.chars().enumerate() 
        {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}