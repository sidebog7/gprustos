use memory::PAGE_SIZE;
use super::VirtualAddress;


#[derive(Debug, Clone, Copy)]
pub struct Page {
    number: usize,
}

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000,
                "invalid address: 0x{:x}",
                address);
        Page { number: address / PAGE_SIZE }
    }

    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    pub fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }
    pub fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }
    pub fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }
    pub fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
}
