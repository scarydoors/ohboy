use crate::mbc;

pub trait ReadMemory {
    fn read_memory_u8(&self, address: u16) -> u8;
    fn read_memory_u16(&self, address: u16) -> u16;
}

pub trait WriteMemory {
    fn write_memory_u8(&mut self, address: u16, value: u8);
    fn write_memory_u16(&mut self, address: u16, value: u16);
}

pub trait ReadWriteMemory: ReadMemory + WriteMemory {}

pub struct Memory {
    mbc: mbc::MBC
}

impl Memory {
    pub fn new(mbc: mbc::MBC) -> Self {
        Self {
            mbc: mbc
        }
    }
}

impl ReadMemory for Memory {
    fn read_memory_u8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF  => self.mbc.read_memory_u8(address),
            _ => unimplemented!("address: {:x}", address)
        }
    }

    fn read_memory_u16(&self, address: u16) -> u16 {
        todo!()
    }
}

impl WriteMemory for Memory {
    fn write_memory_u8(&mut self, address: u16, value: u8) {
        todo!()
    }

    fn write_memory_u16(&mut self, address: u16, value: u16) {
        todo!()
    }
}
