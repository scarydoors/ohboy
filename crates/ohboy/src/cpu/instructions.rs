use crate::cpu::{CpuError, register::{ShortRegisterName, WordRegisterName}};
use ohboy_macro::byte_permutations;
// TODO; figure out how to do the errors properly
#[derive(Debug)]
struct OperandTooWide;

#[derive(Copy, Clone, Debug)]
pub enum Operand3 {
    Register(ShortRegisterName),
    IndirectHL,
}

impl std::fmt::Display for Operand3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand3::Register(r) => write!(f, "{}", r),
            Operand3::IndirectHL => write!(f, "[hl]"),
        }
    }
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

impl std::fmt::Display for Operand2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.register)
    }
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

impl std::fmt::Display for ConditionalOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionalOperand::NZ => write!(f, "nz"),
            ConditionalOperand::Z => write!(f, "z"),
            ConditionalOperand::NC => write!(f, "nc"),
            ConditionalOperand::C => write!(f, "c"),
        }
    }
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
    JumpImmediate | { address: u16 },
    JumpRelativeConditional { operand: ConditionalOperand } | { relative: i8 },
    XorRegister { operand: Operand3 },
    LoadIndirectHLToRegister8 { operand: Operand3 },
    LoadImmediateToRegister8 { operand: Operand3 } | { immediate: u8 },
    LoadImmediateToRegister16 { operand: Operand2 } | { immediate: u16 },
);

impl RawInstruction {
    pub fn new(opcode: u8) -> Result<Self, CpuError> {
        match opcode {
            0x00 => Ok(Self::Nop),
            0xC3 => Ok(Self::JumpImmediate),
            byte_permutations!("0b1010_1xxx") => {
                let idx = get_0000_0xxx(opcode);
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::XorRegister { operand })
            },
            byte_permutations!("0b0010_xx00") => {
                let idx = get_000x_x000(opcode);
                let operand = ConditionalOperand::new(idx).unwrap();
                Ok(Self::JumpRelativeConditional { operand })
            },
            byte_permutations!("0b01xx_x110") => {
                let idx = get_00xx_x000(opcode);
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::LoadIndirectHLToRegister8 { operand })
            },
            byte_permutations!("0b00xx_x110") => {
                let idx = get_00xx_x000(opcode);
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::LoadImmediateToRegister8 { operand })
            },
            byte_permutations!("0b00xx_0001") => {
                let idx = get_00xx_0000(opcode);
                let operand = Operand2::new(idx, LastOperand2::SP).unwrap();
                Ok(Self::LoadImmediateToRegister16 { operand })
            },
            _ => Err(CpuError::InvalidInstruction)
        }
    }
}

fn get_00xx_x000(byte: u8) -> u8 {
    (byte >> 3) & 0b111
}

fn get_00xx_0000(byte: u8) -> u8 {
    (byte >> 4) & 0b11
}

fn get_000x_x000(byte: u8) -> u8 {
    (byte >> 3) & 0b11
}

fn get_0000_0xxx(byte: u8) -> u8 {
    byte & 0b111
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Nop => write!(f, "nop"),
            Instruction::JumpImmediate { address } => write!(f, "jp {:#x}", address),
            Instruction::JumpRelativeConditional { operand, relative } => write!(f, "jr {}, {:+}", operand, relative),
            Instruction::XorRegister { operand } => write!(f, "xor {}", operand),
            Instruction::LoadIndirectHLToRegister8 { operand } => write!(f, "ld {}, [hl]", operand),
            Instruction::LoadImmediateToRegister8 { operand, immediate } => write!(f, "ld {}, {:#x}", operand, immediate),
            Instruction::LoadImmediateToRegister16 { operand, immediate } => write!(f, "ld {}, {:#x}", operand, immediate),
        }
    }
}

