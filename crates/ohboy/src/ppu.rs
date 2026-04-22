use bitflags::bitflags;

bitflags! {
    #[derive(Default, Copy, Clone)]
    pub struct LCDControlFlags: u8 {
        const LCDAndPPUEnable = 1 << 7;
        const WindowTileMapArea = 1 << 6;
        const WindowEnable = 1 << 5;
        const BGAndWindowTileArea = 1 << 4;
        const BGTileMapArea = 1 << 3;
        const OBJSize = 1 << 2;
        const OBJEnable = 1 << 1;
        const BGAndWindowEnablePriority = 1;
    }
}

bitflags! {
    #[derive(Default, Copy, Clone)]
    pub struct LCDStatusFlags: u8 {
        const LYCIntSelect = 1 << 6;
        const Mode2IntSelect = 1 << 5;
        const Mode1IntSelect = 1 << 4;
        const Mode0IntSelect = 1 << 3;
        const LYCEqLY = 1 << 2;
        const PPUMode = 0b11;
    }
}
