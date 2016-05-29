use memory::Frame;
use super::temporarypage::TemporaryPage;
use super::mapper::Mapper;
pub use super::entry::*;

pub struct InactivePageTable {
    pub p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame,
               active_table: &mut Mapper,
               temporary_page: &mut TemporaryPage)
               -> InactivePageTable {
        {
            let table = temporary_page.map_table_frame(frame.clone(), active_table);
            // now we are able to zero the table
            table.zero();
            // set up recursive mapping for the table
            table[511].set(frame.clone(), PRESENT | WRITABLE);
        }
        temporary_page.unmap(active_table);

        InactivePageTable { p4_frame: frame }
    }
}
