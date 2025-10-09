use spin::Mutex;
use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)] //Nun können Copy Semantics genutzt werden für den Type, sowie das man es printen und comparen kann
#[repr(u8)] //dadurch werden alle Enum Variants als u8 gespeichert
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar
{
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer
{
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer
{
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer 
{
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

impl fmt::Write for Writer
{
    fn write_str(&mut self, s: &str) -> fmt::Result
    {
        self.write_string(s);
        Ok(())
    }
}


lazy_static!                //damit der Writer nicht während der Compile Time berechnet wird, sondern erst wenn es das erste mal aufgerufen wird (also lazy)
{
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer
    {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }, //what the helly🤨 => mit einem raw pointer auf die Speicheradresse für VGA zeigen (es ist sicher das es dort liegt)
    });
}

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
