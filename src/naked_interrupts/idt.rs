use x86_64::instructions::segmentation::{CS, Segment};

use x86_64::structures::gdt::SegmentSelector;
use x86_64::PrivilegeLevel;

use bit_field::BitField;

pub struct Idt([Entry; 16]);

impl Idt {
    pub fn new() -> Idt {
        Idt([Entry::missing(); 16])
    }

    pub fn set_handler(&mut self, entry: u8, handler: HandlerFunc) {
        self.0[entry as usize] = Entry::new(CS::get_reg(), handler);
    }

    pub fn load(&'static self) {
        use x86_64::instructions::tables::{DescriptorTablePointer, lidt};
        use x86_64::addr::VirtAddr;
        use core::mem::size_of;

        let u64_base  = self as *const _ as u64;

        let ptr = DescriptorTablePointer {
            limit: (size_of::<Self>() - 1) as u16,
            base: VirtAddr::new(u64_base),
        };

        unsafe { lidt(&ptr) };
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Entry {
    pointer_low: u16,
    gdt_selector: SegmentSelector,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

pub type HandlerFunc = extern "C" fn() -> !;

impl Entry {
    fn new(gdt_selector: SegmentSelector, handler: HandlerFunc) -> Self {
        let pointer = handler as u64;
        Entry {
            gdt_selector: gdt_selector,
            pointer_low: pointer as u16,
            pointer_middle: (pointer >> 16) as u16,
            pointer_high: (pointer >> 32) as u32,
            options: EntryOptions::new(),
            reserved: 0,
        }
    }

    fn missing() -> Self {
        Entry {
            gdt_selector: SegmentSelector::new(0, PrivilegeLevel::Ring0),
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::minimal(),
            reserved: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EntryOptions(u16);

impl EntryOptions {
    fn minimal() -> Self {
        let mut options = 0;
        options.set_bits(9..12, 0b111);
        EntryOptions(options)
    }

    fn new() -> Self {
        let mut options = Self::minimal();
        options.set_present(true).disable_interrupts(true);
        options
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0.set_bit(15, present);
        /*self.0 = self.0 & (!0b0100_0000_0000_0000);
        if present {
            self.0 = self.0 | 0b0100_0000_0000_0000;
        }*/
        self
    }

    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.0.set_bit(8, !disable);
        /*self.0 = self.0 & (!0b0000_0000_1000_0000);
        if !disable {
            self.0 = self.0 | 0b0000_0000_1000_0000;
        }*/
        self
    }

    pub fn privilege_level(&mut self, dpl: u16) -> &mut Self {
        self.0.set_bits(13..15, dpl);
        /*self.0 = (self.0 & (!0b0111_0000_0000_0000)) | dpl;*/
        self
    }

    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0.set_bits(0..3, index);
        /*self.0 = (self.0 & (!0b0000_0000_0000_0111)) | index;*/
        self
    }
}
