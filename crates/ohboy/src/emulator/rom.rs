use crate::emulator::mbc;

#[derive(Debug, Clone)]
pub struct Rom {
    pub data: Vec<u8>,
}

const TITLE_RANGE: std::ops::RangeInclusive<usize> = 0x0134..=0x0143;

const ROM_SIZE_IDX: usize = 0x0148;

const RAM_SIZE_IDX: usize = 0x0149;

const CARTRIDGE_TYPE_IDX: usize = 0x0147;

impl Rom {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
        }
    }

    pub fn title(&self) -> &str {
        str::from_utf8(self.data.get(TITLE_RANGE).unwrap()).unwrap()
    }

    pub fn cartridge_type(&self) -> CartridgeType {
        CartridgeType::from_byte(*self.data.get(CARTRIDGE_TYPE_IDX).unwrap())
    }

    pub fn rom_size(&self) -> usize {
        32 * (1 << self.data.get(ROM_SIZE_IDX).unwrap())
    }

    pub fn ram_size(&self) -> usize {
        match self.data.get(RAM_SIZE_IDX).unwrap() {
            0x00 => 0,
            0x01 => 0,
            0x02 => 8,
            0x03 => 32,
            0x04 => 128,
            0x05 => 64,
            _ => panic!("unsupported ram size")
        }
    }
}

#[derive(Default, Debug)]
pub struct CartridgeType {
    pub mbc_type: mbc::MbcType,
    pub ram: bool,
    pub battery: bool,
    pub timer: bool,
    pub rumble: bool,
    pub sensor: bool,
    pub pocket_camera: bool,
    pub bandai_tama5: bool,
}

impl CartridgeType {
    pub fn from_byte(value: u8) -> Self {
        match value {
            0x00 => Self {
                mbc_type: mbc::MbcType::RomOnly,
                ..Default::default()
            },
            0x01 => Self {
                mbc_type: mbc::MbcType::MBC1,
                ..Default::default()
            },
            0x02 => Self {
                mbc_type: mbc::MbcType::MBC1,
                ram: true,
                ..Default::default()
            },
            0x03 => Self {
                mbc_type: mbc::MbcType::MBC1,
                ram: true,
                battery: true,
                ..Default::default()
            },
            0x05 => Self {
                mbc_type: mbc::MbcType::MBC2,
                ..Default::default()
            },
            0x06 => Self {
                mbc_type: mbc::MbcType::MBC2,
                battery: true,
                ..Default::default()
            },
            0x08 => Self {
                mbc_type: mbc::MbcType::RomOnly,
                ram: true,
                ..Default::default()
            },
            0x09 => Self {
                mbc_type: mbc::MbcType::RomOnly,
                ram: true,
                battery: true,
                ..Default::default()
            },
            0x0B => Self {
                mbc_type: mbc::MbcType::MMM01,
                ..Default::default()
            },
            0x0C => Self {
                mbc_type: mbc::MbcType::MMM01,
                ram: true,
                ..Default::default()
            },
            0x0D => Self {
                mbc_type: mbc::MbcType::MMM01,
                ram: true,
                battery: true,
                ..Default::default()
            },
            0x0F => Self {
                mbc_type: mbc::MbcType::MBC3,
                timer: true,
                battery: true,
                ..Default::default()
            },
            0x10 => Self {
                mbc_type: mbc::MbcType::MBC3,
                timer: true,
                ram: true,
                battery : true,
                ..Default::default()
            },
            0x11 => Self {
                mbc_type: mbc::MbcType::MBC3,
                ..Default::default()
            },
            0x12 => Self {
                mbc_type: mbc::MbcType::MBC3,
                ram : true,
                ..Default::default()
            },
            0x13 => Self {
                mbc_type: mbc::MbcType::MBC3,
                ram: true,
                battery : true,
                ..Default::default()
            },
            0x19 => Self {
                mbc_type: mbc::MbcType::MBC5,
                ..Default::default()
            },
            0x1A => Self {
                mbc_type: mbc::MbcType::MBC5,
                ram: true,
                ..Default::default()
            },
            0x1B => Self {
                mbc_type: mbc::MbcType::MBC5,
                ram: true,
                battery: true,
                ..Default::default()
            },
            0x1C => Self {
                mbc_type: mbc::MbcType::MBC5,
                rumble: true,
                ..Default::default()
            },
            0x1D => Self {
                mbc_type: mbc::MbcType::MBC5,
                rumble: true,
                ram: true,
                ..Default::default()
            },
            0x1E => Self {
                mbc_type: mbc::MbcType::MBC5,
                rumble: true,
                ram: true,
                battery: true,
                ..Default::default()
            },
            0x20 => Self {
                mbc_type: mbc::MbcType::MBC6,
                ..Default::default()
            },
            0x22 => Self {
                mbc_type: mbc::MbcType::MBC7,
                sensor: true,
                rumble: true,
                ram: true,
                battery: true,
                ..Default::default()
            },
            _ => unimplemented!("unsupported cartridge type"),
        }
    }
}


