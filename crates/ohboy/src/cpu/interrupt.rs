use bitflags::bitflags;
use crate::cpu::register::Register;

pub enum RequestedIMEState {
    Enable,
    Disable,
}

bitflags! {
    #[derive(Copy, Clone)]
    pub struct RequestFlags: u8 {
        const JOYPAD = 1 << 4;
        const SERIAL = 1 << 3;
        const TIMER = 1 << 2;
        const LCD = 1 << 1;
        const VBLANK = 1;
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    pub struct EnableFlags: u8 {
        const JOYPAD = 1 << 4;
        const SERIAL = 1 << 3;
        const TIMER = 1 << 2;
        const LCD = 1 << 1;
        const VBLANK = 1;
    }
}

pub struct Interrupts {
    // interrupt master enable flag
    ime: bool,
    ime_requested_state: Option<RequestedIMEState>,

    enable_flag: Register<EnableFlags>,
    request_flag: Register<RequestFlags>
}

