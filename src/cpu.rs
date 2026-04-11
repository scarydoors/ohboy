use bitflags::{Flags, bitflags};

use crate::{cpu::{instructions::Instruction, register::Registers}, mbc, memory::{self, Memory, ReadMemory, WriteMemory}, rom};

mod register;
mod instructions;

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
            let opcode = self.read_at_pc_then_inc();
            if let Some(i) = Instruction::new(opcode) {
                match i {
                    Instruction::Nop => { // nop

                    },
                    Instruction::JumpRegister => {
                        let addr_lsb = self.read_at_pc_then_inc() as u16;
                        let addr_msb = self.read_at_pc_then_inc() as u16;

                        let addr = (addr_msb << 8) | addr_lsb;
                        println!("jump addr: {:x}", addr);
                        self.registers.pc().set(addr);
                    },
                    Instruction::XorRegister { register } => {
                        let operand = self.registers.
                        let a = self.registers.a();

                        let result = a ^ operand;
                        self.registers.set_a(result);

                        let mut flags = CpuFlagRegister::empty();
                        if result == 0 {
                            flags |= CpuFlagRegister::ZERO_FLAG;
                        }
                        self.registers.set_f(flags);
                    },
                    op if (op & 0b0000_0001) == (0b0000_0001) => {
                        println!("{}", (op & 0b111));
                        let operand = self.registers.get_8_bit(((opcode >> 4) & 0b11) as usize).unwrap();
                        let val_lsb = self.read_at_pc_then_inc() as u16;
                        let val_msb = self.read_at_pc_then_inc() as u16;

                        let val = (val_msb << 8) | val_lsb;
                    },
                }
            } else {
                unimplemented!("unknown opcode {:x}!", opcode)
            }
        }
    }

    pub fn read_at_pc_then_inc(&mut self) -> u8 {
        let pc = self.registers.pc_mut();
        let cur_pc = pc.get();
        pc.set(cur_pc + 1);

        self.memory.read_memory_u8(pc.get() as usize)
    }
}

