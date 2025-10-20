//! Testet ob ein Stack Overflow passieren kann

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use simple_os::{serial_print, serial_println, exit_qemu, QemuExitCode};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

////////////////////////////////////////////////////////////////////////////////
///
/// ## Eingangspunkt
/// 
/// Startet die Funktion die einen Stack Overflow herbeiführt.
/// 
////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> !
{
    serial_print!("stack_overflow::stack_overflow..\t");

    simple_os::gdt::init();
    init_test_idt();

    stack_overflow();

    panic!("Execution continued after stack overflow");
}

////////////////////////////////////////////////////////////////////////////////
///
/// ## test_double_fault_handler()
/// 
/// Implementiert einen Double Fault Handler zum Testen, bei dem ein Stack Frame
/// und ein Error Code mitgegeben wird.
/// 
////////////////////////////////////////////////////////////////////////////////
extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> !
{
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop{}
}

////////////////////////////////////////////////////////////////////////////////
///
/// ## stack_overflow()
/// 
/// Funktion die einen Stack Overflow herbeiführt.
/// Die Flag '#[allow(unconditional_recursion)]' muss genutzt werden da der 
/// Kompiler (richtigerweise) das Programm nicht kompiliert.
/// 
/// ### Erklärung 
/// 
/// stack_overflow() -> Die Adresse der nächsten Anweisung nach dem 
/// Funktionsaufruf (die Rücksprungadresse) wird auf den Stack gelegt -> die 
/// Funktion wird erneut ausgeführt. Da die Funktion keine Abbruchbedingung hat
/// ruft sie sich immer wieder selbst auf und legt jedes mal eine weitere
/// Rücksprungadresse auf den Stack, der somit vollläuft.
/// 
////////////////////////////////////////////////////////////////////////////////
#[allow(unconditional_recursion)]
fn stack_overflow()
{
    stack_overflow(); // for each recrusion, the return address is pushed
    volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
}

#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    simple_os::test_panic_handler(info);
}

lazy_static!
{
    static ref TEST_IDT: InterruptDescriptorTable =
    {
        let mut idt = InterruptDescriptorTable::new();
        unsafe
        {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(simple_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

////////////////////////////////////////////////////////////////////////////////
///
/// ## init_test_idt()
/// 
/// Lädt eine Interrupt Descriptor Table zum Testen.
/// 
////////////////////////////////////////////////////////////////////////////////
pub fn init_test_idt()
{
    TEST_IDT.load();
}