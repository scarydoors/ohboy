pub mod romonly;

use crate::{mbc::romonly::RomOnly, memory::{ReadMemory, ReadWriteMemory, WriteMemory}, rom};


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

pub type MBC = Box<dyn ReadWriteMemory>;

pub fn create_mbc(rom: rom::Rom) -> MBC {
    let cartridge_type = rom.cartridge_type();
    match cartridge_type.mbc_type {
        MBCType::RomOnly => Box::new(RomOnly::new(rom)),
        _ => unimplemented!("unsupported mbc type for create_mbc"),
    }
}
