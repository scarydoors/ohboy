use crate::{cpu::{interrupt::{self, EnableFlags, RequestFlags}, register::Register}, mbc};

pub trait ReadMemory {
    fn read_memory(&self, address: u16) -> u8;
}

pub trait WriteMemory {
    fn write_memory(&mut self, address: u16, value: u8);
}

pub trait ReadWriteMemory: ReadMemory + WriteMemory {}

impl<T: ReadMemory + WriteMemory> ReadWriteMemory for T {}

struct MemoryRegion<const N: usize, const START: u16>([u8; N]);

impl<const N: usize, const START: u16> MemoryRegion<N, START> {
    const SIZE: usize = N;
    const START: u16 = START;
    const END: u16 = START + (Self::SIZE as u16) - 1;

    fn address_to_idx(address: u16) -> usize {
        (address - Self::START) as usize
    }
}

impl<const N: usize, const START: u16> ReadMemory for MemoryRegion<N, START> {
    fn read_memory(&self, address: u16) -> u8 {
        self.0[Self::address_to_idx(address)]
    }
}

impl<const N: usize, const START: u16> WriteMemory for MemoryRegion<N, START> {
    fn write_memory(&mut self, address: u16, value: u8) {
        self.0[Self::address_to_idx(address)] = value;
    }
}

impl<const N: usize, const START: u16> Default for MemoryRegion<N, START> {
    fn default() -> Self {
        Self([0; N])
    }
}

type VRam = MemoryRegion<8192, 0x8000>;

type WRam = MemoryRegion<8192, 0xC000>;

const REQUESTED_INTERRUPTS_ADDRESS: u16 = 0xFF0F;
const ENABLED_INTERRUPTS_ADDRESS: u16 = 0xFFFF;

pub struct Memory {
    // provides rom bank and stuff
    mbc: mbc::MBC,

    vram: VRam,
    wram: WRam,

    // IO registers
    requested_interrupts: Register<interrupt::RequestFlags>,
    enabled_interrupts: Register<interrupt::EnableFlags>,
}

impl Memory {
    pub fn new(mbc: mbc::MBC) -> Self {
        Self {
            mbc: mbc,
            vram: Default::default(),
            wram: Default::default(),

            requested_interrupts: Default::default(),
            enabled_interrupts: Default::default()
        }
    }
}

impl ReadMemory for Memory {
    fn read_memory(&self, address: u16) -> u8 {
        match address {
            mbc::MBC_ROM_START..=mbc::MBC_ROM_END
            | mbc::MBC_EXTERNAL_RAM_START..=mbc::MBC_EXTERNAL_RAM_END  => self.mbc.read_memory(address),
            VRam::START..=VRam::END => self.vram.read_memory(address),
            WRam::START..=WRam::END => self.wram.read_memory(address),
            REQUESTED_INTERRUPTS_ADDRESS => self.requested_interrupts.get().bits(),
            ENABLED_INTERRUPTS_ADDRESS => self.enabled_interrupts.get().bits(),
            _ => unimplemented!("address: {:x}", address)
        }
    }
}

impl WriteMemory for Memory {
    fn write_memory(&mut self, address: u16, value: u8) {
        match address {
            mbc::MBC_ROM_START..=mbc::MBC_ROM_END
            | mbc::MBC_EXTERNAL_RAM_START..=mbc::MBC_EXTERNAL_RAM_END  => self.mbc.write_memory(address, value),
            VRam::START..=VRam::END => self.vram.write_memory(address, value),
            WRam::START..=WRam::END => self.wram.write_memory(address, value),
            REQUESTED_INTERRUPTS_ADDRESS => self.requested_interrupts.set(RequestFlags::from_bits_truncate(value)),
            ENABLED_INTERRUPTS_ADDRESS => self.enabled_interrupts.set(EnableFlags::from_bits_truncate(value)),
            _ => unimplemented!("address: {:x}", address)
        }
    }
}
