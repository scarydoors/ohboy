use std::{env, fs::{self}, path, process::ExitCode};

use winit::event_loop::{ControlFlow, EventLoop};

use crate::{app::App, emulator::{Emulator, EmulatorHandle, Rom}};

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

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    // TODO: event loop proxy so we can avoid using about_to_wait and properly wake up the event loop
    let emulator_handle = EmulatorHandle::spawn();
    let mut app = App::new(emulator_handle);

    app.load_rom(rom);
    event_loop.run_app(&mut app).unwrap();

    ExitCode::SUCCESS
}
