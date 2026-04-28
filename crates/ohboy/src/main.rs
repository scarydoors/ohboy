use std::{env, fs::{self}, path, process::ExitCode};

use crate::{cpu::Cpu, emulator::Emulator, rom::Rom};

mod cpu;
mod rom;
mod memory;
mod mbc;
mod ppu;
mod emulator;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let filepath = match args.get(1) {
        Some(s) => {
            path::Path::new(s)
        },
        None => {
            eprintln!("missing path argument");
            return ExitCode::from(2);
        },
    };

    let rom = Rom::new(fs::read(filepath).unwrap());

    println!("Loaded ROM with title: {}", rom.title());
    println!("ROM size: {}KiB", rom.rom_size());
    println!("SRAM size: {}KiB", rom.ram_size());
    println!("MBC type: {:?}", rom.cartridge_type());

    let mut emulator = Emulator::new(&rom);
    loop {
        emulator.run_frame();
    }

    ExitCode::SUCCESS
}
