use crate::emulator::{TimeCycle, memory::Memory};

pub struct Dma {
    cycles_left: Option<TimeCycle>,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            cycles_left: None
        }
    }

    pub fn step(memory: &mut Memory, steps: TimeCycle) {
        
    }
}
