use std::{env, fs::{self}, path, process::ExitCode};

use winit::event_loop::{ControlFlow, EventLoop};

use crate::{emulator::{Emulator, Rom}, app::App};

mod emulator;
mod app;

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

    //let mut emulator = Emulator::new(&rom);
    //loop {
    //    emulator.run_frame();
    //}
    
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();

    app.load_rom(&rom);
    event_loop.run_app(&mut app).unwrap();

    ExitCode::SUCCESS
}
