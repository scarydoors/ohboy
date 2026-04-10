use bitflags::{Flags, bitflags};

use crate::{mbc, memory::{self, Memory, ReadMemory, WriteMemory}, rom};

pub struct Cpu {
    registers: Registers,
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
            println!("reading opcode at {:x}", self.registers.pc());
            let opcode = self.read_at_pc_then_inc();
            match opcode {
                0x00 => { // nop

                },
                0xC3 => {
                    let addr_lsb = self.read_at_pc_then_inc() as u16;
                    let addr_msb = self.read_at_pc_then_inc() as u16;

                    let addr = (addr_msb << 8) | addr_lsb;
                    println!("jump addr: {:x}", addr);
                    self.registers.set_pc(addr);
                },
                op if (op & 0b1010_1000) == (0b1010_1000) => {
                    println!("{}", (op & 0b111));
                    let operand = self.registers.get_8_bit((opcode & 0b111) as usize).unwrap();
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
                    
                    let 

                },
                _ => {
                    unimplemented!("unknown opcode {:x}!", opcode)
                }
            }
        }
    }

    pub fn read_at_pc_then_inc(&mut self) -> u8 {
        let pc = self.registers.pc();
        self.registers.set_pc(pc + 1);

        self.memory.read_memory_u8(pc as usize)
    }
}

struct Registers {
    pc: u16,
    sp: u16,

    a: u8,
    f: CpuFlagRegister,

    b: u8,
    c: u8,

    d: u8,
    e: u8,

    h: u8,
    l: u8
}

impl Registers {
    pub fn new() -> Self {
        // initialize the registers using DMG values (from TCAGBD doc)
        Self {
            pc: 0x0100,
            sp: 0xFFFE,

            a: 0x01,
            f: CpuFlagRegister::ZERO_FLAG | CpuFlagRegister::SUB_FLAG | CpuFlagRegister::CARRY_FLAG,

            b: 0x00,
            c: 0x13,

            d: 0x00,
            e: 0xD8,

            h: 0x01,
            l: 0x4D,
        }
    }

    fn pc(&self) -> u16 {
        self.pc
    }

    fn sp(&self) -> u16 {
        self.sp
    }

    fn a(&self) -> u8 {
        self.a
    }

    fn f(&self) -> &CpuFlagRegister {
        &self.f
    }

    fn af(&self) -> u16 {
        ((self.a as u16) << 8) | self.f.bits() as u16
    }

    fn b(&self) -> u8 {
        self.b
    }

    fn c(&self) -> u8 {
        self.c
    }

    fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    fn d(&self) -> u8 {
        self.d
    }

    fn e(&self) -> u8 {
        self.e
    }

    fn de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    fn h(&self) -> u8 {
        self.h
    }

    fn l(&self) -> u8 {
        self.l
    }

    fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
    }

    fn set_sp(&mut self, sp: u16) {
        self.sp = sp;
    }

    fn set_a(&mut self, a: u8) {
        self.a = a;
    }

    fn set_f(&mut self, f: CpuFlagRegister) {
        self.f = f;
    }

    fn set_b(&mut self, b: u8) {
        self.b = b;
    }

    fn set_c(&mut self, c: u8) {
        self.c = c;
    }

    fn set_d(&mut self, d: u8) {
        self.d = d;
    }

    fn set_e(&mut self, e: u8) {
        self.e = e;
    }

    fn set_h(&mut self, h: u8) {
        self.h = h;
    }

    fn set_l(&mut self, l: u8) {
        self.l = l;
    }

    fn get_8_bit(&self, idx: usize) -> Option<u8> {
        // TODO: validate idx is valid with bitmask
        if idx == 7 {
            Some(self.a())
        } else {
            [
                self.b(),
                self.c(),
                self.d(),
                self.e(),
                self.h(),
                self.l(),
                self.a(),
            ]
                .get(idx)
                .copied()
        }
    }

    fn get_16_bit(&self) -> Option<u16> {
    }


    fn set(&mut self) {
    }
}

bitflags! {
    struct CpuFlagRegister: u8 {
        const ZERO_FLAG = 1 << 7;
        const SUB_FLAG = 1 << 6;
        const HALF_CARRY_FLAG = 1 << 5;
        const CARRY_FLAG = 1 << 4;
    }
}
