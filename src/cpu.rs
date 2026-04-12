use bitflags::{Flags, bitflags};

use crate::{cpu::{instructions::{ConditionalOperand, Instruction, Operand3}, register::{CpuFlagRegister, Registers, WordRegisterRead, WordRegisterWrite}}, mbc, memory::{self, Memory, ReadMemory, WriteMemory}, rom};

//PC 036C is when tiles are loaded probably
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
            let m_cycles = self.cycle();
        }
    }

    pub fn cycle(&mut self) -> MachineCycle {
        println!("reading opcode at {:x}", self.registers.pc().get());
        let opcode = self.consume_pc_u8();
        if let Ok(i) = Instruction::new(opcode) {
            match i {
                Instruction::Nop => {
                    MachineCycle(1)
                },
                Instruction::JumpRegister => {
                    let addr = self.consume_pc_u16();

                    println!("jumping to {:x}", addr);
                    self.registers.pc_mut().set(addr);
                    MachineCycle(4)
                },
                Instruction::JumpRelativeConditional { operand } => {
                    let relative = self.consume_pc_i8();
                    if self.check_condition(operand) {
                        self.registers.pc_mut().update(|pc| pc.wrapping_add_signed(relative as i16));
                        MachineCycle(3)
                    } else {
                        MachineCycle(2)
                    }
                },
                Instruction::XorRegister { operand } => {
                    let value = match operand {
                        Operand3::Register(r) => self.registers.get_short_register(r).get_u8(),
                        Operand3::IndirectHL => self.memory.read_memory_u8(self.registers.hl().get_u16().into()),
                    };

                    let result = self.registers.a_mut().update(|a| a ^ value);

                    if result == 0 {
                        self.registers.f_mut().set(CpuFlagRegister::ZERO_FLAG);
                    }

                    match operand {
                        Operand3::Register(_) => MachineCycle(2),
                        Operand3::IndirectHL => MachineCycle(3),
                    }
                },
                Instruction::LoadIndirectHLToRegister8 { operand } => {
                    let hl_address = self.registers.hl().get_u16();
                    let hl_val = self.memory.read_memory_u8(hl_address);
                    match operand {
                        Operand3::Register(r) => self.registers.get_short_register_mut(r).set_u8(hl_val),
                        Operand3::IndirectHL => self.memory.write_memory_u8(hl_address, hl_val),
                    };

                    MachineCycle(2)
                },
                Instruction::LoadImmediateToRegister16 { operand } => {
                    let val = self.consume_pc_u16();
                    self.registers.get_word_register_mut(operand.register).set_u16(val);
                    MachineCycle(3)
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
