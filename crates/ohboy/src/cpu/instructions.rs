use crate::cpu::{CpuError, register::{ShortRegisterName, WordRegisterName}};
use ohboy_macro::{byte_permutations, match_bits};


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

#[derive(Debug, Copy, Clone)]
pub enum IndirectOperand {
    BC,
    DE,
    HLInc,
    HLDec,
}

impl IndirectOperand {
    fn new(idx: u8) -> Result<Self, OperandTooWide> {
        match idx {
            0 => Ok(Self::BC),
            1 => Ok(Self::DE),
            2 => Ok(Self::HLInc),
            3 => Ok(Self::HLDec),
            _ => Err(OperandTooWide)
        }
    }

    pub fn register(&self) -> WordRegisterName {
        match self {
            IndirectOperand::BC => WordRegisterName::BC,
            IndirectOperand::DE => WordRegisterName::DE,
            IndirectOperand::HLInc | IndirectOperand::HLDec => WordRegisterName::HL,
        }
    }
}

impl std::fmt::Display for IndirectOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndirectOperand::BC => write!(f, "[bc]"),
            IndirectOperand::DE => write!(f, "[de]"),
            IndirectOperand::HLInc => write!(f, "[hl+]"),
            IndirectOperand::HLDec => write!(f, "[hl-]"),
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
    DisableInterrupts,
    Halt,
    JumpImmediate | { address: u16 },
    JumpRelativeConditional { operand: ConditionalOperand } | { relative: i8 },
    XorRegister { operand: Operand3 },
    IncRegister { operand: Operand3 },
    DecRegister { operand: Operand3 },
    LoadAccumulatorToIndirect { operand: IndirectOperand },
    LoadIndirectToAccumulator { operand: IndirectOperand },
    LoadIndirectHLToRegister8 { operand: Operand3 },
    LoadImmediateToRegister8 { operand: Operand3 } | { immediate: u8 },
    LoadImmediateToRegister16 { operand: Operand2 } | { immediate: u16 },
    LoadAccumulatorToHighMemory | { immediate: u8 },
    LoadHighMemoryToAccumulator | { immediate: u8 },
    LoadAccumulatorToIndirectC,
    LoadAccumulatorToMemory | { immediate: u16 },
    CompareImmediate | { immediate: u8 },
);

impl RawInstruction {
    pub fn new(opcode: u8) -> Result<Self, CpuError> {
        match opcode {
            0x00 => Ok(Self::Nop),
            0xC3 => Ok(Self::JumpImmediate),
            0xF3 => Ok(Self::DisableInterrupts),
            byte_permutations!("0b1010_1xxx") => {
                let idx = match_bits!(opcode, "0b1010_1xxx");
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::XorRegister { operand })
            },
            byte_permutations!("0b00xx_x100") => {
                let idx = match_bits!(opcode, "0b00xx_x100");
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::IncRegister { operand })
            },
            byte_permutations!("0b00xx_x101") => {
                let idx = match_bits!(opcode, "0b00xx_x101");
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::DecRegister { operand })
            },
            byte_permutations!("0b0010_xx00") => {
                let idx = match_bits!(opcode, "0b0010_xx00");
                let operand = ConditionalOperand::new(idx).unwrap();
                Ok(Self::JumpRelativeConditional { operand })
            },
            byte_permutations!("0b00xx_0010") => {
                let idx = match_bits!(opcode, "0b00xx_0010");
                let operand = IndirectOperand::new(idx).unwrap();
                Ok(Self::LoadAccumulatorToIndirect { operand })
            },
            byte_permutations!("0b00xx_1010") => {
                let idx = match_bits!(opcode, "0b00xx_1010");
                let operand = IndirectOperand::new(idx).unwrap();
                Ok(Self::LoadIndirectToAccumulator { operand })
            },
            byte_permutations!("0b01xx_x110") => {
                let idx = match_bits!(opcode, "0b01xx_x110");
                let operand = Operand3::new(idx).unwrap();
                if let Operand3::IndirectHL = operand {
                    Ok(Self::Halt)
                } else {
                    Ok(Self::LoadIndirectHLToRegister8 { operand })
                }
            },
            byte_permutations!("0b00xx_x110") => {
                let idx = match_bits!(opcode, "0b00xx_x110");
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::LoadImmediateToRegister8 { operand })
            },
            byte_permutations!("0b00xx_0001") => {
                let idx = match_bits!(opcode, "0b00xx_0001");
                let operand = Operand2::new(idx, LastOperand2::SP).unwrap();
                Ok(Self::LoadImmediateToRegister16 { operand })
            },
            0xE0 => {
                Ok(Self::LoadAccumulatorToHighMemory)
            },
            0xE2 => {
                Ok(Self::LoadAccumulatorToIndirectC)
            },
            0xEA => {
                Ok(Self::LoadAccumulatorToMemory)
            },
            0xF0 => {
                Ok(Self::LoadHighMemoryToAccumulator)
            },
            0xFE => {
                Ok(Self::CompareImmediate)
            },
            _ => Err(CpuError::InvalidInstruction)
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Instruction::*;

        match self {
            Nop => write!(f, "nop"),
            Halt => write!(f, "halt"),
            DisableInterrupts => write!(f, "di"),
            JumpImmediate { address } => write!(f, "jp {:#x}", address),
            JumpRelativeConditional { operand, relative } => write!(f, "jr {}, {:+}", operand, relative),
            XorRegister { operand } => write!(f, "xor {}", operand),
            IncRegister { operand } => write!(f, "inc {}", operand),
            DecRegister { operand } => write!(f, "dec {}", operand),
            LoadAccumulatorToIndirect { operand } => write!(f, "ld {}, a", operand),
            LoadIndirectToAccumulator { operand } => write!(f, "ld a, {}", operand),
            LoadIndirectHLToRegister8 { operand } => write!(f, "ld {}, [hl]", operand),
            LoadImmediateToRegister8 { operand, immediate } => write!(f, "ld {}, {:#x}", operand, immediate),
            LoadImmediateToRegister16 { operand, immediate } => write!(f, "ld {}, {:#x}", operand, immediate),
            LoadAccumulatorToHighMemory { immediate } => write!(f, "ldh {:#x}, a", immediate),
            LoadAccumulatorToIndirectC => write!(f, "ldh [c], a"),
            LoadHighMemoryToAccumulator { immediate } => write!(f, "ldh a, {:#x}", immediate),
            LoadAccumulatorToMemory { immediate } => write!(f, "ld {:#x}, a", immediate),
            CompareImmediate { immediate } => write!(f, "cp {:#x}", immediate),
        }
    }
}

