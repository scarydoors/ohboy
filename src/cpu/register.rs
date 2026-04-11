use bitflags::bitflags;

pub struct Registers {
    pc: Register<u16>,
    sp: Register<u16>,

    a: Register<u8>,
    // TODO: get rid of bitflags, not needed, simplifies implementation
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
}

impl ShortRegisterName {
    pub fn from_3_bit_index(idx: usize) -> Option<Self> {
        if idx == 7 {
            Some(Self::A)
        } else {
            [
                Self::B,
                Self::C,
                Self::D,
                Self::E,
                Self::H,
                Self::L,
            ]
                .get(idx)
                .copied()
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
    
    pub fn get_short_register(&self, r: ShortRegisterName) -> &dyn ByteRegisterRead {
        use ShortRegisterName::*;

        match r {
            A => self.a(),
            F => self.f(),
            B => self.b(),
            C => self.c(),
            D => self.d(),
            E => self.e(),
            H => self.h(),
            L => self.l(),
        }
    }

    pub fn get_short_register_mut(&mut self, r: ShortRegisterName) -> &mut dyn ByteRegisterWrite {
        use ShortRegisterName::*;

        match r {
            A => self.a_mut(),
            F => self.f_mut(),
            B => self.b_mut(),
            C => self.c_mut(),
            D => self.d_mut(),
            E => self.e_mut(),
            H => self.h_mut(),
            L => self.l_mut(),
        }
    }

    pub fn get_word_register(&self, r: WordRegisterName) -> Box<dyn WordRegisterRead + '_> {
        use WordRegisterName::*;
        match r {
            BC => Box::new(self.bc()),
            DE => Box::new(self.de()),
            HL => Box::new(self.hl()),
            AF => Box::new(self.af()),
            PC => Box::new(WordRegisterSingle(self.pc())),
            SP => Box::new(WordRegisterSingle(self.sp())),
        }
    }

    pub fn get_word_register_mut(&mut self, r: WordRegisterName) -> Box<dyn WordRegisterWrite + '_> {
        use WordRegisterName::*;
        match r {
            BC => Box::new(self.bc_mut()),
            DE => Box::new(self.de_mut()),
            HL => Box::new(self.hl_mut()),
            AF => Box::new(self.af_mut()),
            PC => Box::new(WordRegisterSingleMut(self.pc_mut())),
            SP => Box::new(WordRegisterSingleMut(self.sp_mut())),
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
                    pub fn [<$high $low>](&self) -> WordRegisterPair<Register<$high_ty>, Register<$low_ty>> {
                        WordRegisterPair::new(&self.$high, &self.$low)
                    }

                    pub fn [<$high $low _mut>](&mut self) -> WordRegisterPairMut<Register<$high_ty>, Register<$low_ty>> {
                        WordRegisterPairMut::new(&mut self.$high, &mut self.$low)
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

impl<H: ByteRegisterRead, L: ByteRegisterRead> WordRegisterRead for WordRegisterPair<'_, H, L> {
    fn get_u16(&self) -> u16 {
        self.get()
    }
}

pub trait WordRegisterPairRead {
    fn high(&self) -> u8;
    fn low(&self) -> u8;

    fn get(&self) -> u16 {
        ((self.high() as u16) << 8) | self.low() as u16
    }
}

pub trait WordRegisterPairWrite<'a>: WordRegisterPairRead {
    fn set_high(&mut self, value: u8);
    fn set_low(&mut self, value: u8);

    fn set(&mut self, value: u16) {
        self.set_high((value >> 8) as u8);
        self.set_low(value as u8);
    }

    fn update<F: Fn(u16) -> u16>(&mut self, f: F) {
        self.set(f(self.get()));
    }
}

pub struct WordRegisterPair<'a, H: ByteRegisterRead, L: ByteRegisterRead> {
    high: &'a H,
    low: &'a L
}

impl<'a, H: ByteRegisterRead, L: ByteRegisterRead> WordRegisterPair<'a, H, L> {
    pub fn new(high: &'a H, low: &'a L) -> Self {
        Self {
            high,
            low,
        }
    }
}

impl<'a, H: ByteRegisterRead, L: ByteRegisterRead> WordRegisterPairRead for WordRegisterPair<'a, H, L> {
    fn high(&self) -> u8 {
        self.high.get_u8()
    }

    fn low(&self) -> u8 {
        self.low.get_u8()
    }
}

pub struct WordRegisterPairMut<'a, H: ByteRegisterWrite, L: ByteRegisterWrite> {
    high: &'a mut H,
    low: &'a mut L,
}

impl<'a, H: ByteRegisterWrite, L: ByteRegisterWrite> WordRegisterPairMut<'a, H, L> {
    pub fn new(high: &'a mut H, low: &'a mut L) -> Self {
        Self {
            high,
            low
        }
    }
}

impl<H: ByteRegisterWrite, L: ByteRegisterWrite> WordRegisterPairRead for WordRegisterPairMut<'_, H, L> {
    fn high(&self) -> u8 {
        self.high.get_u8()
    }

    fn low(&self) -> u8 {
        self.low.get_u8()
    }
}

impl<H: ByteRegisterWrite, L: ByteRegisterWrite> WordRegisterPairWrite<'_> for WordRegisterPairMut<'_, H, L> {
    fn set_high(&mut self, value: u8) {
        self.high.set_u8(value);
    }

    fn set_low(&mut self, value: u8) {
        self.low.set_u8(value);
    }
}

impl<H: ByteRegisterWrite, L: ByteRegisterWrite> WordRegisterRead for WordRegisterPairMut<'_, H, L> {
    fn get_u16(&self) -> u16 {
        self.get()
    }
}

impl<H: ByteRegisterWrite, L: ByteRegisterWrite> WordRegisterWrite for WordRegisterPairMut<'_, H, L> {
    fn set_u16(&mut self, value: u16) {
        self.set(value);
    }
}

pub struct WordRegisterSingle<'a, T: WordRegisterRead>(&'a T);

impl<'a, T: WordRegisterRead> WordRegisterRead for WordRegisterSingle<'a, T> {
    fn get_u16(&self) -> u16 {
        self.0.get_u16()
    }
}

pub struct WordRegisterSingleMut<'a, T: WordRegisterWrite>(&'a mut T);

impl<'a, T: WordRegisterWrite> WordRegisterRead for WordRegisterSingleMut<'a, T> {
    fn get_u16(&self) -> u16 {
        self.0.get_u16()
    }
}

impl<'a, T: WordRegisterWrite> WordRegisterWrite for WordRegisterSingleMut<'a, T> {
    fn set_u16(&mut self, value: u16) {
        self.0.set_u16(value);
    }
}
