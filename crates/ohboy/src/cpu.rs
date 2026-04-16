use bitflags::{Flags, bitflags};

use crate::{cpu::{instructions::{ConditionalOperand, Instruction, Operand3, RawInstruction}, register::{CpuFlagRegister, Registers, WordRegisterRead, WordRegisterWrite}}, mbc, memory::{self, Memory, ReadMemory, WriteMemory}, rom};

//PC 0x2817 is when tiles are loaded probably
mod register;
mod instructions;


pub enum CpuError {
    InvalidInstruction,
}

pub struct Cpu {
    registers: register::Registers,
    rom: rom::Rom,
    memory: memory::Memory,

    enable_interrupts: bool,
}

impl Cpu {
    pub fn new(rom: rom::Rom) -> Self {
        let mbc = mbc::create_mbc(rom.clone());
        Self {
            memory: Memory::new(mbc),
            registers: Registers::new(),
            rom: rom,

            enable_interrupts: true,
        }
    }

    pub fn run(&mut self) {
        loop {
            let (m_cycles, instruction) = self.cycle();
            println!("{}", instruction);
        }
    }

    pub fn cycle(&mut self) -> (MachineCycle, Instruction) {
        //println!("reading opcode at {:x}", self.registers.pc().get());
        let opcode = self.consume_pc_u8();
        if let Ok(i) = RawInstruction::new(opcode) {
            match i {
                RawInstruction::Nop => {
                    (MachineCycle(1), Instruction::Nop)
                },
                RawInstruction::JumpImmediate => {
                    let address = self.consume_pc_u16();

                    self.registers.pc_mut().set(address);
                    (MachineCycle(4), Instruction::JumpImmediate { address })
                },
                RawInstruction::JumpRelativeConditional { operand } => {
                    let relative = self.consume_pc_i8();
                    let machine_cycle = if self.check_condition(operand) {
                        self.registers.pc_mut().update(|pc| pc.wrapping_add_signed(relative as i16));
                        MachineCycle(3)
                    } else {
                        MachineCycle(2)
                    };

                    (machine_cycle, Instruction::JumpRelativeConditional { operand, relative })
                },
                RawInstruction::XorRegister { operand } => {
                    let value = match operand {
                        Operand3::Register(r) => self.registers.get_short_register(r).get_u8(),
                        Operand3::IndirectHL => self.memory.read_memory(self.registers.hl().get_u16().into()),
                    };

                    let result = self.registers.a_mut().update(|a| a ^ value);

                    if result == 0 {
                        self.registers.f_mut().set(CpuFlagRegister::ZERO_FLAG);
                    }

                    let machine_cycle = match operand {
                        Operand3::Register(_) => MachineCycle(2),
                        Operand3::IndirectHL => MachineCycle(3),
                    };

                    (machine_cycle, Instruction::XorRegister { operand })
                },
                RawInstruction::DecRegister { operand } => {
                    let SubCarryResult { result, half_carry, .. } = match operand {
                        Operand3::Register(r) => {
                            let register = self.registers.get_short_register_mut(r);
                            let result = sub_carry(register.get_u8(), 1);
                            register.set_u8(result.result);

                            result
                        },
                        Operand3::IndirectHL => {
                            let address = self.registers.hl().get_u16();
                            let result = sub_carry(self.memory.read_memory(address), 1);
                            self.memory.write_memory(address, result.result);

                            result
                        },
                    };
                    
                    self.registers.f_mut().update(|mut f| {
                        f.insert(CpuFlagRegister::SUB_FLAG);
                        if result == 0 {
                            f.insert(CpuFlagRegister::ZERO_FLAG);
                        } else {
                            f.remove(CpuFlagRegister::ZERO_FLAG);
                        }

                        if half_carry {
                            f.insert(CpuFlagRegister::HALF_CARRY_FLAG);
                        } else {
                            f.remove(CpuFlagRegister::HALF_CARRY_FLAG);
                        }
                        f
                    });

                    (MachineCycle(1), Instruction::DecRegister { operand })
                },
                RawInstruction::LoadAccumulatorToIndirect { operand } => {
                    let mut register = self.registers.get_word_register_mut(operand.register());
                    let address = match operand {
                        instructions::MemoryOperand::HLInc => {
                            let address = register.get_u16();
                            register.update_u16(&|hl| hl + 1);
                            address
                        },
                        instructions::MemoryOperand::HLDec => {
                            let address = register.get_u16();
                            register.update_u16(&|hl| hl - 1);
                            address
                        },
                        _ => register.get_u16(),
                    };
                    let a = self.registers.a().get();
                    self.memory.write_memory(address, a);
                    (MachineCycle(2), Instruction::LoadAccumulatorToIndirect { operand })
                },
                RawInstruction::LoadIndirectHLToRegister8 { operand } => {
                    let address = self.registers.hl().get_u16();
                    let val = self.memory.read_memory(address);
                    match operand {
                        Operand3::Register(r) => self.registers.get_short_register_mut(r).set_u8(val),
                        Operand3::IndirectHL => self.memory.write_memory(address, val),
                    };

                    (MachineCycle(2), Instruction::LoadIndirectHLToRegister8 { operand })
                },
                RawInstruction::LoadImmediateToRegister8 { operand } => {
                    let immediate = self.consume_pc_u8();
                    match operand {
                        Operand3::Register(r) => self.registers.get_short_register_mut(r).set_u8(immediate),
                        Operand3::IndirectHL => {
                            let address = self.registers.hl().get_u16();
                            self.memory.write_memory(address, immediate);
                        }
                    }
                    (MachineCycle(2), Instruction::LoadImmediateToRegister8 { operand, immediate })
                },
                RawInstruction::LoadImmediateToRegister16 { operand } => {
                    let immediate = self.consume_pc_u16();
                    self.registers.get_word_register_mut(operand.register).set_u16(immediate);
                    (MachineCycle(3), Instruction::LoadImmediateToRegister16 { operand, immediate })
                },
            }
        } else {
            unimplemented!("unknown opcode {:x}!", opcode)
        }
    }

    fn check_condition(&self, operand: ConditionalOperand) -> bool {
        let flags = self.registers.f().get();
        match operand {
            ConditionalOperand::NZ => !flags.contains(CpuFlagRegister::ZERO_FLAG),
            ConditionalOperand::Z => flags.contains(CpuFlagRegister::ZERO_FLAG),
            ConditionalOperand::NC => !flags.contains(CpuFlagRegister::CARRY_FLAG),
            ConditionalOperand::C => flags.contains(CpuFlagRegister::CARRY_FLAG),
        }
    }

    fn consume_pc_u8(&mut self) -> u8 {
        let pc = self.registers.pc_mut();
        let cur_pc = pc.get();
        pc.set(cur_pc + 1);

        self.memory.read_memory(cur_pc)
    }

    fn consume_pc_u16(&mut self) -> u16 {
        let lsb = self.consume_pc_u8();
        let msb = self.consume_pc_u8();

        u16_le(lsb, msb)
    }

    fn consume_pc_i8(&mut self) -> i8 {
        self.consume_pc_u8() as i8
    }
}

fn u16_le(lsb: u8, msb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

pub struct TimeCycle(usize);

impl From<MachineCycle> for TimeCycle {
    fn from(value: MachineCycle) -> Self {
        Self(value.0 * 4)
    }
}

pub struct MachineCycle(usize);


struct SubCarryResult { result: u8, carry: bool, half_carry: bool }
fn sub_carry(a: u8, b: u8) -> SubCarryResult {
    let (result, carry) = a.overflowing_sub(b);
    let half_carry = (a & 0xF) < (b & 0xF);

    return SubCarryResult { result, carry, half_carry }
}
