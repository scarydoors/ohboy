use bitflags::{Flags, bitflags};
#[derive(Clone, Copy, Debug, Default)]
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

    pub fn update<F: Fn(T) -> T>(&mut self, f: F) -> T {
        let result = f(self.get());
        self.set(result);
        result
    }
}

impl<T: Copy + Flags> Register<T> {
    pub fn from_bits_retain(bits: T::Bits) -> Self {
        Self(T::from_bits_retain(bits))
    }

    pub fn set_retain(&mut self, bits: T::Bits) {
        self.0 = T::from_bits_retain(bits);
    }
}

pub trait ByteRegisterRead {
    fn get_u8(&self) -> u8;
}

pub trait ByteRegisterWrite: ByteRegisterRead {
    fn set_u8(&mut self, value: u8);
    fn update_u8(&mut self, f: &dyn Fn(u8) -> u8) -> u8 {
        let result = f(self.get_u8());
        self.set_u8(result);
        result
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

pub trait WordRegisterRead {
    fn get_u16(&self) -> u16;
}

pub trait WordRegisterWrite: WordRegisterRead {
    fn set_u16(&mut self, value: u16);
    fn update_u16(&mut self, f: &dyn Fn(u16) -> u16) -> u16 {
        let result = f(self.get_u16());
        self.set_u16(result);
        result
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

macro_rules! impl_word_register_read {
    ($ty:ty) => {
        impl<'a> WordRegisterRead for $ty {
            fn get_u16(&self) -> u16 {
                match self {
                    Self::Pair { high, low } => {
                        (high.get_u8() as u16) << 8 | low.get_u8() as u16
                    }
                    Self::Single(r) => r.get_u16(),
                }
            }
        }
    };
}

pub enum WordRegisterRef<'a> {
    Pair {high: &'a dyn ByteRegisterRead, low: &'a dyn ByteRegisterRead},
    Single(&'a dyn WordRegisterRead)
}

impl_word_register_read!(WordRegisterRef<'a>);

pub enum WordRegisterRefMut<'a> {
    Pair { high: &'a mut dyn ByteRegisterWrite, low: &'a mut dyn ByteRegisterWrite },
    Single(&'a mut dyn WordRegisterWrite)
}

impl_word_register_read!(WordRegisterRefMut<'a>);

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
