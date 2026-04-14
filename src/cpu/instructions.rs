use crate::cpu::{CpuError, register::{ShortRegisterName, WordRegisterName}};
// TODO; figure out how to do the errors properly
#[derive(Debug)]
struct OperandTooWide;

#[derive(Copy, Clone, Debug)]
pub enum Operand3 {
    Register(ShortRegisterName),
    IndirectHL,
}

impl Operand3 {
    fn new(idx: u8) -> Result<Self, OperandTooWide> {
        match idx {
            0 => Ok(Self::Register(ShortRegisterName::B)),
            1 => Ok(Self::Register(ShortRegisterName::C)),
            2 => Ok(Self::Register(ShortRegisterName::D)),
            3 => Ok(Self::Register(ShortRegisterName::E)),
            4 => Ok(Self::Register(ShortRegisterName::H)),
            5 => Ok(Self::Register(ShortRegisterName::L)),
            6 => Ok(Self::IndirectHL),
            7 => Ok(Self::Register(ShortRegisterName::A)),
            _ => Err(OperandTooWide)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Operand2 {
    pub register: WordRegisterName
}

enum LastOperand2 {
    SP,
    AF
}

impl From<LastOperand2> for WordRegisterName {
    fn from(value: LastOperand2) -> Self {
        match value {
            LastOperand2::SP => WordRegisterName::SP,
            LastOperand2::AF => WordRegisterName::AF,
        }
    }
}

impl Operand2 {
    fn new(idx: u8, last: LastOperand2) -> Result<Self, OperandTooWide> {
        match idx {
            0 => Ok(Self { register: WordRegisterName::BC }),
            1 => Ok(Self { register: WordRegisterName::DE }),
            2 => Ok(Self { register: WordRegisterName::HL }),
            3 => Ok(Self { register: last.into() }),
            _ => Err(OperandTooWide)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ConditionalOperand {
    NZ,
    Z,
    NC,
    C,
}

impl ConditionalOperand {
    fn new(idx: u8) -> Result<Self, OperandTooWide> {
        match idx {
            0 => Ok(Self::NZ),
            1 => Ok(Self::Z),
            2 => Ok(Self::NC),
            3 => Ok(Self::C),
            _ => Err(OperandTooWide)
        }
    }
}


macro_rules! instructions {
    (@full [$($variant:tt)*]) => {
        #[derive(Clone, Copy, Debug)]
        pub enum Instruction {
            $($variant)*
        }
    };
    (@full [$($variant:tt)*] $name:ident {$($raw_field:ident: $raw_type:ident),*} | {$($full_field:ident: $full_type:ident),*}, $($rest:tt)*) => {
        instructions!(@full [$($variant)* $name {$($raw_field: $raw_type),*,$($full_field: $full_type),*},] $($rest)*);
    };

    (@full [$($variant:tt)*] $name:ident | {$($full_field:ident: $full_type:ident),*}, $($rest:tt)*) => {
        instructions!(@full [$($variant)* $name {$($full_field: $full_type),*},] $($rest)*);
    };

    (@full [$($variant:tt)*] $name:ident {$($raw_field:ident: $raw_type:ident),*}, $($rest:tt)*) => {
        instructions!(@full [$($variant)* $name {$($raw_field: $raw_type),*},] $($rest)*);
    };

    (@full [$($variant:tt)*] $name:ident, $($rest:tt)*) => {
        instructions!(@full [$($variant)* $name,] $($rest)*);
    };

    (@raw [$($variant:tt)*]) => {
        #[derive(Clone, Copy, Debug)]
        pub enum RawInstruction {
            $($variant)*
        }
    };
    (@raw [$($variant:tt)*] $name:ident $({$($raw_field:ident: $raw_type:ident),*})? $(| {$($_ignored:tt)*})?, $($rest:tt)*) => {
        instructions!(@raw [$($variant)* $name $({ $($raw_field: $raw_type),* })?,] $($rest)*);
    };

    ($($rest:tt)*) => {
        instructions!(@full [] $($rest)*);
        instructions!(@raw [] $($rest)*);
    };
}

instructions!(
    Nop,
    JumpRegister | { address: u8 },
    JumpRelativeConditional { operand: ConditionalOperand } | { relative: i8 },
    XorRegister { operand: Operand3 },
    LoadIndirectHLToRegister8 { operand: Operand3 },
    LoadImmediateToRegister16 { operand: Operand2 },
);

impl RawInstruction {
    pub fn new(opcode: u8) -> Result<Self, CpuError> {
        match opcode {
            0x00 => Ok(Self::Nop),
            0xC3 => Ok(Self::JumpRegister),
            op if common_bits(op, 0b1010_1000) => {
                let idx = get_00xxx000(op);
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::XorRegister { operand })
            },
            op if common_bits(op, 0b0000_0001) => {
                let idx = get_00xx0000(op);
                let operand = Operand2::new(idx, LastOperand2::SP).unwrap();
                Ok(Self::LoadImmediateToRegister16 { operand })
            },
            op if common_bits(op, 0b0010_0000) => {
                let idx = get_000xx000(op);
                let operand = ConditionalOperand::new(idx).unwrap();
                Ok(Self::JumpRelativeConditional { operand })
            },
            op if common_bits(op, 0b0100_0110) => {
                let idx = get_00xxx000(op);
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::LoadIndirectHLToRegister8 { operand })
            },
            _ => Err(CpuError::InvalidInstruction)
        }
    }
}

fn common_bits(left: u8, right: u8) -> bool {
    left & right == right
}

fn get_00xxx000(byte: u8) -> u8 {
    (byte >> 3) & 0b111
}

fn get_00xx0000(byte: u8) -> u8 {
    (byte >> 4) & 0b11
}

fn get_000xx000(byte: u8) -> u8 {
    (byte >> 3) & 0b11
}
