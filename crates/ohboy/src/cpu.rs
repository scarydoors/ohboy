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
}

impl Cpu {
    pub fn new(rom: rom::Rom) -> Self {
        let mbc = mbc::create_mbc(rom.clone());
        Self {
            memory: Memory::new(mbc),
            registers: Registers::new(),
            rom: rom,
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
                        Operand3::IndirectHL => self.memory.read_memory_u8(self.registers.hl().get_u16().into()),
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
                RawInstruction::LoadIndirectHLToRegister8 { operand } => {
                    let address = self.registers.hl().get_u16();
                    let val = self.memory.read_memory_u8(address);
                    match operand {
                        Operand3::Register(r) => self.registers.get_short_register_mut(r).set_u8(val),
                        Operand3::IndirectHL => self.memory.write_memory_u8(address, val),
                    };

                    (MachineCycle(2), Instruction::LoadIndirectHLToRegister8 { operand })
                },
                RawInstruction::LoadImmediateToRegister8 { operand } => {
                    let immediate = self.consume_pc_u8();
                    match operand {
                        Operand3::Register(r) => self.registers.get_short_register_mut(r).set_u8(immediate),
                        Operand3::IndirectHL => {
                            let address = self.registers.hl().get_u16();
                            self.memory.write_memory_u8(address, immediate);
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

        self.memory.read_memory_u8(cur_pc)
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
