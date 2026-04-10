use bitflags::bitflags;

pub struct Registers {
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
}

impl Registers {
    pub fn new() -> Self {
        // initialize the registers using DMG values (from TCAGBD doc)
        Self {
            pc: 0x0100,
            sp: 0xFFFE,

            a: 0x01,
            f: CpuFlagRegister::ZERO_FLAG | CpuFlagRegister::SUB_FLAG | CpuFlagRegister::CARRY_FLAG,

            b: 0x00,
            c: 0x13,

            d: 0x00,
            e: 0xD8,

            h: 0x01,
            l: 0x4D,
        }
    }

    fn pc(&self) -> u16 {
        self.pc
    }

    fn sp(&self) -> u16 {
        self.sp
    }

    fn a(&self) -> u8 {
        self.a
    }

    fn f(&self) -> &CpuFlagRegister {
        &self.f
    }

    fn af(&self) -> u16 {
        ((self.a as u16) << 8) | self.f.bits() as u16
    }

    fn b(&self) -> u8 {
        self.b
    }

    fn c(&self) -> u8 {
        self.c
    }

    fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    fn d(&self) -> u8 {
        self.d
    }

    fn e(&self) -> u8 {
        self.e
    }

    fn de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    fn h(&self) -> u8 {
        self.h
    }

    fn l(&self) -> u8 {
        self.l
    }

    fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
    }

    fn set_sp(&mut self, sp: u16) {
        self.sp = sp;
    }

    fn set_a(&mut self, a: u8) {
        self.a = a;
    }

    fn set_f(&mut self, f: CpuFlagRegister) {
        self.f = f;
    }

    fn set_b(&mut self, b: u8) {
        self.b = b;
    }

    fn set_c(&mut self, c: u8) {
        self.c = c;
    }

    fn set_d(&mut self, d: u8) {
        self.d = d;
    }

    fn set_e(&mut self, e: u8) {
        self.e = e;
    }

    fn set_h(&mut self, h: u8) {
        self.h = h;
    }

    fn set_l(&mut self, l: u8) {
        self.l = l;
    }

    fn get_8_bit(&self, idx: usize) -> Option<u8> {
        // TODO: validate idx is valid with bitmask
        if idx == 7 {
            Some(self.a())
        } else {
            [
                self.b(),
                self.c(),
                self.d(),
                self.e(),
                self.h(),
                self.l(),
                self.a(),
            ]
                .get(idx)
                .copied()
        }
    }

    fn set(&mut self) {
    }
}

bitflags! {
    struct CpuFlagRegister: u8 {
        const ZERO_FLAG = 1 << 7;
        const SUB_FLAG = 1 << 6;
        const HALF_CARRY_FLAG = 1 << 5;
        const CARRY_FLAG = 1 << 4;
    }
}

#[derive(Clone, Copy)]
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

pub trait RegisterPairRead {
    fn high(&self) -> u8;
    fn low(&self) -> u8;

    fn get(&self) -> u16 {
        ((self.high() as u16) << 8) | self.low() as u16
    }
}

pub trait RegisterPairWrite<'a>: RegisterPairRead {
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

pub struct RegisterPair {
    high: Register<u8>,
    low: Register<u8>
}

impl RegisterPair {
    pub fn new(high: Register<u8>, low: Register<u8>) -> Self {
        Self {
            high,
            low,
        }
    }
}

impl RegisterPairRead for RegisterPair {
    fn high(&self) -> u8 {
        self.high.get()
    }

    fn low(&self) -> u8 {
        self.low.get()
    }
}

pub struct RegisterPairMut<'a> {
    high: &'a mut Register<u8>,
    low: &'a mut Register<u8>,
}

impl<'a> RegisterPairMut<'a> {
    pub fn new(high: &'a mut Register<u8>, low: &'a mut Register<u8>) -> Self {
        Self {
            high,
            low
        }
    }
}

impl RegisterPairRead for RegisterPairMut<'_> {
    fn high(&self) -> u8 {
        self.high.get()
    }

    fn low(&self) -> u8 {
        self.low.get()
    }
}

impl RegisterPairWrite<'_> for RegisterPairMut<'_> {
    fn set_high(&mut self, value: u8) {
        self.high.set(value);
    }

    fn set_low(&mut self, value: u8) {
        self.low.set(value);
    }
}
