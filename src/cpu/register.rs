use bitflags::bitflags;

pub struct Registers {
    pc: Register<u16>,
    sp: Register<u16>,

    a: Register<u8>,
    f: Register<CpuFlagRegister>,

    b: Register<u8>,
    c: Register<u8>,

    d: Register<u8>,
    e: Register<u8>,

    h: Register<u8>,
    l: Register<u8>
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
    HLIndirect,
}
impl ShortRegisterName {
    pub fn from_3_bit_index(idx: usize) -> Option<Self> {
        [
            Self::B,
            Self::C,
            Self::D,
            Self::E,
            Self::H,
            Self::L,
            Self::HLIndirect,
            Self::A
        ]
            .get(idx)
            .copied()
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

impl WordRegisterName {
    pub fn from_2_bit_index(idx: usize) -> Option<Self> {
        [
            Self::BC,
            Self::DE,
            Self::HL,
            Self::AF
        ]
            .get(idx)
            .copied()
    }
}

impl Registers {
    pub fn new() -> Self {
        // initialize the registers using DMG values (from TCAGBD doc)
        Self {
            pc: 0x0100.into(),
            sp: 0xFFFE.into(),

            a: 0x01.into(),
            f: (CpuFlagRegister::ZERO_FLAG | CpuFlagRegister::SUB_FLAG | CpuFlagRegister::CARRY_FLAG).into(),

            b: 0x00.into(),
            c: 0x13.into(),

            d: 0x00.into(),
            e: 0xD8.into(),

            h: 0x01.into(),
            l: 0x4D.into(),
        }
    }
    
    pub fn get_short_register(&self, r: ShortRegisterName) -> Result<&dyn ByteRegisterRead, String> {
        use ShortRegisterName::*;

        match r {
            A => Ok(self.a()),
            F => Ok(self.f()),
            B => Ok(self.b()),
            C => Ok(self.c()),
            D => Ok(self.d()),
            E => Ok(self.e()),
            H => Ok(self.h()),
            L => Ok(self.l()),
            HLIndirect => Err("must handle HL indirect manually".into()),
        }
    }

    pub fn get_short_register_mut(&mut self, r: ShortRegisterName) -> Result<&mut dyn ByteRegisterWrite, String> {
        use ShortRegisterName::*;

        match r {
            A => Ok(self.a_mut()),
            F => Ok(self.f_mut()),
            B => Ok(self.b_mut()),
            C => Ok(self.c_mut()),
            D => Ok(self.d_mut()),
            E => Ok(self.e_mut()),
            H => Ok(self.h_mut()),
            L => Ok(self.l_mut()),
            HLIndirect => Err("must handle HL indirect manually".into()),
        }
    }

    pub fn get_word_register(&self, r: WordRegisterName) -> WordRegisterRef {
        use WordRegisterName::*;
        match r {
            BC => self.bc(),
            DE => self.de(),
            HL => self.hl(),
            AF => self.af(),
            PC => WordRegisterRef::Single(self.pc()),
            SP => WordRegisterRef::Single(self.sp()),
        }
    }

    pub fn get_word_register_mut(&mut self, r: WordRegisterName) -> WordRegisterRefMut {
        use WordRegisterName::*;
        match r {
            BC => self.bc_mut(),
            DE => self.de_mut(),
            HL => self.hl_mut(),
            AF => self.af_mut(),
            PC => WordRegisterRefMut::Single(self.pc_mut()),
            SP => WordRegisterRefMut::Single(self.sp_mut()),
        }
    }
}

macro_rules! impl_register_methods {
    ($($register:ident: $register_ty:ty),*$(,)?) => {
        paste::item! {
                impl Registers {
                $(
                    pub fn $register(&self) -> &Register<$register_ty> {
                        &self.$register 
                    }
                    
                    pub fn [<$register _mut>](&mut self) -> &mut Register<$register_ty> {
                        &mut self.$register
                    }
                )*
                }
        }
    }
}

impl_register_methods!(
    pc: u16,
    sp: u16,

    a: u8,
    f: CpuFlagRegister,

    b: u8,
    c: u8,

    d: u8,
    e: u8,

    h: u8,
    l: u8
);

macro_rules! impl_register_pair_methods {
    ($(($high:ident: $high_ty:ty, $low:ident: $low_ty:ty)),*$(,)?) => {
        paste::item! {
            $(
                impl Registers {
                    pub fn [<$high $low>](&self) -> WordRegisterRef {
                        WordRegisterRef::Pair {high: &self.$high, low: &self.$low}
                    }

                    pub fn [<$high $low _mut>](&mut self) -> WordRegisterRefMut {
                        WordRegisterRefMut::Pair {high: &mut self.$high, low: &mut self.$low}
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
    (a: u8, f: CpuFlagRegister),
);

bitflags! {
    #[derive(Copy, Clone)]
    struct CpuFlagRegister: u8 {
        const ZERO_FLAG = 1 << 7;
        const SUB_FLAG = 1 << 6;
        const HALF_CARRY_FLAG = 1 << 5;
        const CARRY_FLAG = 1 << 4;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Register<T: Copy>(T); 

impl<T: Copy> Register<T> {
    pub fn new(value: T) -> Self {
        Register(value)
    }

    pub fn get(&self) -> T {
        self.0
    }

    pub fn set(&mut self, value: T) {
        self.0 = value
    }

    pub fn update<F: Fn(T) -> T>(&mut self, f: F) {
        self.set(f(self.get()))
    }
}

pub trait ByteRegisterRead {
    fn get_u8(&self) -> u8;
}

pub trait ByteRegisterWrite: ByteRegisterRead {
    fn set_u8(&mut self, value: u8);
    fn update_u8(&mut self, f: &dyn Fn(u8) -> u8) {
        self.set_u8(f(self.get_u8()))
    }
}

impl<T: Copy> From<T> for Register<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
} 

impl ByteRegisterRead for Register<u8> {
    fn get_u8(&self) -> u8 {
        self.get()
    }
}

impl ByteRegisterWrite for Register<u8> {
    fn set_u8(&mut self, value: u8) {
        self.set(value);
    }
}

impl ByteRegisterRead for Register<CpuFlagRegister> {
    fn get_u8(&self) -> u8 {
        self.get().bits()
    }
}

impl ByteRegisterWrite for Register<CpuFlagRegister> {
    fn set_u8(&mut self, value: u8) {
        self.set(CpuFlagRegister::from_bits_truncate(value));
    }
}

pub trait WordRegisterRead {
    fn get_u16(&self) -> u16;
}

pub trait WordRegisterWrite: WordRegisterRead {
    fn set_u16(&mut self, value: u16);
    fn update_u16(&mut self, f: &dyn Fn(u16) -> u16) {
        self.set_u16(f(self.get_u16()))
    }
}

impl WordRegisterRead for Register<u16> {
    fn get_u16(&self) -> u16 {
        self.get()
    }
}

impl WordRegisterWrite for Register<u16> {
    fn set_u16(&mut self, value: u16) {
        self.set(value);
    }
}

pub enum WordRegisterRef<'a> {
    Pair {high: &'a dyn ByteRegisterRead, low: &'a dyn ByteRegisterRead},
    Single(&'a dyn WordRegisterRead)
}

impl<'a> WordRegisterRead for WordRegisterRef<'a> {
    fn get_u16(&self) -> u16 {
        match self {
            Self::Pair { high, low } => {
                (high.get_u8() as u16) << 8  | low.get_u8() as u16
            },
            Self::Single(r) => {
                r.get_u16()
            },
        }
    }
}

pub enum WordRegisterRefMut<'a> {
    Pair { high: &'a mut dyn ByteRegisterWrite, low: &'a mut dyn ByteRegisterWrite },
    Single(&'a mut dyn WordRegisterWrite)
}

impl<'a> WordRegisterRead for WordRegisterRefMut<'a> {
    fn get_u16(&self) -> u16 {
        match self {
            Self::Pair { high, low } => {
                (high.get_u8() as u16) << 8  | low.get_u8() as u16
            },
            Self::Single(r) => {
                r.get_u16()
            },
        }
    }
}

impl<'a> WordRegisterWrite for WordRegisterRefMut<'a> {
    fn set_u16(&mut self, value: u16) {
        match self {
            WordRegisterRefMut::Pair { high, low } => {
                high.set_u8((value >> 8) as u8);
                low.set_u8(value as u8);
            },
            WordRegisterRefMut::Single(r) => {
                r.set_u16(value);
            },
        }
    }
}
