use bitflags::bitflags;

use crate::{emulator::TimeCycle, memory::Memory};

bitflags! {
    #[derive(Default, Copy, Clone)]
    pub struct LCDControlFlags: u8 {
        const LCDAndPPUEnable = 1 << 7;
        const WindowTileMapArea = 1 << 6;
        const WindowEnable = 1 << 5;
        const BGAndWindowTileArea = 1 << 4;
        const BGTileMapArea = 1 << 3;
        const OBJSize = 1 << 2;
        const OBJEnable = 1 << 1;
        const BGAndWindowEnablePriority = 1;
    }
}

bitflags! {
    #[derive(Default, Copy, Clone)]
    pub struct LCDStatusFlags: u8 {
        const LYCIntSelect = 1 << 6;
        const Mode2IntSelect = 1 << 5;
        const Mode1IntSelect = 1 << 4;
        const Mode0IntSelect = 1 << 3;
        const LYCEqLY = 1 << 2;
        const PPUMode = 0b11;
    }
}

enum PPUMode {
}

const FRAMEBUFFER_WIDTH: usize = 160;
const FRAMEBUFFER_HEIGHT: usize = 144;

pub struct Ppu {
    pub framebuffer: Vec<u8>
}

impl Ppu {
    pub fn new() -> Self {
        Self { 
            framebuffer: Vec::with_capacity(FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT),
        }
    }

    pub fn step(&mut self, memory: &Memory, steps: TimeCycle) -> bool {
        false
    }
}

