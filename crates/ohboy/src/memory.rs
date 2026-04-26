use bitflags::bitflags;

use crate::{cpu::{interrupt::{self, EnableFlags, RequestFlags}, register::Register}, mbc, ppu::{LCDControlFlags, LCDStatusFlags}};

pub trait ReadMemory {
    fn read_memory(&self, address: u16) -> u8;
}

pub trait WriteMemory {
    fn write_memory(&mut self, address: u16, value: u8);
}

pub trait ReadWriteMemory: ReadMemory + WriteMemory {}

impl<T: ReadMemory + WriteMemory> ReadWriteMemory for T {}

struct MemoryRegion<const N: usize, const START: u16, const END: u16>([u8; N]);

impl<const N: usize, const START: u16, const END: u16> MemoryRegion<N, START, END> {
    const SIZE: usize = {
        assert!(END >= START, "invalid region: END must be >= START");
        assert!(N == END as usize - START as usize + 1, "N must be equal to number of addresses in START..=END");
        N
    };
    const START: u16 = START;
    const END: u16 = END;

    fn address_to_idx(address: u16) -> usize {
        debug_assert!((START..=END).contains(&address), "address out of region bounds");
        (address - Self::START) as usize
    }
}

impl<const N: usize, const START: u16, const END: u16> ReadMemory for MemoryRegion<N, START, END> {
    fn read_memory(&self, address: u16) -> u8 {
        self.0[Self::address_to_idx(address)]
    }
}

impl<const N: usize, const START: u16, const END: u16> WriteMemory for MemoryRegion<N, START, END> {
    fn write_memory(&mut self, address: u16, value: u8) {
        self.0[Self::address_to_idx(address)] = value;
    }
}

impl<const N: usize, const START: u16, const END: u16> Default for MemoryRegion<N, START, END> {
    fn default() -> Self {
        Self([0; N])
    }
}

type VRam = MemoryRegion<8192, 0x8000, 0x9FFF>;
type WRam = MemoryRegion<8192, 0xC000, 0xDFFF>;

const REQUESTED_INTERRUPTS_ADDRESS: u16 = 0xFF0F;

const SERIAL_TRANSFER_DATA_ADDRESS: u16 = 0xFF01;
const SERIAL_TRANSFER_CONTROL_ADDRESS: u16 = 0xFF02;

const LCD_CONTROL_ADDRESS: u16 = 0xFF40;
const LCD_STATUS_ADDRESS: u16 = 0xFF41;
const LCD_Y_ADDRESS: u16 = 0xFF44;
const SCREEN_Y_ADDRESS: u16 = 0xFF42;
const SCREEN_X_ADDRESS: u16 = 0xFF43;

type HRam = MemoryRegion<127, 0xFF80, 0xFFFE>;

const ENABLED_INTERRUPTS_ADDRESS: u16 = 0xFFFF;

pub struct Memory {
    // provides rom bank and stuff
    mbc: mbc::MBC,

    vram: VRam,
    wram: WRam,

    // IO registers
    requested_interrupts: Register<interrupt::RequestFlags>,

    serial_transfer_data: Register<u8>,
    serial_transfer_control: Register<SerialControlFlags>,

    lcd_control: Register<LCDControlFlags>,
    lcd_status: Register<LCDStatusFlags>,
    lcd_y: Register<u8>,
    screen_y: Register<u8>,
    screen_x: Register<u8>,

    hram: HRam,
    enabled_interrupts: Register<interrupt::EnableFlags>,
}

impl Memory {
    pub fn new(mbc: mbc::MBC) -> Self {
        Self {
            mbc: mbc,
            vram: Default::default(),
            wram: Default::default(),
            hram: Default::default(),

            requested_interrupts: Register::from_bits_retain(0xE1),

            serial_transfer_data: Default::default(),
            serial_transfer_control: Register::from_bits_retain(0x7E),

            lcd_control: Register::from_bits_retain(0x91),
            lcd_status: Register::from_bits_retain(0x85),
            lcd_y: Default::default(),
            screen_y: Default::default(),
            screen_x: Default::default(),

            enabled_interrupts: Default::default(),
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
            SERIAL_TRANSFER_DATA_ADDRESS => self.serial_transfer_data.get(),
            SERIAL_TRANSFER_CONTROL_ADDRESS => self.serial_transfer_control.get().bits(),
            LCD_CONTROL_ADDRESS => self.lcd_control.get().bits(),
            LCD_STATUS_ADDRESS => self.lcd_status.get().bits(),
            LCD_Y_ADDRESS => self.lcd_y.get(),
            SCREEN_Y_ADDRESS => self.screen_y.get(),
            SCREEN_X_ADDRESS => self.screen_x.get(),
            HRam::START..=HRam::END => self.hram.read_memory(address),
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
            SERIAL_TRANSFER_DATA_ADDRESS => self.serial_transfer_data.set(value),
            SERIAL_TRANSFER_CONTROL_ADDRESS => self.serial_transfer_control.set(SerialControlFlags::from_bits_truncate(value)),
            LCD_CONTROL_ADDRESS => self.lcd_control.set(LCDControlFlags::from_bits_truncate(value)),
            LCD_STATUS_ADDRESS => self.lcd_status.set(LCDStatusFlags::from_bits_truncate(value)),
            SCREEN_Y_ADDRESS => self.screen_y.set(value),
            SCREEN_X_ADDRESS => self.screen_x.set(value),
            HRam::START..=HRam::END => self.hram.write_memory(address, value),
            ENABLED_INTERRUPTS_ADDRESS => self.enabled_interrupts.set(EnableFlags::from_bits_truncate(value)),
            _ => unimplemented!("address: {:x}", address)
        }
    }
}

bitflags! {
    #[derive(Default, Clone, Copy)]
    pub struct SerialControlFlags: u8 {
        const TRANSFER_ENABLE = 1 << 7;
        const CLOCK_SELECT = 1;
    }
}
