//! Modul interrupts
//! 
//! Dieses Modul enthält die Intialisierung und Definition der
//! **Interrupt Descriptor Table (IDT)**, die von der CPU verwendet wird,
//! um Interrupts und Exceptions den passenden Handler-Funktionen zuzuordnen
//! 
//! Enthält Handler für:
//! - Breakpoints
//! - Double Faults (mit separatem Stack aus dem TSS)

use x86_64::{structures::idt::{InterruptDescriptorTable, InterruptStackFrame}};
use crate::{print, println};
use lazy_static::lazy_static;
use crate::gdt;
use spin;
use pic8259::ChainedPics;
use x86_64::structures::idt::PageFaultErrorCode;
use crate::hlt_loop;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// # Interrupt Index
/// 
/// Dieses Enum speichert die Offsets für die verschiedenen Eingänge
/// die ein Interrupt bei einem PIC haben kann.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex
{
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex
{
    fn as_u8(self) -> u8
    {
        self as u8
    }

    fn as_usize(self) -> usize
    {
        usize::from(self.as_u8())
    }
}

lazy_static! 
{
    /// Globale Instanz der Interrupt Descriptor Table.
    /// 
    /// Wird mithilfe von [lazy_static!] erstellt, damit sie zur Laufzeit
    /// initialisiert werden kann, ohne unsafe Code im globalen Kontext.
    /// 
    /// Die IDT enthält aktuell Einträge für:
    /// - Breakpoint Exceptions (int3)
    /// - Double Faults (mit dedizierten Stack aus der GDT)
    /// 
    /// [`lazy_static!`]: https://docs.rs/lazy_static/latest/lazy_static/
    static ref IDT: InterruptDescriptorTable = 
    {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe
        {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        
        idt
    };
}

/// Lädt die Interrupt Descriptor Table in die CPU.
/// 
/// Diese Funktion initialisiert die Interruptverwaltung, indem sie die
/// globale IDT mit dem Befehl '*lidt*' lädt.
/// 
/// Nach dem Aufruf sind alle gesetzten Interrupt Handler aktiv
pub fn init_idt() 
{
    IDT.load();
}

/// Handler für Breakpoint Exceptions (int3)
/// 
/// Wird aufgerufen, wenn ein Software-Breakpoint ausgelöst wird.
/// Gibt den aktuellen Stack-Frame über println aus, um zu prüfen,
/// ober der Breakpoint korrekt funktioniert.
/// 
/// Wird für Tests oder Debugging genutzt.
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// # Handler für Double Fault Exceptions.
/// 
/// Diese Funktion löst ein panic! aus, da ein Double Fault meist
/// auf einen schweren Systemfehler hinweist, wie z. B. einen Stack
/// Overflow.
/// 
/// # Sicherheit
/// 
/// Der Handler verwendet einen separaten Stack, der im TSS definiert ist, da
/// nachdem eine CPU Ausnahme passiert das System auf den separaten Stack wechselt.
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64
) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

/// # Handler für Timer Interrupts
/// 
/// Die `notify_end_of_interrupt()`-Funktion bestimmt ob er erste oder zweite PIC
/// einen Interrupt gesendet hat und benutzt dann die `command` und `data` Ports
/// um ein `EOI`(End of Interrupt)-Signal zu senden zum jeweiligen Controller.  
/// Wenn der zweite PIC einen Interrupt sendet müssen beide PICs benachrichtigt werden,
/// da dieser mit dem ersten auf der Input Line verbunden ist.
/// 
/// # Sicherheit
/// 
/// Die Funktion ist `unsafe`, weil wenn die falsche Interrupt Vector Nummer verwendet,
/// kann es passieren das wichtige noch ungesendete Interrupts verloren gehen oder sich
/// das System aufhängt.
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame)
{
    print!(".");

    unsafe
    {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

/// # Handler für Keyboard Interrupts
/// 
/// Erstellt ein Keyboard Objekt, welches von einem `Mutex` gesichert wird.
/// 
/// Initialisiert:
/// - US Keyboard
/// - Scancode Set 1
/// - HandleControl als ignoriert
/// - Port 0x60 als I/O
/// 
/// Wenn diese Funktion aktiv ist wird sie mit einem `lock()` versehen, um weitere Interrupts
/// zu verhindern, bis der Keyboard Interrupt abgearbeitet ist. 
/// Der Port 0x60 wird genutzt, da er der I/O - Port des PS/2 Controllers ist.
/// 
/// Bei jedem Keyboard Interrupt wird der Handler aufgerufen und für die Dauer mit dem Mutex gesichert.
/// Dann wird der scancode vom Keyboard Controller gelesen und der Wert als Parameter in die `add_byte()`
/// Methode eingefügt, welche den Scancode in ein Option<KeyEvent> "übersetzt". Das `KeyEvent` stellt fest
/// welcher Key gedrückt wurde und ob es ein Drücken oder Loslassen war. Um dieses KeyEvent
/// zu interpretieren, wird es an die `process_keyevent()` Methode weitergegeben, welche das KeyEvent
/// in einen Character umändert wenn möglich
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame)
{
    // use x86_64::instructions::interrupts;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static!
    {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
            Mutex::new(Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore)
            );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) 
    {
        if let Some(key) = keyboard.process_keyevent(key_event)
        {
            match key
            {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }    
    }

    unsafe
    {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode)
{
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("ACCESSED ADDRESS: {:?}", Cr2::read());
    println!("ERROR CODE: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

/// # Offset für die PICs
/// 
/// [ChainedPics] repräsentiert das PIC-Layout.
/// Die Offset Range  für die PICs wird festgelegt auf 32-47, um keine Überlappungen
/// zu erzeugen mit anderen Interrupts.
/// 
/// # Sicherheit
/// 
/// Durch das Mutex auf die `Chained Pics` Struktur, ist es möglich einen sicheren 
/// veränderbaren Zugriff über die `lock` Methode zu bekommen.
/// 
/// Die `ChainedPics::new()` Methode ist unsafe, da durch ein falsch gesetztes Offset 
/// undefiniertes Verhalten verursacht werden kann.
pub static PICS: spin::Mutex<ChainedPics> = 
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });


/// # Breakpoint Test
/// 
/// Testet ob das Setzen des Breakpoint erfolgreich ist, indem es den Software
/// Interrupt int3 setzt.
/// Bei einem Erfolg sollte die Nachricht 'EXCEPTION: BREAKPOINT' im Kernel Log stehen
#[test_case]
fn test_breakpoint_exception()
{
    x86_64::instructions::interrupts::int3();
}