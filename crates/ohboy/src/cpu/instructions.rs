use crate::{cpu::{CpuError, register::{ShortRegisterName, WordRegisterName}}, emulator::MachineCycle};
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
    (@full $enum_name:ident [$($variant:tt)*]) => {
        #[derive(Clone, Copy, Debug)]
        pub enum $enum_name {
            $($variant)*
        }
    };
    (@full $enum_name:ident [$($variant:tt)*] $name:ident {$($raw_field:ident: $raw_type:ident),*} | {$($full_field:ident: $full_type:ident),*}, $($rest:tt)*) => {
        instructions!(@full $enum_name [$($variant)* $name {$($raw_field: $raw_type),*,$($full_field: $full_type),*},] $($rest)*);
    };

    (@full $enum_name:ident [$($variant:tt)*] $name:ident | {$($full_field:ident: $full_type:ident),*}, $($rest:tt)*) => {
        instructions!(@full $enum_name [$($variant)* $name {$($full_field: $full_type),*},] $($rest)*);
    };

    (@full $enum_name:ident [$($variant:tt)*] $name:ident {$($raw_field:ident: $raw_type:ident),*}, $($rest:tt)*) => {
        instructions!(@full $enum_name [$($variant)* $name {$($raw_field: $raw_type),*},] $($rest)*);
    };

    (@full $enum_name:ident [$($variant:tt)*] $name:ident, $($rest:tt)*) => {
        instructions!(@full $enum_name [$($variant)* $name,] $($rest)*);
    };

    (@raw $enum_name:ident [$($variant:tt)*]) => {
        #[derive(Clone, Copy, Debug)]
        pub enum $enum_name {
            $($variant)*
        }
    };
    (@raw $enum_name:ident [$($variant:tt)*] $name:ident $({$($raw_field:ident: $raw_type:ident),*})? $(| {$($_ignored:tt)*})?, $($rest:tt)*) => {
        instructions!(@raw $enum_name [$($variant)* $name $({ $($raw_field: $raw_type),* })?,] $($rest)*);
    };

    (enums: {raw: $raw:ident, full: $full:ident, }, instructions: {$($rest:tt)*},) => {
        instructions!(@full $full [] $($rest)*);
        instructions!(@raw $raw [] $($rest)*);
    };
}

instructions!(
    enums: {
        raw: RawInstruction,
        full: Instruction,
    },
    instructions: {
        Nop,
        EnableInterrupts,
        DisableInterrupts,
        Halt,
        CallFunction | { address: u16 },
        ReturnFunction,
        PopStackToRegister { operand: Operand2 },
        PushRegisterToStack { operand: Operand2 },
        JumpImmediate | { address: u16 },
        JumpHL,
        JumpRelativeConditional { operand: ConditionalOperand } | { relative: i8 },
        XorRegister { operand: Operand3 },
        IncRegister8 { operand: Operand3 },
        IncRegister16 { operand: Operand2 },
        DecRegister8 { operand: Operand3 },
        DecRegister16 { operand: Operand2 },
        LoadRegisterToRegister { left_operand: Operand3, right_operand: Operand3 },
        LoadAccumulatorToIndirect { operand: IndirectOperand },
        LoadIndirectToAccumulator { operand: IndirectOperand },
        LoadImmediateToRegister8 { operand: Operand3 } | { immediate: u8 },
        LoadImmediateToRegister16 { operand: Operand2 } | { immediate: u16 },
        LoadAccumulatorToHighMemory | { immediate: u8 },
        LoadHighMemoryToAccumulator | { immediate: u8 },
        LoadAccumulatorToIndirectC,
        LoadAccumulatorToMemory | { immediate: u16 },
        CompareImmediate | { immediate: u8 },
        AddRegister { operand: Operand3 },
        BitwiseAndImmediate | { immediate: u8 },
        BitwiseOrRegister { operand: Operand3 },
        BitwiseAndRegister { operand: Operand3 },
        ComplementAccumulator,
        Restart { address: u8 },
        AddRegisterToHL { operand: Operand2 },
        CBPrefix,
    },
);

instructions!(
    enums: {
        raw: RawCBInstruction,
        full: CBInstruction,
    },
    instructions: {
        SwapNibbles { operand: Operand3 },
    },
);

impl RawInstruction {
    pub fn new(opcode: u8) -> Result<Self, CpuError> {
        match opcode {
            0x00 => Ok(Self::Nop),
            0xFB => Ok(Self::EnableInterrupts),
            0xF3 => Ok(Self::DisableInterrupts),
            0xCD => Ok(Self::CallFunction),
            0xC9 => Ok(Self::ReturnFunction),
            byte_permutations!("0b11xx_0001") => {
                let idx = match_bits!(opcode, "0b11xx_0001");
                let operand = Operand2::new(idx, LastOperand2::AF).unwrap();
                Ok(Self::PopStackToRegister { operand })
            },
            byte_permutations!("0b11xx_0101") => {
                let idx = match_bits!(opcode, "0b11xx_0101");
                let operand = Operand2::new(idx, LastOperand2::AF).unwrap();
                Ok(Self::PushRegisterToStack { operand })
            },
            0xC3 => Ok(Self::JumpImmediate),
            0xE9 => Ok(Self::JumpHL),
            byte_permutations!("0b0010_xx00") => {
                let idx = match_bits!(opcode, "0b0010_xx00");
                let operand = ConditionalOperand::new(idx).unwrap();
                Ok(Self::JumpRelativeConditional { operand })
            },
            byte_permutations!("0b1010_1xxx") => {
                let idx = match_bits!(opcode, "0b1010_1xxx");
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::XorRegister { operand })
            },
            byte_permutations!("0b00xx_x100") => {
                let idx = match_bits!(opcode, "0b00xx_x100");
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::IncRegister8 { operand })
            },
            byte_permutations!("0b00xx_0011") => {
                let idx = match_bits!(opcode, "0b00xx_0011");
                let operand = Operand2::new(idx, LastOperand2::SP).unwrap();
                Ok(Self::IncRegister16 { operand })
            },
            byte_permutations!("0b00xx_x101") => {
                let idx = match_bits!(opcode, "0b00xx_x101");
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::DecRegister8 { operand })
            },
            byte_permutations!("0b00xx_1011") => {
                let idx = match_bits!(opcode, "0b00xx_1011");
                let operand = Operand2::new(idx, LastOperand2::SP).unwrap();
                Ok(Self::DecRegister16 { operand })
            },
            byte_permutations!("0b01xx_xxxx") => {
                let left_operand = {
                    let idx = match_bits!(opcode, "0b01xx_x000");
                    Operand3::new(idx).unwrap()
                };
                let right_operand = {
                    let idx = match_bits!(opcode, "0b0100_0xxx");
                    Operand3::new(idx).unwrap()
                };

                if let (Operand3::IndirectHL, Operand3::IndirectHL) = (left_operand, right_operand) {
                    Ok(Self::Halt)
                } else {
                    Ok(Self::LoadRegisterToRegister { left_operand, right_operand })
                }
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
            byte_permutations!("0b1000_0xxx") => {
                let idx = match_bits!(opcode, "0b1000_0xxx");
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::AddRegister { operand })
            }
            0xE6 => {
                Ok(Self::BitwiseAndImmediate)
            },
            byte_permutations!("0b1011_0xxx") => {
                let idx = match_bits!(opcode, "0b1011_0xxx");
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::BitwiseOrRegister { operand })
            },
            byte_permutations!("0b1010_0xxx") => {
                let idx = match_bits!(opcode, "0b1011_0xxx");
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::BitwiseAndRegister { operand })
            },
            0x2F => {
                Ok(Self::ComplementAccumulator)
            },
            byte_permutations!("0b11xxx111") => {
                let address_8th = match_bits!(opcode, "0b11xxx111");
                Ok(Self::Restart { address: address_8th * 8 })
            },
            byte_permutations!("0b00xx_1001") => {
                let idx = match_bits!(opcode, "0b00xx_1001");
                let operand = Operand2::new(idx, LastOperand2::SP).unwrap();
                Ok(Self::AddRegisterToHL { operand })
            }
            0xCB => {
                Ok(Self::CBPrefix)
            },
            _ => Err(CpuError::InvalidInstruction { opcode })
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Instruction::*;

        match self {
            Nop => write!(f, "nop"),
            Halt => write!(f, "halt"),
            EnableInterrupts => write!(f, "ei"),
            DisableInterrupts => write!(f, "di"),
            CallFunction { address } => write!(f, "call {:#x}", address),
            PopStackToRegister { operand } => write!(f, "pop {}", operand),
            PushRegisterToStack { operand } => write!(f, "push {}", operand),
            ReturnFunction => write!(f, "ret"),
            JumpImmediate { address } => write!(f, "jp {:#x}", address),
            JumpHL => write!(f, "jp hl"),
            JumpRelativeConditional { operand, relative } => write!(f, "jr {}, {:+}", operand, relative),
            XorRegister { operand } => write!(f, "xor {}", operand),
            IncRegister8 { operand } => write!(f, "inc {}", operand),
            IncRegister16 { operand } => write!(f, "inc {}", operand),
            DecRegister8 { operand } => write!(f, "dec {}", operand),
            DecRegister16 { operand } => write!(f, "dec {}", operand),
            LoadRegisterToRegister { left_operand, right_operand } => write!(f, "ld {}, {}", left_operand, right_operand),
            LoadAccumulatorToIndirect { operand } => write!(f, "ld {}, a", operand),
            LoadIndirectToAccumulator { operand } => write!(f, "ld a, {}", operand),
            LoadImmediateToRegister8 { operand, immediate } => write!(f, "ld {}, {:#x}", operand, immediate),
            LoadImmediateToRegister16 { operand, immediate } => write!(f, "ld {}, {:#x}", operand, immediate),
            LoadAccumulatorToHighMemory { immediate } => write!(f, "ldh {:#x}, a", immediate),
            LoadAccumulatorToIndirectC => write!(f, "ldh [c], a"),
            LoadHighMemoryToAccumulator { immediate } => write!(f, "ldh a, {:#x}", immediate),
            LoadAccumulatorToMemory { immediate } => write!(f, "ld {:#x}, a", immediate),
            CompareImmediate { immediate } => write!(f, "cp {:#x}", immediate),
            AddRegister { operand } => write!(f, "add {}", operand),
            BitwiseAndImmediate { immediate } => write!(f, "and {:#x}", immediate),
            BitwiseOrRegister { operand } => write!(f, "or {}", operand),
            BitwiseAndRegister { operand } => write!(f, "and {}", operand),
            ComplementAccumulator => write!(f, "cpl"),
            AddRegisterToHL { operand } => write!(f, "add hl, {}", operand),
            Restart { address } => write!(f, "rst {:#x}", address),
            _ => Err(std::fmt::Error)
        }
    }
}

impl RawCBInstruction {
    pub fn new(opcode: u8) -> Result<Self, CpuError> {
        match opcode {
            byte_permutations!("0b0011_0xxx") => {
                let idx = match_bits!(opcode, "0b0011_0xxx");
                let operand = Operand3::new(idx).unwrap();
                Ok(Self::SwapNibbles { operand })
            },
            _ => Err(CpuError::InvalidCBInstruction { opcode })
        }
    }
}

impl std::fmt::Display for CBInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CBInstruction::*;

        match self {
            SwapNibbles { operand } => write!(f, "swap {}", operand),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AnyInstruction {
    Instruction(Instruction),
    CBInstruction(CBInstruction),
}

impl From<CBInstruction> for AnyInstruction {
    fn from(v: CBInstruction) -> Self {
        Self::CBInstruction(v)
    }
}

impl From<Instruction> for AnyInstruction {
    fn from(v: Instruction) -> Self {
        Self::Instruction(v)
    }
}

impl std::fmt::Display for AnyInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyInstruction::Instruction(i) => write!(f, "{}", i),
            AnyInstruction::CBInstruction(cb) => write!(f, "{}", cb),
        }
    }
}
