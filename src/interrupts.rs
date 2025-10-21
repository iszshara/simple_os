//! Modul interrupts
//! 
//! Dieses Modul enthält die Intialisierung und Definition der
//! **Interrupt Descriptor Table (IDT)**, die von der CPU verwendet wird,
//! um Interrupts und Exceptions den passenden Handler-Funktionen zuzuordnen
//! 
//! Enthält Handler für:
//! - Breakpoints
//! - Double Faults (mit separatem Stack aus dem TSS)

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use lazy_static::lazy_static;
use crate::gdt;

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