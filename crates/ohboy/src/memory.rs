use crate::mbc;

pub trait ReadMemory {
    fn read_memory(&self, address: u16) -> u8;
}

pub trait WriteMemory {
    fn write_memory(&mut self, address: u16, value: u8);
}

pub trait ReadWriteMemory: ReadMemory + WriteMemory {}

impl<T: ReadMemory + WriteMemory> ReadWriteMemory for T {}


const WRAM_SIZE: usize = 8 * 1024;
const WRAM_START: u16 = 0xC000;
const WRAM_END: u16 = 0xDFFF;

pub struct Memory {
    // provides rom bank and stuff
    mbc: mbc::MBC,

    vram: [u8; VRAM_SIZE],
    wram: [u8; WRAM_SIZE],
}

impl Memory {
    pub fn new(mbc: mbc::MBC) -> Self {
        Self {
            mbc: mbc,
            vram: [0; VRAM_SIZE],
            wram: [0; WRAM_SIZE],
        }
    }
}

impl ReadMemory for Memory {
    fn read_memory(&self, address: u16) -> u8 {
        match address {
            mbc::MBC_ROM_START..=mbc::MBC_ROM_END
            | mbc::MBC_EXTERNAL_RAM_START..=mbc::MBC_EXTERNAL_RAM_END  => self.mbc.read_memory(address),
            VRAM_START..=VRAM_END => self.vram[(address - 
            WRAM_START..=WRAM_END => self.wram[(address - 0xC000) as usize],
            _ => unimplemented!("address: {:x}", address)
        }
    }
}

fn read_array()

impl WriteMemory for Memory {
    fn write_memory(&mut self, address: u16, value: u8) {
        match address {
            mbc::MBC_ROM_START..=mbc::MBC_ROM_END
            | mbc::MBC_EXTERNAL_RAM_START..=mbc::MBC_EXTERNAL_RAM_END  => self.mbc.write_memory(address, value),
            WRAM_START..=WRAM_END => self.wram[(address - 0xC000) as usize] = value,
            _ => unimplemented!("address: {:x}", address)
        }
    }
}
