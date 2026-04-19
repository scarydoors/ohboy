pub mod romonly;

use crate::{mbc::romonly::RomOnly, memory::ReadWriteMemory, rom};


#[derive(Default, Debug)]
pub enum MBCType {
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

pub type MBC = Box<dyn ReadWriteMemory>;

pub fn create_mbc(rom: rom::Rom) -> MBC {
    let cartridge_type = rom.cartridge_type();
    match cartridge_type.mbc_type {
        MBCType::RomOnly => Box::new(RomOnly::new(rom)),
        _ => unimplemented!("unsupported mbc type for create_mbc"),
    }
}
