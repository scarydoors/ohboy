use bitflags::{Flags, bitflags};

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

pub struct Joypad {
    flags: JoypadFlags,

    start_pressed: bool,
    select_pressed: bool,
    b_pressed: bool,
    a_pressed: bool,
    
    down_pressed: bool,
    up_pressed: bool,
    left_pressed: bool,
    right_pressed: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            flags: JoypadFlags::from_bits_retain(0xCF),

            start_pressed: false,
            select_pressed: false,
            b_pressed: false,
            a_pressed: false,

            down_pressed: false,
            up_pressed: false,
            left_pressed: false,
            right_pressed: false,
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
        if self.flags.contains(JoypadFlags::SELECT_DPAD) {
            self.flags.set(JoypadFlags::BUTTONS_START_DPAD_DOWN, !self.down_pressed);
            self.flags.set(JoypadFlags::BUTTONS_SELECT_DPAD_UP, !self.up_pressed);
            self.flags.set(JoypadFlags::BUTTONS_B_DPAD_LEFT, !self.left_pressed);
            self.flags.set(JoypadFlags::BUTTONS_A_DPAD_RIGHT, !self.right_pressed);
        } else if self.flags.contains(JoypadFlags::SELECT_BUTTONS) {
            self.flags.set(JoypadFlags::BUTTONS_START_DPAD_DOWN, !self.down_pressed);
            self.flags.set(JoypadFlags::BUTTONS_SELECT_DPAD_UP, !self.up_pressed);
            self.flags.set(JoypadFlags::BUTTONS_B_DPAD_LEFT, !self.left_pressed);
            self.flags.set(JoypadFlags::BUTTONS_A_DPAD_RIGHT, !self.right_pressed);
        } else {
            self.flags &= !(JoypadFlags::BUTTONS_START_DPAD_DOWN | JoypadFlags::BUTTONS_SELECT_DPAD_UP | JoypadFlags::BUTTONS_B_DPAD_LEFT | JoypadFlags::BUTTONS_A_DPAD_RIGHT);

        }
    }
}
