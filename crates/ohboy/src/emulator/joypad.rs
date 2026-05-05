use bitflags::bitflags;

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct JoypadFlags: u8 {
        const SELECT_BUTTONS = 1 << 5;
        const SELECT_DPAD = 1 << 4;

        const BUTTONS_START = 1 << 3;
        const DPAD_DOWN = 1 << 3;

        const BUTTONS_SELECT = 1 << 2;
        const DPAD_UP = 1 << 2;

        const BUTTONS_B = 1 << 1;
        const DPAD_LEFT = 1 << 1;

        const BUTTONS_A = 1;
        const DPAD_RIGHT = 1;
    }
}
