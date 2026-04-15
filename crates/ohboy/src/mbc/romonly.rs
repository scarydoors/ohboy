use crate::{memory::{ReadMemory, ReadWriteMemory, WriteMemory}, rom};

pub struct RomOnly {
    rom_data: Vec<u8>
    // TODO: external ram
}

impl RomOnly {
    pub fn new(rom: rom::Rom) -> Self {
        Self {
            rom_data: rom.data
        }
    }
}

impl ReadMemory for RomOnly {
    fn read_memory_u8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => {
                *self.rom_data.get(address as usize).unwrap()
            },
            0xA000..=0xBFFF => {
                todo!("external ram")
            },
            _ => unimplemented!()
        }
    }

    fn read_memory_u16(&self, address: u16) -> u16 {
        todo!()
    }
}

impl WriteMemory for RomOnly {
    fn write_memory_u8(&mut self, address: u16, value: u8) {
        todo!()
    }

    fn write_memory_u16(&mut self, address: u16, value: u16) {
        todo!()
    }
}

impl ReadWriteMemory for RomOnly {}
