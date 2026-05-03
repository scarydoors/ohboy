use core::panic;

use crate::{cpu::{instructions::{AnyInstruction, BitIndexOperand, CBInstruction, ConditionalOperand, IndirectOperand, Instruction, Operand3, RawCBInstruction, RawInstruction}, register::{ByteRegisterWrite, CpuFlags, Registers, WordRegisterRead, WordRegisterWrite}}, emulator::MachineCycle, mbc, memory::{self, Memory, ReadMemory, WriteMemory}, rom};

//PC 0x2817 is when tiles are loaded probably
pub mod register;
pub mod interrupt;
mod instructions;


#[derive(Debug, thiserror::Error)]
pub enum CpuError {
    #[error("unknown opcode: {opcode:#x}")]
    InvalidInstruction { opcode: u8 },

    #[error("unknown cb-prefixed opcode: {opcode:#x}")]
    InvalidCBInstruction { opcode: u8 },
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

    pub fn cycle(&mut self, memory: &mut Memory) -> Result<(MachineCycle, AnyInstruction), CpuError> {
        // if self.registers.pc().get() == 0x2817 {
        //     panic!("should have tiles:\noam: {:?}, vram: {:?}", memory.oam, memory.vram,);
        // }
        let opcode = self.consume_pc_u8(memory);
        let (machine_cycle, instruction) = self.execute(memory, opcode)?;
        Ok(match instruction {
            Instruction::CBPrefix => {
                let opcode = self.consume_pc_u8(memory);
                let (machine_cycle, cb_instruction) = self.execute_cb_prefix(memory, opcode)?;
                (machine_cycle, cb_instruction.into())
            }
            _ => {
                (machine_cycle, instruction.into())    
            }
        })
    }

    fn execute(&mut self, memory: &mut Memory, opcode: u8) -> Result<(MachineCycle, Instruction), CpuError> {
        Ok(
            match RawInstruction::new(opcode)? {
                RawInstruction::Nop => {
                    (MachineCycle(1), Instruction::Nop)
                },
                RawInstruction::JumpImmediate => {
                    let address = self.consume_pc_u16(memory);

                    self.registers.pc_mut().set(address);
                    (MachineCycle(4), Instruction::JumpImmediate { address })
                },
                RawInstruction::JumpHL => {
                    let address = self.registers.hl().get_u16();

                    self.registers.pc_mut().set(address);
                    (MachineCycle(1), Instruction::JumpHL)
                },
                RawInstruction::EnableInterrupts => {
                    // TODO: actually enable interrupts
                    self.pending_interrupt_enable = true;
                    (MachineCycle(1), Instruction::EnableInterrupts)
                },
                RawInstruction::DisableInterrupts => {
                    self.interrupt_master_enable = false;
                    self.pending_interrupt_enable = false;
                    (MachineCycle(1), Instruction::DisableInterrupts)
                },
                RawInstruction::CallFunction => {
                    let address = self.consume_pc_u16(memory);

                    self.push_stack(memory, self.registers.pc().get());
                    self.registers.pc_mut().set(address);

                    (MachineCycle(6), Instruction::CallFunction { address })
                },
                RawInstruction::PopStackToRegister { operand } => {
                    let popped = self.pop_stack(memory);
                    self.registers.get_word_register_mut(operand.register).set_u16(popped);

                    (MachineCycle(3), Instruction::PopStackToRegister { operand })
                },
                RawInstruction::PushRegisterToStack { operand } => {
                    let value = self.registers.get_word_register(operand.register).get_u16();
                    self.push_stack(memory, value);

                    (MachineCycle(4), Instruction::PushRegisterToStack { operand })
                },
                RawInstruction::ReturnFunction => {
                    let popped = self.pop_stack(memory);
                    self.registers.pc_mut().set(popped);

                    (MachineCycle(4), Instruction::ReturnFunction)
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
                RawInstruction::IncRegister8 { operand } => {
                    let AddCarryResult { result, half_carry, .. } = match operand {
                        Operand3::Register(r) => {
                            let register = self.registers.get_short_register_mut(r);
                            let result = add_carry_8(register.get_u8(), 1);
                            register.set_u8(result.result);

                            result
                        },
                        Operand3::IndirectHL => {
                            let address = self.registers.hl().get_u16();
                            let result = add_carry_8(memory.read_memory(address), 1);
                            memory.write_memory(address, result.result);

                            result
                        },
                    };
                    
                    self.registers.f_mut().update(|mut f| {
                        f.remove(CpuFlags::SUB);
                        f.set(CpuFlags::ZERO, result == 0);
                        f.set(CpuFlags::HALF_CARRY, half_carry);
                        f
                    });

                    let machine_cycle = match operand {
                        Operand3::Register(_) => MachineCycle(1),
                        Operand3::IndirectHL => MachineCycle(3),
                    };

                    (machine_cycle, Instruction::IncRegister8 { operand })
                },
                RawInstruction::IncRegister16 { operand } => {
                    let mut register = self.registers.get_word_register_mut(operand.register);
                    register.update_u16(&|r| {
                        r.wrapping_add(1)
                    });
                    (MachineCycle(2), Instruction::IncRegister16 { operand })
                },
                RawInstruction::DecRegister8 { operand } => {
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

                    (MachineCycle(1), Instruction::DecRegister8 { operand })
                },
                RawInstruction::DecRegister16 { operand } => {
                    let mut register = self.registers.get_word_register_mut(operand.register);
                    register.update_u16(&|r| {
                        r.wrapping_sub(1)
                    });
                    (MachineCycle(2), Instruction::DecRegister16 { operand })
                },
                RawInstruction::LoadRegisterToRegister { left_operand, right_operand } => {
                    let right = match right_operand {
                        Operand3::Register(r) => self.registers.get_short_register(r).get_u8(),
                        Operand3::IndirectHL => {
                            memory.read_memory(self.registers.hl().get_u16())
                        },
                    };

                    match left_operand {
                        Operand3::Register(r) => {
                            self.registers.get_short_register_mut(r).set_u8(right);
                        },
                        Operand3::IndirectHL => {
                            let address = self.registers.hl().get_u16();
                            memory.write_memory(address, right);
                        },
                    }

                    let machine_cycle = if matches!(left_operand, Operand3::IndirectHL) || matches!(right_operand, Operand3::IndirectHL) {
                        MachineCycle(2)
                    } else {
                        MachineCycle(1)
                    };

                    (machine_cycle, Instruction::LoadRegisterToRegister { left_operand, right_operand })
                }
                RawInstruction::LoadAccumulatorToIndirect { operand } => {
                    let address = self.get_indirect_address(operand);
                    let a = self.registers.a().get();
                    memory.write_memory(address, a);
                    (MachineCycle(2), Instruction::LoadAccumulatorToIndirect { operand })
                },
                RawInstruction::LoadIndirectToAccumulator { operand } => {
                    let address = self.get_indirect_address(operand);
                    self.registers.a_mut().set(memory.read_memory(address));
                    (MachineCycle(2), Instruction::LoadIndirectToAccumulator { operand })
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
                RawInstruction::LoadAccumulatorToIndirectC => {
                    let address = high_address(self.registers.c().get());
                    memory.write_memory(address, self.registers.a().get());

                    (MachineCycle(2), Instruction::LoadAccumulatorToIndirectC)
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
                },
                RawInstruction::AddRegister { operand } => {
                    let a = self.registers.a_mut().get();
                    let AddCarryResult { result, carry, half_carry } = match operand {
                        Operand3::Register(r) => {
                            let value = self.registers.get_short_register(r).get_u8();
                            let result = add_carry_8(value, a);

                            result
                        },
                        Operand3::IndirectHL => {
                            let address = self.registers.hl().get_u16();
                            let value = memory.read_memory(address);
                            let result = add_carry_8(value, a);

                            result
                        },
                    };
                    self.registers.a_mut().set(result);

                    self.registers.f_mut().update(|_| {
                        let mut f = CpuFlags::empty();
                        f.set(CpuFlags::ZERO, result == 0);
                        f.set(CpuFlags::HALF_CARRY, half_carry);
                        f.set(CpuFlags::CARRY, carry);
                        f
                    });

                    let machine_cycle = match operand {
                        Operand3::Register(_) => MachineCycle(1),
                        Operand3::IndirectHL => MachineCycle(2),
                    };

                    (machine_cycle, Instruction::AddRegister { operand })
                },
                RawInstruction::BitwiseAndImmediate => {
                    let immediate = self.consume_pc_u8(memory);

                    let result = self.registers.a_mut().update(|a| a & immediate);

                    self.registers.f_mut().update(|_| {
                        let mut z = CpuFlags::empty();
                        z.set(CpuFlags::ZERO, result == 0);
                        z.insert(CpuFlags::HALF_CARRY);
                        z
                    });

                    (MachineCycle(2), Instruction::BitwiseAndImmediate { immediate })
                }
                RawInstruction::BitwiseOrRegister { operand } => {
                    let value = match operand {
                        Operand3::Register(r) => self.registers.get_short_register(r).get_u8(),
                        Operand3::IndirectHL => memory.read_memory(self.registers.hl().get_u16().into()),
                    };
                    let result = self.registers.a_mut().update(|a| a | value);

                    self.registers.f_mut().update(|_| {
                        let mut z = CpuFlags::empty();
                        z.set(CpuFlags::ZERO, result == 0);
                        z
                    });

                    (MachineCycle(1), Instruction::BitwiseOrRegister { operand })
                },
                RawInstruction::BitwiseAndRegister { operand } => {
                    let value = match operand {
                        Operand3::Register(r) => self.registers.get_short_register(r).get_u8(),
                        Operand3::IndirectHL => memory.read_memory(self.registers.hl().get_u16().into()),
                    };
                    let result = self.registers.a_mut().update(|a| a & value);

                    self.registers.f_mut().update(|_| {
                        let mut z = CpuFlags::empty();
                        z.set(CpuFlags::ZERO, result == 0);
                        z.insert(CpuFlags::HALF_CARRY);
                        z
                    });

                    (MachineCycle(1), Instruction::BitwiseAndRegister { operand })
                },
                RawInstruction::ComplementAccumulator => {
                    self.registers.a_mut().update(|a| !a);
                    self.registers.f_mut().update(|mut f| {
                        f.insert(CpuFlags::SUB);
                        f.insert(CpuFlags::HALF_CARRY);
                        f
                    });
                    (MachineCycle(1), Instruction::ComplementAccumulator)
                },
                RawInstruction::AddRegisterToHL { operand } => {
                    let value =  self.registers.get_word_register(operand.register).get_u16();

                    let mut hl = self.registers.hl_mut();
                    let AddCarryResult { result, half_carry, carry } = add_carry_16(hl.get_u16(), value);
                    hl.set_u16(result);

                    self.registers.f_mut().update(|mut f| {
                        f.set(CpuFlags::HALF_CARRY, half_carry);
                        f.set(CpuFlags::CARRY, carry);
                        f.remove(CpuFlags::SUB);
                        f
                    });
                    (MachineCycle(2), Instruction::AddRegisterToHL { operand })
                },
                RawInstruction::Restart { address } => {
                    self.push_stack(memory, self.registers.pc().get());
                    self.registers.pc_mut().set(address.into());

                    (MachineCycle(4), Instruction::Restart { address })
                }
                RawInstruction::Halt => unimplemented!("halt"),
                RawInstruction::CBPrefix => (MachineCycle(0), Instruction::CBPrefix),
            }
        )
    }

    fn execute_cb_prefix(&mut self, memory: &mut Memory, opcode: u8) -> Result<(MachineCycle, CBInstruction), CpuError> {
        Ok(
            match RawCBInstruction::new(opcode)? {
                RawCBInstruction::RegisterResetBit { bit_operand, operand } => {
                    match operand {
                        Operand3::Register(r) => {
                            self.registers.get_short_register_mut(r).update_u8(&|r| {
                                reset_bit(r, bit_operand)
                            });
                        },
                        Operand3::IndirectHL => {
                            let address = self.registers.hl().get_u16();
                            let value = memory.read_memory(address);
                            memory.write_memory(address, reset_bit(value, bit_operand));
                        },
                    }

                    let machine_cycle = match operand {
                        Operand3::Register(_) => MachineCycle(2),
                        Operand3::IndirectHL => MachineCycle(4),
                    };

                    (machine_cycle, CBInstruction::RegisterResetBit { bit_operand, operand })
                },
                RawCBInstruction::SwapNibbles { operand } => {
                    let result = match operand {
                        Operand3::Register(r) => {
                            self.registers.get_short_register_mut(r).update_u8(&|r| {
                                swap_nibbles(r)
                            })
                        },
                        Operand3::IndirectHL => {
                            let address = self.registers.hl().get_u16();
                            let value = memory.read_memory(address);

                            let result = swap_nibbles(value);
                            memory.write_memory(address, result);
                            result
                        },
                    };

                    self.registers.f_mut().update(|_| {
                        let mut f = CpuFlags::empty();
                        f.set(CpuFlags::ZERO, result == 0);
                        f
                    });

                    (MachineCycle(2), CBInstruction::SwapNibbles { operand })
                },
            }
        )
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

    fn get_indirect_address(&mut self, indirect: IndirectOperand) -> u16 {
        let mut register = self.registers.get_word_register_mut(indirect.register());
        match indirect {
            instructions::IndirectOperand::HLInc => {
                let address = register.get_u16();
                register.update_u16(&|hl| hl.wrapping_add(1));
                address
            },
            instructions::IndirectOperand::HLDec => {
                let address = register.get_u16();
                register.update_u16(&|hl| hl.wrapping_sub(1));
                address
            },
            _ => register.get_u16(),
        }
    }

    fn push_stack(&mut self, memory: &mut Memory, value: u16) {
        let sp = self.registers.sp_mut();
        sp.update(|sp| sp.wrapping_sub(1));
        memory.write_memory(sp.get(), u16_msb(value));
        sp.update(|sp| sp.wrapping_sub(1));
        memory.write_memory(sp.get(), u16_lsb(value));
    }

    fn pop_stack(&mut self, memory: &mut Memory) -> u16 {
        let sp = self.registers.sp_mut();
       
        let lsb = memory.read_memory(sp.get());
        sp.update(|sp| sp.wrapping_add(1));
        let msb = memory.read_memory(sp.get());
        sp.update(|sp| sp.wrapping_add(1));

        u16_le(lsb, msb)
    }
}

fn u16_le(lsb: u8, msb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

fn u16_lsb(value: u16) -> u8 {
    value as u8
}

fn u16_msb(value: u16) -> u8 {
    (value >> 8) as u8
}

fn lower_nibble(value: u8) -> u8 {
    value & 0xF
}

fn upper_nibble(value: u8) -> u8 {
    (value >> 4) & 0xF
}

fn swap_nibbles(value: u8) -> u8 {
    let lower = lower_nibble(value);
    let upper = upper_nibble(value);

    lower << 4 | upper
}


struct SubCarryResult { result: u8, carry: bool, half_carry: bool }
fn sub_carry(a: u8, b: u8) -> SubCarryResult {
    let (result, carry) = a.overflowing_sub(b);
    return SubCarryResult { result, carry, half_carry: is_half_carry_8(a, b, result) }
}

struct AddCarryResult<T> { result: T, carry: bool, half_carry: bool }
fn add_carry_8(a: u8, b: u8) -> AddCarryResult<u8> {
    let (result, carry) = a.overflowing_add(b);
    return AddCarryResult { result, carry, half_carry: is_half_carry_8(a, b, result) }
}

fn add_carry_16(a: u16, b: u16) -> AddCarryResult<u16> {
    let (result, carry) = a.overflowing_add(b);
    return AddCarryResult { result, carry, half_carry: is_half_carry_16(a, b, result) }
}

fn is_half_carry_8(a: u8, b: u8, result: u8) -> bool {
    (a ^ b ^ result) & 0x10 == 0x10
}

fn is_half_carry_16(a: u16, b: u16, result: u16) -> bool {
    (a ^ b ^ result) & 0x1000 == 0x1000 
}

fn high_address(low: u8) -> u16 {
    0xFF00 | (low as u16)
}

fn reset_bit(value: u8, position: BitIndexOperand) -> u8 {
    value & !(1 << position.0)
}
