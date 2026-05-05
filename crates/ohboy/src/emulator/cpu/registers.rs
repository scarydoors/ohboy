use bitflags::bitflags;

use crate::emulator::register::{ByteRegisterRead, ByteRegisterWrite, Register, WordRegisterRef, WordRegisterRefMut};



pub struct Registers {
    pub pc: Register<u16>,
    pub sp: Register<u16>,

    pub a: Register<u8>,
    pub f: Register<CpuFlags>,

    pub b: Register<u8>,
    pub c: Register<u8>,

    pub d: Register<u8>,
    pub e: Register<u8>,

    pub h: Register<u8>,
    pub l: Register<u8>
}

#[derive(Copy, Clone, Debug)]
pub enum ShortRegisterName {
    A,
    F,

    B,
    C,

    D,
    E,

    H,
    L,
}

impl std::fmt::Display for ShortRegisterName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortRegisterName::A => write!(f, "a"),
            ShortRegisterName::F => write!(f, "f"),
            ShortRegisterName::B => write!(f, "b"),
            ShortRegisterName::C => write!(f, "c"),
            ShortRegisterName::D => write!(f, "d"),
            ShortRegisterName::E => write!(f, "e"),
            ShortRegisterName::H => write!(f, "h"),
            ShortRegisterName::L => write!(f, "l"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum WordRegisterName {
    BC,
    DE,
    HL,
    AF,

    PC,
    SP,
}

impl std::fmt::Display for WordRegisterName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WordRegisterName::BC => write!(f, "bc"),
            WordRegisterName::DE => write!(f, "de"),
            WordRegisterName::HL => write!(f, "hl"),
            WordRegisterName::AF => write!(f, "af"),
            WordRegisterName::PC => write!(f, "pc"),
            WordRegisterName::SP => write!(f, "sp"),
        }
    }
}

impl Registers {
    pub fn new() -> Self {
        // initialize the registers using DMG values (from TCAGBD doc)
        Self {
            pc: 0x0100.into(),
            sp: 0xFFFE.into(),

            a: 0x01.into(),
            f: (CpuFlags::ZERO | CpuFlags::SUB | CpuFlags::CARRY).into(),

            b: 0x00.into(),
            c: 0x13.into(),

            d: 0x00.into(),
            e: 0xD8.into(),

            h: 0x01.into(),
            l: 0x4D.into(),
        }
    }
    
    pub fn get_short_register(&self, r: ShortRegisterName) -> &dyn ByteRegisterRead {
        use ShortRegisterName::*;

        match r {
            A => &self.a,
            F => &self.f,
            B => &self.b,
            C => &self.c,
            D => &self.d,
            E => &self.e,
            H => &self.h,
            L => &self.l,
        }
    }

    pub fn get_short_register_mut(&mut self, r: ShortRegisterName) -> &mut dyn ByteRegisterWrite {
        use ShortRegisterName::*;

        match r {
            A => &mut self.a,
            F => &mut self.f,
            B => &mut self.b,
            C => &mut self.c,
            D => &mut self.d,
            E => &mut self.e,
            H => &mut self.h,
            L => &mut self.l,
        }
    }

    pub fn get_word_register(&self, r: WordRegisterName) -> WordRegisterRef<'_> {
        use WordRegisterName::*;
        match r {
            BC => self.bc(),
            DE => self.de(),
            HL => self.hl(),
            AF => self.af(),
            PC => WordRegisterRef::Single(&self.pc),
            SP => WordRegisterRef::Single(&self.sp),
        }
    }

    pub fn get_word_register_mut(&mut self, r: WordRegisterName) -> WordRegisterRefMut<'_> {
        use WordRegisterName::*;
        match r {
            BC => self.bc_mut(),
            DE => self.de_mut(),
            HL => self.hl_mut(),
            AF => self.af_mut(),
            PC => WordRegisterRefMut::Single(&mut self.pc),
            SP => WordRegisterRefMut::Single(&mut self.sp),
        }
    }
}

macro_rules! impl_register_pair_methods {
    ($(($high:ident: $high_ty:ty, $low:ident: $low_ty:ty)),*$(,)?) => {
        paste::item! {
            $(
                impl Registers {
                    pub fn [<$high $low>](&self) -> crate::emulator::register::WordRegisterRef<'_> {
                        crate::emulator::register::WordRegisterRef::Pair {high: &self.$high, low: &self.$low}
                    }

                    pub fn [<$high $low _mut>](&mut self) -> crate::emulator::register::WordRegisterRefMut<'_> {
                        crate::emulator::register::WordRegisterRefMut::Pair {high: &mut self.$high, low: &mut self.$low}
                    }
                }
            )*
                
        }
    };
}

impl_register_pair_methods!(
    (b: u8, c: u8),
    (d: u8, e: u8),
    (h: u8, l: u8),
    (a: u8, f: CpuFlags),
);

bitflags! {
    #[derive(Copy, Clone)]
    pub struct CpuFlags: u8 {
        // Z Flag, set if result of an operation is zero.
        const ZERO = 1 << 7;
        // N Flag, used by DAA instruction, set if previous instruction has been a subtraction
        const SUB = 1 << 6;
        // H Flag, used by DAA instruction, set if lower nibble of the result have carried
        const HALF_CARRY = 1 << 5;
        // C Flag, set if upper nibble of the result have carried
        const CARRY = 1 << 4;
    }
}

impl ByteRegisterRead for Register<CpuFlags> {
    fn get_u8(&self) -> u8 {
        self.get().bits()
    }
}

impl ByteRegisterWrite for Register<CpuFlags> {
    fn set_u8(&mut self, value: u8) {
        self.set(CpuFlags::from_bits_truncate(value));
    }
}

