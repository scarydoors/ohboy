use bitflags::bitflags;

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
