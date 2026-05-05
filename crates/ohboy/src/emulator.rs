use std::fs::File;
use std::io::Write;

use crate::emulator::memory::Memory;
use crate::emulator::cpu::Cpu;
use crate::emulator::ppu::Ppu;

mod cpu;
mod rom;
mod memory;
mod mbc;
mod ppu;

pub use crate::emulator::rom::Rom;

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
            let pc = self.cpu.registers.pc.get();
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
    let bytes = memory.vram.0.get(0..(10*128*3)).unwrap();

    let mut tiles = Vec::new();
    for tile_bytes in bytes.chunks(16) {
        let mut idxs = Vec::new();
        for bytes in tile_bytes.chunks(2) {
            let lsb = bytes[0];
            let msb = bytes[1];

            for i in (0..8).rev() {
                let lsb_bit = (lsb >> i) & 1;
                let msb_bit = (msb >> i) & 1;

                idxs.push(msb_bit << 1 | lsb_bit);
            }
        }
        tiles.push(idxs);
    }

    let mut out = File::create("tiles.ppm").expect("failed to create tiles.ppm");
    out.write(b"P3\n").unwrap();
    out.write(format!("{} {}\n", 8 * 10, tiles.chunks(10).len() * 8).as_bytes()).unwrap();
    out.write(b"255\n").unwrap();
    for tile_row in tiles.chunks(10) {
        for y in 0..8 {
            println!("itered {y}");
            for tile in tile_row {
                let start_idx = y * 8;
                let end_idx = (y * 8) + 8;
                for tile_idx in &tile[start_idx..end_idx] {
                    let rgb = idx_to_rgb(*tile_idx);
                    out.write(format!("{} {} {} ", rgb[0], rgb[1], rgb[2]).as_bytes()).unwrap();
                }
            }
            out.write(b"\n").unwrap();
        }
    }
    panic!("stop");
    // out.write(
    //     idxs
    //         .iter()
    //         .fold(String::new(),
    //             |mut acc, i| {
    //                 acc += &idx_to_rgb(*i)
    //                     .iter()
    //                     .fold(String::new(), |acc, rgb| {
    //                         format!("{}{} ", acc, rgb)
    //                     });
    //                 acc
    //             }
    //         ).as_bytes()
    // ).unwrap();
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
