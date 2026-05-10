use std::fs::File;
use std::io::Write;
use std::sync::mpsc::{Receiver, Sender, SyncSender, channel, sync_channel};
use std::time::Duration;

use crate::emulator::cpu::registers::Registers;
use crate::emulator::memory::Memory;
use crate::emulator::cpu::Cpu;
use crate::emulator::memory::vram::VRam;
use crate::emulator::ppu::Ppu;

pub mod cpu;
mod rom;
mod memory;
mod mbc;
mod ppu;
mod joypad;
mod register;

pub use crate::emulator::rom::Rom;

pub struct Emulator {
    cpu: Cpu,
    ppu: Ppu,
    memory: Memory,
    cycles: usize
}

impl Emulator {
    pub fn new(rom: &Rom) -> Self {
        Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            memory: Memory::new(mbc::create_mbc(rom)),
            cycles: 0
        }
    }

    pub fn run_frame(&mut self) -> bool {
        self.cycles += 1;
        let pc = self.cpu.registers.pc.get();
        let machine_cycle = match self.cpu.cycle(&mut self.memory) {
            Ok((machine_cycle, instruction)) => {
                println!("{:#x}: {}", pc, instruction);
                machine_cycle
            },
            Err(e) => {
                //dump_tiles(&self.memory);
                panic!("{}, should have tiles:\noam: {:?}, vram: {:?}, cycles: {}", e, self.memory.oam, self.memory.vram, self.cycles);
            }
        };

        self.ppu.step(&mut self.memory, machine_cycle.into())
    }
}

pub struct TimeCycle(pub usize);

impl From<MachineCycle> for TimeCycle {
    fn from(value: MachineCycle) -> Self {
        Self(value.0 * 4)
    }
}

pub struct MachineCycle(pub usize);

// fn dump_tiles(memory: &Memory) {
//     let bytes = memory.vram.0.get(0..(10*128*3)).unwrap();
//
//     let mut tiles = Vec::new();
//     for tile_bytes in bytes.chunks(16) {
//         let mut idxs = Vec::new();
//         for bytes in tile_bytes.chunks(2) {
//             let lsb = bytes[0];
//             let msb = bytes[1];
//
//             for i in (0..8).rev() {
//                 let lsb_bit = (lsb >> i) & 1;
//                 let msb_bit = (msb >> i) & 1;
//
//                 idxs.push(msb_bit << 1 | lsb_bit);
//             }
//         }
//         tiles.push(idxs);
//     }
//
//     let mut out = File::create("tiles.ppm").expect("failed to create tiles.ppm");
//     out.write(b"P3\n").unwrap();
//     out.write(format!("{} {}\n", 8 * 10, tiles.chunks(10).len() * 8).as_bytes()).unwrap();
//     out.write(b"255\n").unwrap();
//     // for tile_row in tiles.chunks(10) {
//     //     for y in 0..8 {
//     //         for tile in tile_row {
//     //             let start_idx = y * 8;
//     //             let end_idx = (y * 8) + 8;
//     //             for tile_idx in &tile[start_idx..end_idx] {
//     //                 let rgb = idx_to_rgb(*tile_idx);
//     //                 out.write(format!("{} {} {} ", rgb[0], rgb[1], rgb[2]).as_bytes()).unwrap();
//     //             }
//     //         }
//     //         out.write(b"\n").unwrap();
//     //     }
//     // }
//     // out.write(
//     //     idxs
//     //         .iter()
//     //         .fold(String::new(),
//     //             |mut acc, i| {
//     //                 acc += &idx_to_rgb(*i)
//     //                     .iter()
//     //                     .fold(String::new(), |acc, rgb| {
//     //                         format!("{}{} ", acc, rgb)
//     //                     });
//     //                 acc
//     //             }
//     //         ).as_bytes()
//     // ).unwrap();
// }

pub struct Snapshot {
    pub registers: Registers,
    pub vram: VRam,
}

impl Snapshot {
    pub fn new(emulator: &Emulator) -> Self {
        Self {
            registers: emulator.cpu.registers.clone(),
            vram: emulator.memory.vram.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub enum EmulatorCommand {
    Resume,
    Pause,
    LoadRom(Rom),
    Active(ActiveCommand),
}

#[derive(Debug, Clone)]
pub enum ActiveCommand {
    ButtonDown,
}

pub struct EmulatorHandle {
    command_tx: Sender<EmulatorCommand>,
    snapshot_rx: Receiver<Snapshot>
}

pub struct EmulatorThread {
    paused: bool,
    emulator: Option<Emulator>,
    command_rx: Receiver<EmulatorCommand>,
    snapshot_tx: SyncSender<Snapshot>
}

impl EmulatorHandle {
    pub fn spawn() -> Self {
        let (command_tx, command_rx) = channel();
        let (snapshot_tx, snapshot_rx) = sync_channel(1);

        std::thread::spawn(move || {
            let mut thread = EmulatorThread::new(command_rx, snapshot_tx);
            thread.run();
        });

        Self {
            command_tx,
            snapshot_rx
        }
    }

    pub fn send_command(&self, command: EmulatorCommand) {
        let _todo = self.command_tx.send(command);
    }

    pub fn try_recv_snapshot(&self) -> Option<Snapshot> {
        self.snapshot_rx.try_recv().ok()
    }
}

impl EmulatorThread {
    fn new(command_rx: Receiver<EmulatorCommand>, snapshot_tx: SyncSender<Snapshot>) -> Self {
        Self {
            paused: true,
            emulator: None,
            command_rx,
            snapshot_tx,
        }
    }

    fn run(&mut self) {
        loop {
            let command = self.command_rx.try_recv().ok();
            if let Some(c) = command {
                self.process_command(c);
            }

            if let Some(e) = self.emulator.as_mut() {
                // add timing for when an actual frame is ready
                if !self.paused {
                    e.run_frame();
                }

                let _ = self.snapshot_tx.try_send(Snapshot::new(e));
            }
        }
    }

    fn process_command(&mut self, command: EmulatorCommand) {
        use EmulatorCommand::*;

        match command {
            Resume => self.paused = false,
            Pause => self.paused = true,
            LoadRom(ref rom) => { self.emulator = Some(Emulator::new(rom)); },
            Active(command) => {
                if let Some(_e) = &self.emulator {
                    use ActiveCommand::*;

                    match command {
                        ButtonDown => {println!("pressed button")},
                    }
                }
            }
        };
    }
}

