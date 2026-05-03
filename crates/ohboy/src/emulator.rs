use std::fmt::format;
use std::fs::File;
use std::io::Write;
use std::mem::take;

use crate::mbc;
use crate::memory::{Memory, ReadMemory};
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
                Err(e) => {
                    dump_tiles(&self.memory);
                    panic!("{}, should have tiles:\noam: {:?}, vram: {:?}", e, self.memory.oam, self.memory.vram,);
                }
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

fn dump_tiles(memory: &Memory) {
    let bytes = memory.vram.0.get(0..16).unwrap();

    let mut idxs = Vec::new();
    for bytes in bytes.chunks_exact(2) {
        let lsb = bytes[0];
        let msb = bytes[1];

        for i in (0..8).rev() {
            let lsb_bit = (lsb >> i) & 1;
            let msb_bit = (msb >> i) & 1;

            idxs.push(msb_bit << 1 | lsb_bit);
        }
    }
    //panic!("{:?}", idxs);

    let mut out = File::create("tiles.ppm").expect("failed to create tiles.ppm");
    out.write(b"P3\n").unwrap();
    out.write(b"8 8\n").unwrap();
    out.write(b"255\n").unwrap();
    out.write(
        idxs
            .iter()
            .fold(String::new(),
                |mut acc, i| {
                    acc += &idx_to_rgb(*i)
                        .iter()
                        .fold(String::new(), |acc, rgb| {
                            format!("{}{} ", acc, rgb)
                        });
                    acc
                }
            ).as_bytes()
    ).unwrap();
}

fn idx_to_rgb(idx: u8) -> [u8; 3] {
    match idx {
        3 => [0, 0, 0],
        2 => [85, 85, 85],
        1 => [170, 170, 170],
        0 => [255, 255, 255],
        _ => panic!("wat"),
    }
}
