use bitflags::{Flags, bitflags};

use crate::emulator::{TimeCycle, memory::Memory};

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

impl LCDStatusFlags {
    fn set_ppu_mode(&mut self, mode: PpuMode) {
        self.remove(Self::PPUMode);
        *self |= Self::from_bits_truncate(mode.into());
    }
}

enum PpuMode {
    Mode2,
    Mode3,
    Mode0,
    Mode1,
}

impl From<PpuMode> for u8 {
    fn from(value: PpuMode) -> Self {
        match value {
            PpuMode::Mode2 => 2,
            PpuMode::Mode3 => 3,
            PpuMode::Mode0 => 0,
            PpuMode::Mode1 => 1,
        }
    }
}

const FRAMEBUFFER_WIDTH: usize = 160;
const FRAMEBUFFER_HEIGHT: usize = 144;

pub struct Ppu {
    pub framebuffer: Vec<u8>,
    pub total_dots: usize 
}

const SCANLINE_LENGTH: usize = 456;

impl Ppu {
    pub fn new() -> Self {
        Self { 
            framebuffer: Vec::with_capacity(FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT),
            total_dots: 0
        }
    }

    pub fn step(&mut self, memory: &mut Memory, steps: TimeCycle) -> bool {
        for _ in 0..steps.0 {
            let ly = &mut memory.lcd_y;

            match self.scanline_dots() {
                0 => {
                    // fire mode 2 stat interrupt
                    // collect sprites
                },
                80 => {
                    // fire mode 3 stat interrupt 
                },
                252 => {
                    // mode 0
                },
                456 => {
                    ly.update(|ly| ly + 1);
                }
                _ => {}
            }

            if ly.get() as usize == FRAMEBUFFER_HEIGHT {
                // present frame bro
                // fire mode 1 vblank interrupt
            } else if ly.get() == 153 {
                ly.set(0);
            }

            self.total_dots += 1; 
        }
        false
    }

    fn scanline_dots(&mut self) -> usize {
        self.total_dots % SCANLINE_LENGTH + 1
    }
}

