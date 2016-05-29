
pub use self::entry::*;
use memory::{PAGE_SIZE, Frame, FrameAllocator};
use multiboot2::BootInformation;
use self::page::Page;
use self::inactivepagetable::InactivePageTable;
use self::temporarypage::TemporaryPage;
pub use self::mapper::Mapper;
use core::ops::{Deref, DerefMut};

mod entry;

mod table;
mod page;

mod mapper;
mod inactivepagetable;
mod temporarypage;

pub const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;


pub fn remap_the_kernel<A>(allocator: &mut A, boot_info: &BootInformation)
    where A: FrameAllocator
{
    let mut temporary_page = TemporaryPage::new(Page::new(0xcafebabe), allocator);

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("no more frames");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        let elf_sections_tag = boot_info.elf_sections_tag()
            .expect("Memory map tag required");

        for section in elf_sections_tag.sections() {
            use self::entry::WRITABLE;

            if !section.is_allocated() {
                // section is not loaded to memory
                continue;
            }
            assert!(section.addr as usize % PAGE_SIZE == 0,
                    "sections need to be page aligned");

            println!("mapping section at addr: {:#x}, size: {:#x}",
                     section.addr,
                     section.size);

            let flags = EntryFlags::from_elf_section_flags(section);

            let start_frame = Frame::containing_address(section.start_address());
            let end_frame = Frame::containing_address(section.end_address() - 1);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }
        }
        // identity map the VGA text buffer
        let vga_buffer_frame = Frame::containing_address(0xb8000);
        mapper.identity_map(vga_buffer_frame, WRITABLE, allocator);

        // identity map the multiboot info structure
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, PRESENT, allocator);
        }

    });

    let old_table = active_table.switch(new_table);
    println!("NEW TABLE!!!");

    // turn the old p4 page into a guard page
    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page, allocator);
    println!("guard page at {:#x}", old_p4_page.start_address());
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    unsafe fn new() -> ActivePageTable {
        ActivePageTable { mapper: Mapper::new() }
    }

    pub fn with<F>(&mut self,
                   table: &mut InactivePageTable,
                   temporary_page: &mut TemporaryPage, // new
                   f: F)
        where F: FnOnce(&mut Mapper)
    {
        use x86::{controlregs, tlb};
        let flush_tlb = || unsafe { tlb::flush_all() };

        {
            let backup = Frame::containing_address(unsafe { controlregs::cr3() } as usize);

            // map temporary_page to current p4 table
            let p4_table = temporary_page.map_table_frame(backup.clone(), self);

            // overwrite recursive mapping
            self.p4_mut()[511].set(table.p4_frame.clone(), PRESENT | WRITABLE);
            flush_tlb();

            // execute f in the new context
            f(self);

            // restore recursive mapping to original p4 table
            p4_table[511].set(backup, PRESENT | WRITABLE);
            flush_tlb();
        }

        temporary_page.unmap(self);
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        use x86::controlregs;

        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(unsafe { controlregs::cr3() } as usize),
        };
        unsafe {
            controlregs::cr3_write(new_table.p4_frame.start_address() as u64);
        }
        old_table
    }
}
