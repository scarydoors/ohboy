pub mod romonly;

use crate::emulator::{Rom, mbc::romonly::RomOnly, memory::{ReadMemory, ReadWriteMemory, WriteMemory}};


#[derive(Default, Debug)]
pub enum MbcType {
    #[default]
    RomOnly,
    MBC1,
    MBC2,
    MBC3,
    MBC4,
    MBC5,
    MBC6,
    MBC7,
    MMM01,
    M161,
    HuC1,
    HuC3,
}

pub const MBC_ROM_START: u16 = 0x0000;
pub const MBC_ROM_BANK_0_END: u16 = 0x3FFF;

pub const MBC_ROM_BANK_N_START: u16 = 0x4000;
pub const MBC_ROM_END: u16 = 0x7FFF;

pub const MBC_EXTERNAL_RAM_START: u16 = 0xA000;
pub const MBC_EXTERNAL_RAM_END: u16 = 0xBFFF;

pub enum Mbc {
    RomOnly(RomOnly)
}

impl ReadMemory for Mbc {
    fn read_memory(&self, address: u16) -> u8 {
        match self {
            Mbc::RomOnly(rom_only) => rom_only.read_memory(address),
        }
    }
}

impl WriteMemory for Mbc {
    fn write_memory(&mut self, address: u16, value: u8) {
        match self {
            Mbc::RomOnly(rom_only) => rom_only.write_memory(address, value),
        }
    }
}

pub fn create_mbc(rom: &Rom) -> Mbc {
    let cartridge_type = rom.cartridge_type();
    match cartridge_type.mbc_type {
        MbcType::RomOnly => Mbc::RomOnly(RomOnly::new(rom.data.clone())),
        _ => unimplemented!("unsupported mbc type for create_mbc"),
    }
}
