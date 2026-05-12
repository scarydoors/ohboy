use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
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
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub memory: Memory,
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
        if pc == 0x6556 {
            return true;
        }

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

pub struct State {
    pub paused: bool,
    pub emulator: Option<Emulator>
}

impl State {
    fn new() -> Self {
        Self {
            paused: true,
            emulator: None
        }
    }
}

pub struct EmulatorHandle {
    command_tx: Sender<EmulatorCommand>,
    pub state: Arc<Mutex<State>>
}

pub struct EmulatorThread {
    command_rx: Receiver<EmulatorCommand>,
    state: Arc<Mutex<State>>
}

impl EmulatorHandle {
    pub fn spawn() -> Self {
        let (command_tx, command_rx) = channel();

        let state = Arc::new(Mutex::new(State::new()));
        let cloned = state.clone();
        std::thread::spawn(move || {
            let mut thread = EmulatorThread::new(command_rx, cloned);
            thread.run();
        });

        Self {
            state,
            command_tx,
        }
    }

    pub fn send_command(&self, command: EmulatorCommand) {
        let _todo = self.command_tx.send(command);
    }
}

impl EmulatorThread {
    fn new(command_rx: Receiver<EmulatorCommand>, state: Arc<Mutex<State>>) -> Self {
        Self {
            state,
            command_rx,
        }
    }

    fn run(&mut self) {
        loop {
            {
                let mut state = self.state.lock().unwrap();
                while let Ok(c) = self.command_rx.try_recv() {
                    self.process_command(&mut state, c);
                }

                if !state.paused {
                    if let Some(e) = state.emulator.as_mut() {
                        while !e.run_frame() {}
                        // add timing for when an actual frame is ready
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(16));
        }
    }

    fn process_command(&self, state: &mut State, command: EmulatorCommand) {
        use EmulatorCommand::*;

        match command {
            Resume => state.paused = false,
            Pause => state.paused = true,
            LoadRom(ref rom) => { state.emulator = Some(Emulator::new(rom)); },
            Active(command) => {
                if let Some(_e) = &state.emulator {
                    use ActiveCommand::*;

                    match command {
                        ButtonDown => {println!("pressed button")},
                    }
                }
            }
        };
    }
}

