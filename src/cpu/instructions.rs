use crate::cpu::register::ShortRegisterName;

#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    Nop,
    JumpRegister,
    XorRegister { register: ShortRegisterName },
}

impl Instruction {
    pub fn new(opcode: u8) -> Option<Self> {
        match opcode {
            0x00 => Some(Self::Nop),
            0xC3 => Some(Self::JumpRegister),
            op if common_bits(op, 0b1010_1000) => {
                let idx = get_00xxx000(op);
                ShortRegisterName::from_3_bit_index(idx as usize).map(|r| {
                    Self::XorRegister { register: r }
                })
            },
            _ => None

        }
    }
}

fn common_bits(left: u8, right: u8) -> bool {
    left & right == right
}

fn get_00xxx000(byte: u8) -> u8 {
    (byte >> 3) & 0b111
}
