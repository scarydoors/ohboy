use crate::mbc;
use crate::memory::Memory;
use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::rom::Rom;

pub struct Emulator {
    cpu: Cpu,
    ppu: Ppu,
    memory: Memory
}

impl Emulator {
    pub fn new(rom: &Rom) -> Self {
        Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            memory: Memory::new(mbc::create_mbc(rom)),
        }
    }

    pub fn run_frame(&mut self) {
        loop {
            let pc = self.cpu.registers.pc().get();
            let machine_cycle = match self.cpu.cycle(&mut self.memory) {
                Ok((machine_cycle, instruction)) => {
                    println!("{:#x}: {}", pc, instruction);
                    machine_cycle
                },
                Err(e) => panic!("{}", e),
            };
            if self.ppu.step(&mut self.memory, machine_cycle.into()) {
                return
            }
        }
    }
}

pub struct TimeCycle(pub usize);

impl From<MachineCycle> for TimeCycle {
    fn from(value: MachineCycle) -> Self {
        Self(value.0 * 4)
    }
}

pub struct MachineCycle(pub usize);

