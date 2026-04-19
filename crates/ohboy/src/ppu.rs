use crate::memory::{ReadMemory, ReadWriteMemory, WriteMemory};

const VRAM_SIZE: usize = 8 * 1024;
const VRAM_START: u16 = 0x8000;
const VRAM_END: u16 = 0x9FFF;

pub struct VRam([u8; VRAM_SIZE]);

impl ReadMemory for VRam {
    fn read_memory(&self, address: u16) -> u8 {
        self.0[address - VRAM_START]
    }
}

impl WriteMemory for VRam {
    fn write_memory(&mut self, address: u16, value: u8) {
        self.0[address - VRAM_START] = value;
    }
}

