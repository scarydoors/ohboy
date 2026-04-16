use crate::mbc;

pub trait ReadMemory {
    fn read_memory(&self, address: u16) -> u8;
}

pub trait WriteMemory {
    fn write_memory(&mut self, address: u16, value: u8);
}

pub trait ReadWriteMemory: ReadMemory + WriteMemory {}

const WRAM_SIZE: usize = 8 * 1024;

pub struct Memory {
    mbc: mbc::MBC,

    wram: [u8; WRAM_SIZE],
}

impl Memory {
    pub fn new(mbc: mbc::MBC) -> Self {
        Self {
            mbc: mbc,
            wram: [0; WRAM_SIZE],
        }
    }
}

impl ReadMemory for Memory {
    fn read_memory(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF  => self.mbc.read_memory(address),
            _ => unimplemented!("address: {:x}", address)
        }
    }
}

impl WriteMemory for Memory {
    fn write_memory(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF  => self.mbc.write_memory(address, value),
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,
            _ => unimplemented!("address: {:x}", address)
        }
    }
}
