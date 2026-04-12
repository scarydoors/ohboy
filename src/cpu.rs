use bitflags::{Flags, bitflags};

use crate::{cpu::{instructions::{Instruction, Operand3}, register::{CpuFlagRegister, Registers, WordRegisterRead, WordRegisterWrite}}, mbc, memory::{self, Memory, ReadMemory, WriteMemory}, rom};

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
            println!("reading opcode at {:x}", self.registers.pc().get());
            let opcode = self.consume_pc_byte();
            if let Ok(i) = Instruction::new(opcode) {
                match i {
                    Instruction::Nop => {
                        // nop
                    },
                    Instruction::JumpRegister => {
                        let addr = self.consume_pc_word();

                        println!("jump addr: {:x}", addr);
                        self.registers.pc_mut().set(addr);
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
                    },
                    Instruction::LoadImmediate16 { operand } => {
                        let val = self.consume_pc_word();
                        self.registers.get_word_register_mut(operand.register).set_u16(val);
                    },
                }
            } else {
                unimplemented!("unknown opcode {:x}!", opcode)
            }
        }
    }

    fn consume_pc_byte(&mut self) -> u8 {
        let pc = self.registers.pc_mut();
        let cur_pc = pc.get();
        pc.set(cur_pc + 1);

        self.memory.read_memory_u8(pc.get() as usize)
    }

    fn consume_pc_word(&mut self) -> u16 {
        let lsb = self.consume_pc_byte();
        let msb = self.consume_pc_byte();

        u16_le(lsb, msb)
    }
}

fn u16_le(lsb: u8, msb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

