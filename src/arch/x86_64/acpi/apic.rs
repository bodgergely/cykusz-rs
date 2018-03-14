use core::iter::FilterMap;
use arch::acpi::rsdt::AcpiStdHeader;

use kernel::mm::{PhysAddr,MappedAddr};

bitflags! {
    pub struct MatdEntryType : u8{
        const ProcLocalApic = 0x0;
        const ProcIOApic = 0x1;
        const IntSrcOverride = 0x2;
    }
}

#[repr(packed, C)]
pub struct MatdHeader {
    hdr: AcpiStdHeader,
    pub local_controller_address: u32,
    flags: u32
}

#[repr(packed, C)]
pub struct MatdEntry {
    typ: MatdEntryType,
    length: u8
}

pub struct MatdIter {
    matd: &'static MatdHeader,
    current: *const u8,
    limit: *const u8
}

#[repr(packed, C)]
pub struct MatdEntryIntSrc {
    matd: MatdEntry,
    bus_src: u8,
    irq_src: u8,
    global_sys_int: u32,
    flags: u16
}

impl MatdEntryIntSrc {
    pub fn irq_src(&'static self) -> u8 {
        self.irq_src
    }

    pub fn global_sys_int(&'static self) -> u32 {
        self.global_sys_int
    }
}

#[repr(packed, C)]
pub struct MatdEntryLocalApic {
    matd: MatdEntry,
    pub proc_id: u8,
    pub apic_id: u8,
    flags: u32
}


#[repr(packed, C)]
pub struct MatdEntryIOApic {
    matd: MatdEntry,
    pub ioapic_id: u8,
    reserved: u8,
    pub ioapic_address: u32,
    pub global_int_base: u32
}

impl MatdEntryIOApic {
    pub fn ioapic_address(&'static self) -> MappedAddr {
        PhysAddr(self.ioapic_address as usize).to_mapped()
    }
}

impl MatdHeader {
    pub fn entries(&'static self) -> MatdIter {
        MatdIter {
            matd: self,
            current: unsafe {
                (self as *const _ as *const u8)
                    .offset(::core::mem::size_of::<MatdHeader>() as isize)
            },
            limit: unsafe {
                (self as *const _ as *const u8)
                    .offset(self.hdr.length as isize)
            }
        }
    }

    pub fn lapic_address(&'static self) -> MappedAddr {
        PhysAddr(self.local_controller_address as usize).to_mapped()
    }

    pub fn lapic_entries(&'static self)
        -> FilterMap<MatdIter, fn(&MatdEntry) -> Option<&'static MatdEntryLocalApic>> {

        self.entries().filter_map(|e| {
            if e.typ == MatdEntryType::ProcLocalApic {
                unsafe {
                    Some(&*(e as *const _ as *const MatdEntryLocalApic))
                }
            } else {
                None
            }
        })
    }

    pub fn ioapic_entries(&'static self)
        -> FilterMap<MatdIter, fn(&MatdEntry) -> Option<&'static MatdEntryIOApic>> {

        self.entries().filter_map(|e| {
            if e.typ == MatdEntryType::ProcIOApic {
                unsafe {
                    Some(&*(e as *const _ as *const MatdEntryIOApic))
                }
            } else {
                None
            }
        })
    }

    pub fn intsrc_entries(&'static self)
        -> FilterMap<MatdIter, fn(&MatdEntry) -> Option<&'static MatdEntryIntSrc>> {

        self.entries().filter_map(|e| {
            if e.typ == MatdEntryType::IntSrcOverride {
                unsafe {
                    Some(&*(e as *const _ as *const MatdEntryIntSrc))
                }
            } else {
                None
            }
        })
    }

    pub fn find_irq_remap(&'static self, int: u32) -> u32 {
        self.intsrc_entries().find(|i| {
            i.irq_src() as u32 == int
        }).map_or(int, |e| {
            e.global_sys_int()
        })
    }
}

impl Iterator for MatdIter {
    type Item = &'static MatdEntry;

    fn next(&mut self) -> Option<&'static MatdEntry> {
        if self.current < self.limit {
            let r = unsafe {
                &*(self.current as *const MatdEntry)
            };

            unsafe {
                self.current = self.current.offset(r.length as isize);
            };

            return Some(r);
        }

        return None;
    }
}