use bitflags::bitflags;

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct JoypadFlags: u8 {
        const SELECT_BUTTONS = 1 << 5;
        const SELECT_DPAD = 1 << 4;

        const BUTTONS_START_DPAD_DOWN = 1 << 3;
        const BUTTONS_SELECT_DPAD_UP  = 1 << 2;
        const BUTTONS_B_DPAD_LEFT  = 1 << 1;
        const BUTTONS_A_DPAD_RIGHT  = 1;
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct ButtonState {
    pub start: bool,
    pub select: bool,
    pub b: bool,
    pub a: bool,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct DpadState {
    pub down: bool,
    pub up: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Debug)]
pub struct Joypad {
    flags: JoypadFlags,

    buttons: ButtonState,
    dpad: DpadState,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            flags: JoypadFlags::from_bits_retain(0xCF),
            buttons: ButtonState::default(),
            dpad: DpadState::default(),
        }
    }

    pub fn get_memory(&self) -> u8 {
        self.flags.bits()
    }
    
    pub fn set_memory(&mut self, value: u8) {
        // We only care about the upper nibble, we always update the lower nibble here anyways.
        self.flags = JoypadFlags::from_bits_retain(value & 0xF0);
        self.update_flags();
    }

    pub fn update_flags(&mut self) {
        // On Game Boy, 0 = pressed, so we negate.
        let (b3, b2, b1, b0) = if self.flags.contains(JoypadFlags::SELECT_DPAD) {
            (!self.dpad.down, !self.dpad.up, !self.dpad.left, !self.dpad.right)
        } else if self.flags.contains(JoypadFlags::SELECT_BUTTONS) {
            (!self.buttons.start, !self.buttons.select, !self.buttons.b, !self.buttons.a)
        } else {
            (false, false, false, false)
        };


        self.flags.set(JoypadFlags::BUTTONS_START_DPAD_DOWN, b3);
        self.flags.set(JoypadFlags::BUTTONS_SELECT_DPAD_UP, b2);
        self.flags.set(JoypadFlags::BUTTONS_B_DPAD_LEFT, b1);
        self.flags.set(JoypadFlags::BUTTONS_A_DPAD_RIGHT, b0);
    }
}
