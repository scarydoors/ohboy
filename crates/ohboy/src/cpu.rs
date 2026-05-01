use bitflags::{Flags, bitflags};

use crate::{cpu::{instructions::{ConditionalOperand, Instruction, Operand3, RawInstruction}, register::{CpuFlags, Registers, WordRegisterRead, WordRegisterWrite}}, emulator::MachineCycle, mbc, memory::{self, Memory, ReadMemory, WriteMemory}, rom};

//PC 0x2817 is when tiles are loaded probably
pub mod register;
pub mod interrupt;
mod instructions;


pub enum CpuError {
    InvalidInstruction,
}

pub struct Cpu {
    pub registers: register::Registers,

    interrupt_master_enable: bool,
    pending_interrupt_enable: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            interrupt_master_enable: false,
            pending_interrupt_enable: false,
        }
    }

    pub fn cycle(&mut self, memory: &mut Memory) -> (MachineCycle, Instruction) {
        //println!("reading opcode at {:x}", self.registers.pc().get());
        if self.registers.pc().get() == 0x0fe2 {
            panic!("should have tiles");
        }
        let opcode = self.consume_pc_u8(memory);
        if let Ok(i) = RawInstruction::new(opcode) {
            match i {
                RawInstruction::Nop => {
                    (MachineCycle(1), Instruction::Nop)
                },
                RawInstruction::JumpImmediate => {
                    let address = self.consume_pc_u16(memory);

                    self.registers.pc_mut().set(address);
                    (MachineCycle(4), Instruction::JumpImmediate { address })
                },
                RawInstruction::DisableInterrupts => {
                    self.interrupt_master_enable = false;
                    self.pending_interrupt_enable = false;
                    (MachineCycle(1), Instruction::DisableInterrupts)
                },
                RawInstruction::JumpRelativeConditional { operand } => {
                    let relative = self.consume_pc_i8(memory);
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
                        Operand3::IndirectHL => memory.read_memory(self.registers.hl().get_u16().into()),
                    };

                    let result = self.registers.a_mut().update(|a| a ^ value);

                    if result == 0 {
                        self.registers.f_mut().set(CpuFlags::ZERO);
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
                            let result = sub_carry(memory.read_memory(address), 1);
                            memory.write_memory(address, result.result);

                            result
                        },
                    };
                    
                    self.registers.f_mut().update(|mut f| {
                        f.insert(CpuFlags::SUB);
                        f.set(CpuFlags::ZERO, result == 0);
                        f.set(CpuFlags::HALF_CARRY, half_carry);
                        f
                    });

                    (MachineCycle(1), Instruction::DecRegister { operand })
                },
                RawInstruction::LoadAccumulatorToIndirect { operand } => {
                    let mut register = self.registers.get_word_register_mut(operand.register());
                    let address = match operand {
                        instructions::IndirectOperand::HLInc => {
                            let address = register.get_u16();
                            register.update_u16(&|hl| hl + 1);
                            address
                        },
                        instructions::IndirectOperand::HLDec => {
                            let address = register.get_u16();
                            register.update_u16(&|hl| hl - 1);
                            address
                        },
                        _ => register.get_u16(),
                    };
                    let a = self.registers.a().get();
                    memory.write_memory(address, a);
                    (MachineCycle(2), Instruction::LoadAccumulatorToIndirect { operand })
                },
                RawInstruction::LoadIndirectHLToRegister8 { operand } => {
                    let address = self.registers.hl().get_u16();
                    let val = memory.read_memory(address);
                    match operand {
                        Operand3::Register(r) => self.registers.get_short_register_mut(r).set_u8(val),
                        Operand3::IndirectHL => memory.write_memory(address, val),
                    };

                    (MachineCycle(2), Instruction::LoadIndirectHLToRegister8 { operand })
                },
                RawInstruction::LoadImmediateToRegister8 { operand } => {
                    let immediate = self.consume_pc_u8(memory);
                    match operand {
                        Operand3::Register(r) => self.registers.get_short_register_mut(r).set_u8(immediate),
                        Operand3::IndirectHL => {
                            let address = self.registers.hl().get_u16();
                            memory.write_memory(address, immediate);
                        }
                    }
                    (MachineCycle(2), Instruction::LoadImmediateToRegister8 { operand, immediate })
                },
                RawInstruction::LoadImmediateToRegister16 { operand } => {
                    let immediate = self.consume_pc_u16(memory);
                    self.registers.get_word_register_mut(operand.register).set_u16(immediate);
                    (MachineCycle(3), Instruction::LoadImmediateToRegister16 { operand, immediate })
                },
                RawInstruction::LoadAccumulatorToHighMemory => {
                    let immediate = self.consume_pc_u8(memory);

                    memory.write_memory(high_address(immediate), self.registers.a().get());

                    (MachineCycle(3), Instruction::LoadAccumulatorToHighMemory { immediate })
                },
                RawInstruction::LoadAccumulatorToMemory => {
                    let immediate = self.consume_pc_u16(memory);

                    let a = self.registers.a().get();
                    memory.write_memory(immediate, a);

                    (MachineCycle(4), Instruction::LoadAccumulatorToMemory { immediate })
                },
                RawInstruction::LoadHighMemoryToAccumulator => {
                    let immediate = self.consume_pc_u8(memory);

                    self.registers.a_mut().set(memory.read_memory(high_address(immediate)));
                    (MachineCycle(3), Instruction::LoadHighMemoryToAccumulator { immediate })
                },
                RawInstruction::CompareImmediate => {
                    let immediate = self.consume_pc_u8(memory);
                    let a = self.registers.a().get();
                    let SubCarryResult { result, carry, half_carry } = sub_carry(a, immediate);

                    self.registers.f_mut().update(|mut f| {
                        f.insert(CpuFlags::SUB);
                        f.set(CpuFlags::ZERO, result == 0);
                        f.set(CpuFlags::HALF_CARRY, half_carry);
                        f.set(CpuFlags::CARRY, carry);
                        f
                    });

                    (MachineCycle(2), Instruction::CompareImmediate { immediate })
                }
                i => unimplemented!("unsupported instruction {:?}", i)
            }
        } else {
            panic!("{:?}\n{:?}", memory.oam, memory.vram);
            unimplemented!("unknown opcode {:x}!", opcode)
        }
    }

    fn check_condition(&self, operand: ConditionalOperand) -> bool {
        let flags = self.registers.f().get();
        match operand {
            ConditionalOperand::NZ => !flags.contains(CpuFlags::ZERO),
            ConditionalOperand::Z => flags.contains(CpuFlags::ZERO),
            ConditionalOperand::NC => !flags.contains(CpuFlags::CARRY),
            ConditionalOperand::C => flags.contains(CpuFlags::CARRY),
        }
    }

    fn consume_pc_u8(&mut self, memory: &mut Memory) -> u8 {
        let pc = self.registers.pc_mut();
        let cur_pc = pc.get();
        pc.set(cur_pc.wrapping_add(1));

        memory.read_memory(cur_pc)
    }

    fn consume_pc_u16(&mut self, memory: &mut Memory) -> u16 {
        let lsb = self.consume_pc_u8(memory);
        let msb = self.consume_pc_u8(memory);

        u16_le(lsb, msb)
    }

    fn consume_pc_i8(&mut self, memory: &mut Memory) -> i8 {
        self.consume_pc_u8(memory) as i8
    }
}

fn u16_le(lsb: u8, msb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}


struct SubCarryResult { result: u8, carry: bool, half_carry: bool }
fn sub_carry(a: u8, b: u8) -> SubCarryResult {
    let (result, carry) = a.overflowing_sub(b);
    let half_carry = (a & 0xF) < (b & 0xF);

    return SubCarryResult { result, carry, half_carry }
}

fn high_address(low: u8) -> u16 {
    0xFF00 | (low as u16)
}
