//! # Modul gdt
//!
//! Dieses Modul implementiert die **Global Descriptor Table (GDT)** für den Kernel.
//!
//! Die GDT ist eine zentrale Struktur im x86_64-System, die Speichersegmente verwaltet.
//! Sie definiert unter anderem:
//! - Das **Kernel-Code-Segment**
//! - Das **Task State Segment (TSS)** für Interrupt-Stack-Handling
//!
//! ## Übersicht
//!
//! - **Globale Initialisierung:** Die GDT wird einmalig über [`lazy_static`] erstellt.
//! - **Segmente:**  
//!   - Kernel-Code-Segment → für die CPU-Ausführung des Kernelcodes  
//!   - TSS-Segment → ermöglicht eigene Stacks für bestimmte Interrupts (z. B. Double Fault)
//! - **Sicherheit:**  
//!   - Das Laden von Segmenten (CS, TSS) ist `unsafe`, da ein falscher Wert zu einem **Triple Fault** führen kann
//!   - Der TSS-Stack wird als `static mut` definiert, um global verfügbar zu sein
//!
//! ## Enthaltene Komponenten
//!
//! - [`GDT`]: statische Referenz auf die GDT und die Selektoren  
//! - [`Selectors`]: enthält die Code- und TSS-Selektoren  
//! - [`init()`]: Initialisiert die GDT und lädt die Segmente in die CPU
//! - [`TSS`]: Task State Segment, das die Interrupt-Stacks enthält

use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use x86_64::structures::gdt::SegmentSelector;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;


lazy_static!
{
    /// Initialisiert den globalen [TaskStateSegment].
    ///
    /// Dieser TaskStateSegment definiert den Interrupt-Stack für kritische Ausnahmen,
    /// insbesondere für **Double Faults**.  
    /// 
    /// Dabei wird:
    /// - ein separater Stack-Bereich von 4096 * 5 Bytes reserviert,
    /// - dessen Start- und Endadresse berechnet,
    /// - und der Stack-Endezeiger (stack_end) im entsprechenden
    ///   [interrupt_stack_table]-Eintrag des TSS hinterlegt.
    ///
    /// # Sicherheit
    ///
    /// Der Stack wird als static mut allokiert, da der Speicherbereich global
    /// und dauerhaft verfügbar sein muss.  
    /// Dies ist sicher, solange der Stack **nur durch die CPU** über den
    /// entsprechenden Interrupt benutzt wird.
    ///
    /// # Hintergrund
    ///
    /// Der separate Stack für Double Faults ist notwendig, weil ein Double Fault
    /// häufig durch **einen defekten oder überlaufenen normalen Stack**
    /// verursacht wird.  
    /// Durch die Zuweisung eines unabhängigen Stackbereichs kann das System
    /// auch im Fehlerfall korrekt reagieren.
    ///
    /// [`interrupt_stack_table`]: x86_64::structures::tss::TaskStateSegment::interrupt_stack_table
    static ref TSS: TaskStateSegment =
    {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] =
        {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static!
{
    /// Globale Instanz der **Global Descriptor Table (GDT)**.
    ///
    /// Die GDT ist eine zentrale Struktur der x86_64-Architektur.
    /// Sie beschreibt verschiedene **Segment-Deskriptoren**, also
    /// Speicherbereiche, die vom Prozessor zur Adressberechnung
    /// oder für privilegierte Operationen verwendet werden.
    ///
    /// In modernen 64-Bit-Systemen spielt die Segmentierung
    /// nur noch eine geringe Rolle, wird aber weiterhin für
    /// bestimmte Systemstrukturen wie das **Task State Segment (TSS)**
    /// benötigt.
    ///
    /// Diese Definition:
    /// - erstellt eine neue [GlobalDescriptorTable],
    /// - fügt einen **Kernel Code Segment Descriptor** hinzu,
    /// - fügt einen **TSS Descriptor** hinzu, der auf das globale [TSS] verweist.
    ///
    /// # Sicherheit
    ///
    /// Die GDT wird global und dauerhaft im Speicher gehalten.
    /// Da sie einmalig initialisiert und anschließend nur gelesen wird,
    /// ist der Einsatz von lazy_static! sicher.
    ///
    /// [GlobalDescriptorTable]: x86_64::structures::gdt::GlobalDescriptorTable
    /// [TSS]: crate::tss::TSS
    static ref GDT: (GlobalDescriptorTable, Selectors) =
    {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors {code_selector, tss_selector})
    };
}

/// Beinhaltet die Segment-Selektoren der GDT.
///
/// Die Selektoren werden verwendet, um die jeweiligen
/// Einträge der Global Descriptor Table zu adressieren.
/// 
/// - [code_selector]: Verweist auf das Kernel-Code-Segment.
/// - [tss_selector]: Verweist auf das Task-State-Segment.
struct Selectors
{
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// Initialisiert und lädt die globale GDT.
///
/// Diese Funktion:
/// 1. lädt die GDT mittels lgdt,
/// 2. aktualisiert das [CS] (Code Segment Register),
/// 3. lädt das Task-State-Segment (TSS) in die CPU.
///
/// # Sicherheit
///
/// Das Laden des Code-Segment-Registers und des TSS ist `unsafe`,
/// da falsche Selektoren oder ungültige Adressen zu einem **Triple Fault**
/// führen können, wodurch das System sofort neu startet.
///
/// Daher darf diese Funktion **nur während der Kernel-Initialisierung**
/// und **nach erfolgreicher Erstellung aller GDT-Einträge** aufgerufen werden.
pub fn init()
{
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};

    GDT.0.load();
    unsafe
    {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}