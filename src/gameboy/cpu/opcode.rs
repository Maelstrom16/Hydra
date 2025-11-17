use genawaiter::stack::Co;

use crate::{
    gameboy::{
        cpu::{self, CPU},
        memory::Memory,
    },
    gen_all,
};

pub trait IntOperand<T> {
    async fn get(&self, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) -> T;
    async fn set(&self, value: T, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>);
}

pub struct RegisterOperand8(pub cpu::Register8);
impl IntOperand<u8> for RegisterOperand8 {
    #[inline(always)]
    async fn get(&self, cpu: &mut CPU, _: &mut Memory, _co: Co<'_, ()>) -> u8 {
        match self.0 {
            cpu::Register8::A => cpu.af[1],
            cpu::Register8::F => cpu.af[0],
            cpu::Register8::B => cpu.bc[1],
            cpu::Register8::C => cpu.bc[0],
            cpu::Register8::D => cpu.de[1],
            cpu::Register8::E => cpu.de[0],
            cpu::Register8::H => cpu.hl[1],
            cpu::Register8::L => cpu.hl[0],
        }
    }
    #[inline(always)]
    async fn set(&self, value: u8, cpu: &mut CPU, _: &mut Memory, _co: Co<'_, ()>) {
        match self.0 {
            cpu::Register8::A => cpu.af[1] = value,
            cpu::Register8::F => cpu.af[0] = value,
            cpu::Register8::B => cpu.bc[1] = value,
            cpu::Register8::C => cpu.bc[0] = value,
            cpu::Register8::D => cpu.de[1] = value,
            cpu::Register8::E => cpu.de[0] = value,
            cpu::Register8::H => cpu.hl[1] = value,
            cpu::Register8::L => cpu.hl[0] = value,
        };
    }
}

pub struct ImmediateOperand8;
impl IntOperand<u8> for ImmediateOperand8 {
    #[inline(always)]
    async fn get(&self, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) -> u8 {
        gen_all!(&co, |co_inner| cpu.step_u8(memory, co_inner))
    }
    #[inline(always)]
    async fn set(&self, _: u8, _: &mut CPU, _: &mut Memory, _co: Co<'_, ()>) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct ImmediateSignedOperand8;
impl IntOperand<i8> for ImmediateSignedOperand8 {
    #[inline(always)]
    async fn get(&self, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) -> i8 {
        gen_all!(&co, |co_inner| cpu.step_u8(memory, co_inner)) as i8
    }
    #[inline(always)]
    async fn set(&self, _: i8, _: &mut CPU, _: &mut Memory, _co: Co<'_, ()>) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct IndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for IndirectOperand8<O> {
    #[inline(always)]
    async fn get(&self, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) -> u8 {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.read_u8(address, memory, co_inner))
    }
    #[inline(always)]
    async fn set(&self, value: u8, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.write_u8(address, value, memory, co_inner));
    }
}
pub struct IncIndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for IncIndirectOperand8<O> {
    #[inline(always)]
    async fn get(&self, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) -> u8 {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        gen_all!(co, |co_inner| self.0.set(address + 1, cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.read_u8(address, memory, co_inner))
    }
    #[inline(always)]
    async fn set(&self, value: u8, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        gen_all!(co, |co_inner| self.0.set(address + 1, cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.write_u8(address, value, memory, co_inner));
    }
}
pub struct DecIndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for DecIndirectOperand8<O> {
    #[inline(always)]
    async fn get(&self, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) -> u8 {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        gen_all!(co, |co_inner| self.0.set(address - 1, cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.read_u8(address, memory, co_inner))
    }
    #[inline(always)]
    async fn set(&self, value: u8, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        gen_all!(co, |co_inner| self.0.set(address - 1, cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.write_u8(address, value, memory, co_inner));
    }
}

pub struct HramIndirectOperand<O: IntOperand<u8>>(pub O);
impl<O: IntOperand<u8>> HramIndirectOperand<O> {
    #[inline(always)]
    async fn as_hram_address(&self, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) -> u16 {
        0xFF00 | (gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner)) as u16)
    }
}
impl<O: IntOperand<u8>> IntOperand<u8> for HramIndirectOperand<O> {
    #[inline(always)]
    async fn get(&self, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) -> u8 {
        let hram_address = gen_all!(co, |co_inner| self.as_hram_address(cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.read_u8(hram_address, memory, co_inner))
    }
    #[inline(always)]
    async fn set(&self, value: u8, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) {
        let hram_address = gen_all!(co, |co_inner| self.as_hram_address(cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.write_u8(hram_address, value, memory, co_inner));
    }
}
impl<O: IntOperand<u16>> IntOperand<u16> for IndirectOperand8<O> {
    #[inline(always)]
    async fn get(&self, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) -> u16 {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        u16::from_le_bytes([
            gen_all!(&co, |co_inner| cpu.read_u8(address, memory, co_inner)),
            gen_all!(&co, |co_inner| cpu.read_u8(address + 1, memory, co_inner)),
        ])
    }
    #[inline(always)]
    async fn set(&self, value: u16, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        let bytes = u16::to_le_bytes(value);
        gen_all!(&co, |co_inner| cpu.write_u8(address, bytes[0], memory, co_inner));
        gen_all!(&co, |co_inner| cpu.write_u8(address + 1, bytes[1], memory, co_inner));
    }
}

pub struct RegisterOperand16(pub cpu::Register16);
impl IntOperand<u16> for RegisterOperand16 {
    #[inline(always)]
    async fn get(&self, cpu: &mut CPU, _: &mut Memory, _co: Co<'_, ()>) -> u16 {
        match self.0 {
            cpu::Register16::AF => u16::from_le_bytes(cpu.af),
            cpu::Register16::BC => u16::from_le_bytes(cpu.bc),
            cpu::Register16::DE => u16::from_le_bytes(cpu.de),
            cpu::Register16::HL => u16::from_le_bytes(cpu.hl),
            cpu::Register16::SP => cpu.sp,
            cpu::Register16::PC => cpu.pc,
        }
    }
    #[inline(always)]
    async fn set(&self, value: u16, cpu: &mut CPU, _: &mut Memory, _co: Co<'_, ()>) {
        match self.0 {
            cpu::Register16::AF => cpu.af = u16::to_le_bytes(value),
            cpu::Register16::BC => cpu.bc = u16::to_le_bytes(value),
            cpu::Register16::DE => cpu.de = u16::to_le_bytes(value),
            cpu::Register16::HL => cpu.hl = u16::to_le_bytes(value),
            cpu::Register16::SP => cpu.sp = value,
            cpu::Register16::PC => cpu.pc = value,
        }
    }
}

pub struct ImmediateOperand16;
impl IntOperand<u16> for ImmediateOperand16 {
    #[inline(always)]
    async fn get(&self, cpu: &mut CPU, memory: &mut Memory, co: Co<'_, ()>) -> u16 {
        u16::from_le_bytes([gen_all!(&co, |co_inner| cpu.step_u8(memory, co_inner)), gen_all!(&co, |co_inner| cpu.step_u8(memory, co_inner))])
    }
    #[inline(always)]
    async fn set(&self, _: u16, _: &mut CPU, _: &mut Memory, _co: Co<'_, ()>) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct ConstOperand16(u16);
impl IntOperand<u16> for ConstOperand16 {
    #[inline(always)]
    async fn get(&self, _: &mut CPU, _: &mut Memory, _co: Co<'_, ()>) -> u16 {
        self.0
    }
    #[inline(always)]
    async fn set(&self, _: u16, _: &mut CPU, _: &mut Memory, _co: Co<'_, ()>) {
        panic!("Cannot write to constant operand")
    }
}

pub enum CondOperand {
    Unconditional,
    NZ,
    Z,
    NC,
    C,
}
impl CondOperand {
    #[inline(always)]
    pub fn evaluate(&self, cpu: &CPU) -> bool {
        match self {
            Self::Unconditional => true,
            Self::NZ => cpu.af[0] & 0b10000000 == 0,
            Self::Z => cpu.af[0] & 0b10000000 != 0,
            Self::NC => cpu.af[0] & 0b00010000 == 0,
            Self::C => cpu.af[0] & 0b00010000 != 0,
        }
    }
}

impl CPU {
    pub async fn opcode_gen(&mut self, memory: &mut Memory, co: Co<'_, ()>) {
        match self.ir {
            0x00 => { // NOP
                // Do nothing
            }
            0x01 => {
                // LD BC, nn
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand16(cpu::Register16::BC), ImmediateOperand16));
            }
            0x02 => {
                // LD (BC), A
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    IndirectOperand8(RegisterOperand16(cpu::Register16::BC)),
                    RegisterOperand8(cpu::Register8::A)
                ));
            }
            0x03 => {
                // INC BC
                gen_all!(co, |co_inner| self.inc16(memory, co_inner, RegisterOperand16(cpu::Register16::BC)));
            }
            0x04 => {
                // INC B
                gen_all!(co, |co_inner| self.inc(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x05 => {
                // DEC B
                gen_all!(co, |co_inner| self.dec(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x06 => {
                // LD B, n
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), ImmediateOperand8));
            }
            0x07 => {
                // RLCA
                gen_all!(co, |co_inner| self.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }
            0x08 => {
                // LD (nn), SP
                gen_all!(co, |co_inner| self.ld(memory, co_inner, IndirectOperand8(ImmediateOperand16), RegisterOperand16(cpu::Register16::SP)));
            }
            0x09 => {
                // ADD HL, BC
                gen_all!(co, |co_inner| self.add_hl(memory, co_inner, RegisterOperand16(cpu::Register16::BC)));
            }
            0x0A => {
                // LD A, (BC)
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    RegisterOperand8(cpu::Register8::A),
                    IndirectOperand8(RegisterOperand16(cpu::Register16::BC))
                ));
            }
            0x0B => {
                // DEC BC
                gen_all!(co, |co_inner| self.dec16(memory, co_inner, RegisterOperand16(cpu::Register16::BC)));
            }
            0x0C => {
                // INC C
                gen_all!(co, |co_inner| self.inc(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x0D => {
                // DEC C
                gen_all!(co, |co_inner| self.dec(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x0E => {
                // LD C, n
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), ImmediateOperand8));
            }
            0x0F => {
                // RRCA
                gen_all!(co, |co_inner| self.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }

            0x10 => {
                // STOP
                gen_all!(co, |co_inner| self.stop(memory, co_inner));
            }
            0x11 => {
                // LD DE, nn
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand16(cpu::Register16::DE), ImmediateOperand16));
            }
            0x12 => {
                // LD (DE), A
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    IndirectOperand8(RegisterOperand16(cpu::Register16::DE)),
                    RegisterOperand8(cpu::Register8::A)
                ));
            }
            0x13 => {
                // INC DE
                gen_all!(co, |co_inner| self.inc16(memory, co_inner, RegisterOperand16(cpu::Register16::DE)));
            }
            0x14 => {
                // INC D
                gen_all!(co, |co_inner| self.inc(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x15 => {
                // DEC D
                gen_all!(co, |co_inner| self.dec(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x16 => {
                // LD D, n
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), ImmediateOperand8));
            }
            0x17 => {
                // RLA
                gen_all!(co, |co_inner| self.rl(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }
            0x18 => {
                // JR e
                gen_all!(co, |co_inner| self.jr(memory, co_inner, CondOperand::Unconditional, ImmediateSignedOperand8));
            }
            0x19 => {
                // ADD HL, DE
                gen_all!(co, |co_inner| self.add_hl(memory, co_inner, RegisterOperand16(cpu::Register16::DE)));
            }
            0x1A => {
                // LD A, (DE)
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    RegisterOperand8(cpu::Register8::A),
                    IndirectOperand8(RegisterOperand16(cpu::Register16::DE))
                ));
            }
            0x1B => {
                // DEC DE
                gen_all!(co, |co_inner| self.dec16(memory, co_inner, RegisterOperand16(cpu::Register16::DE)));
            }
            0x1C => {
                // INC E
                gen_all!(co, |co_inner| self.inc(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x1D => {
                // DEC E
                gen_all!(co, |co_inner| self.dec(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x1E => {
                // LD E, n
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), ImmediateOperand8));
            }
            0x1F => {
                // RRA
                gen_all!(co, |co_inner| self.rr(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }

            0x20 => {
                // JR NZ, e
                gen_all!(co, |co_inner| self.jr(memory, co_inner, CondOperand::NZ, ImmediateSignedOperand8));
            }
            0x21 => {
                // LD HL, nn
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand16(cpu::Register16::HL), ImmediateOperand16));
            }
            0x22 => {
                // LD (HL+), A
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    IncIndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
                    RegisterOperand8(cpu::Register8::A)
                ));
            }
            0x23 => {
                // INC HL
                gen_all!(co, |co_inner| self.inc16(memory, co_inner, RegisterOperand16(cpu::Register16::HL)));
            }
            0x24 => {
                // INC H
                gen_all!(co, |co_inner| self.inc(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x25 => {
                // DEC H
                gen_all!(co, |co_inner| self.dec(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x26 => {
                // LD H, n
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), ImmediateOperand8));
            }
            0x27 => {
                // DAA
                gen_all!(co, |co_inner| self.daa(memory, co_inner));
            }
            0x28 => {
                // JR Z, e
                gen_all!(co, |co_inner| self.jr(memory, co_inner, CondOperand::Z, ImmediateSignedOperand8));
            }
            0x29 => {
                // ADD HL, HL
                gen_all!(co, |co_inner| self.add_hl(memory, co_inner, RegisterOperand16(cpu::Register16::HL)));
            }
            0x2A => {
                // LD A, (HL+)
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    RegisterOperand8(cpu::Register8::A),
                    IncIndirectOperand8(RegisterOperand16(cpu::Register16::HL))
                ));
            }
            0x2B => {
                // DEC HL
                gen_all!(co, |co_inner| self.dec16(memory, co_inner, RegisterOperand16(cpu::Register16::HL)));
            }
            0x2C => {
                // INC L
                gen_all!(co, |co_inner| self.inc(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x2D => {
                // DEC L
                gen_all!(co, |co_inner| self.dec(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x2E => {
                // LD L, n
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), ImmediateOperand8));
            }
            0x2F => {
                // CPL
                gen_all!(co, |co_inner| self.cpl(memory, co_inner));
            }

            0x30 => {
                // JR NC, e
                gen_all!(co, |co_inner| self.jr(memory, co_inner, CondOperand::NC, ImmediateSignedOperand8));
            }
            0x31 => {
                // LD SP, nn
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand16(cpu::Register16::SP), ImmediateOperand16));
            }
            0x32 => {
                // LD (HL-), A
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    DecIndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
                    RegisterOperand8(cpu::Register8::A)
                ));
            }
            0x33 => {
                // INC SP
                gen_all!(co, |co_inner| self.inc16(memory, co_inner, RegisterOperand16(cpu::Register16::SP)));
            }
            0x34 => {
                // INC (HL)
                gen_all!(co, |co_inner| self.inc(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x35 => {
                // DEC (HL)
                gen_all!(co, |co_inner| self.dec(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x36 => {
                // LD (HL), n
                gen_all!(co, |co_inner| self.ld(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), ImmediateOperand8));
            }
            0x37 => {
                // SCF
                gen_all!(co, |co_inner| self.scf(memory, co_inner));
            }
            0x38 => {
                // JR C, e
                gen_all!(co, |co_inner| self.jr(memory, co_inner, CondOperand::C, ImmediateSignedOperand8));
            }
            0x39 => {
                // ADD HL, SP
                gen_all!(co, |co_inner| self.add_hl(memory, co_inner, RegisterOperand16(cpu::Register16::SP)));
            }
            0x3A => {
                // LD A, (HL-)
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    RegisterOperand8(cpu::Register8::A),
                    DecIndirectOperand8(RegisterOperand16(cpu::Register16::HL))
                ));
            }
            0x3B => {
                // DEC SP
                gen_all!(co, |co_inner| self.dec16(memory, co_inner, RegisterOperand16(cpu::Register16::SP)));
            }
            0x3C => {
                // INC A
                gen_all!(co, |co_inner| self.inc(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }
            0x3D => {
                // DEC A
                gen_all!(co, |co_inner| self.dec(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }
            0x3E => {
                // LD A, n
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), ImmediateOperand8));
            }
            0x3F => {
                // CCF
                gen_all!(co, |co_inner| self.ccf(memory, co_inner));
            }

            0x40 => {
                // LD B, B
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::B)));
            }
            0x41 => {
                // LD B, C
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::C)));
            }
            0x42 => {
                // LD B, D
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::D)));
            }
            0x43 => {
                // LD B, E
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::E)));
            }
            0x44 => {
                // LD B, H
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::H)));
            }
            0x45 => {
                // LD B, L
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::L)));
            }
            0x46 => {
                // LD B, (HL)
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    RegisterOperand8(cpu::Register8::B),
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
                ));
            }
            0x47 => {
                // LD B, A
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::A)));
            }
            0x48 => {
                // LD C, B
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::B)));
            }
            0x49 => {
                // LD C, C
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::C)));
            }
            0x4A => {
                // LD C, D
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::D)));
            }
            0x4B => {
                // LD C, E
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::E)));
            }
            0x4C => {
                // LD C, H
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::H)));
            }
            0x4D => {
                // LD C, L
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::L)));
            }
            0x4E => {
                // LD C, (HL)
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    RegisterOperand8(cpu::Register8::C),
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
                ));
            }
            0x4F => {
                // LD C, A
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::A)));
            }

            0x50 => {
                // LD D, B
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::B)));
            }
            0x51 => {
                // LD D, C
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::C)));
            }
            0x52 => {
                // LD D, D
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::D)));
            }
            0x53 => {
                // LD D, E
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::E)));
            }
            0x54 => {
                // LD D, H
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::H)));
            }
            0x55 => {
                // LD D, L
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::L)));
            }
            0x56 => {
                // LD D, (HL)
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    RegisterOperand8(cpu::Register8::D),
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
                ));
            }
            0x57 => {
                // LD D, A
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::A)));
            }
            0x58 => {
                // LD E, B
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::B)));
            }
            0x59 => {
                // LD E, C
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::C)));
            }
            0x5A => {
                // LD E, D
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::D)));
            }
            0x5B => {
                // LD E, E
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::E)));
            }
            0x5C => {
                // LD E, H
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::H)));
            }
            0x5D => {
                // LD E, L
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::L)));
            }
            0x5E => {
                // LD E, (HL)
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    RegisterOperand8(cpu::Register8::E),
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
                ));
            }
            0x5F => {
                // LD E, A
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::A)));
            }

            0x60 => {
                // LD H, B
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::B)));
            }
            0x61 => {
                // LD H, C
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::C)));
            }
            0x62 => {
                // LD H, D
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::D)));
            }
            0x63 => {
                // LD H, E
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::E)));
            }
            0x64 => {
                // LD H, H
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::H)));
            }
            0x65 => {
                // LD H, L
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::L)));
            }
            0x66 => {
                // LD H, (HL)
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    RegisterOperand8(cpu::Register8::H),
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
                ));
            }
            0x67 => {
                // LD H, A
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::A)));
            }
            0x68 => {
                // LD L, B
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::B)));
            }
            0x69 => {
                // LD L, C
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::C)));
            }
            0x6A => {
                // LD L, D
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::D)));
            }
            0x6B => {
                // LD L, E
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::E)));
            }
            0x6C => {
                // LD L, H
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::H)));
            }
            0x6D => {
                // LD L, L
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::L)));
            }
            0x6E => {
                // LD L, (HL)
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    RegisterOperand8(cpu::Register8::L),
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
                ));
            }
            0x6F => {
                // LD L, A
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::A)));
            }

            0x70 => {
                // LD (HL), B
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
                    RegisterOperand8(cpu::Register8::B)
                ));
            }
            0x71 => {
                // LD (HL), C
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
                    RegisterOperand8(cpu::Register8::C)
                ));
            }
            0x72 => {
                // LD (HL), D
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
                    RegisterOperand8(cpu::Register8::D)
                ));
            }
            0x73 => {
                // LD (HL), E
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
                    RegisterOperand8(cpu::Register8::E)
                ));
            }
            0x74 => {
                // LD (HL), H
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
                    RegisterOperand8(cpu::Register8::H)
                ));
            }
            0x75 => {
                // LD (HL), L
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
                    RegisterOperand8(cpu::Register8::L)
                ));
            }
            0x76 => {
                // HALT
                gen_all!(co, |co_inner| self.halt(memory, co_inner));
            }
            0x77 => {
                // LD (HL), A
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
                    RegisterOperand8(cpu::Register8::A)
                ));
            }
            0x78 => {
                // LD A, B
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::B)));
            }
            0x79 => {
                // LD A, C
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::C)));
            }
            0x7A => {
                // LD A, D
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::D)));
            }
            0x7B => {
                // LD A, E
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::E)));
            }
            0x7C => {
                // LD A, H
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::H)));
            }
            0x7D => {
                // LD A, L
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::L)));
            }
            0x7E => {
                // LD A, (HL)
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    RegisterOperand8(cpu::Register8::A),
                    IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
                ));
            }
            0x7F => {
                // LD A, A
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::A)));
            }

            0x80 => {
                // ADD B
                gen_all!(co, |co_inner| self.add(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x81 => {
                // ADD C
                gen_all!(co, |co_inner| self.add(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x82 => {
                // ADD D
                gen_all!(co, |co_inner| self.add(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x83 => {
                // ADD E
                gen_all!(co, |co_inner| self.add(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x84 => {
                // ADD H
                gen_all!(co, |co_inner| self.add(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x85 => {
                // ADD L
                gen_all!(co, |co_inner| self.add(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x86 => {
                // ADD (HL)
                gen_all!(co, |co_inner| self.add(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x87 => {
                // ADD A
                gen_all!(co, |co_inner| self.add(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }
            0x88 => {
                // ADC B
                gen_all!(co, |co_inner| self.adc(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x89 => {
                // ADC C
                gen_all!(co, |co_inner| self.adc(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x8A => {
                // ADC D
                gen_all!(co, |co_inner| self.adc(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x8B => {
                // ADC E
                gen_all!(co, |co_inner| self.adc(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x8C => {
                // ADC H
                gen_all!(co, |co_inner| self.adc(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x8D => {
                // ADC L
                gen_all!(co, |co_inner| self.adc(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x8E => {
                // ADC (HL)
                gen_all!(co, |co_inner| self.adc(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x8F => {
                // ADC A
                gen_all!(co, |co_inner| self.adc(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }

            0x90 => {
                // SUB B
                gen_all!(co, |co_inner| self.sub(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x91 => {
                // SUB C
                gen_all!(co, |co_inner| self.sub(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x92 => {
                // SUB D
                gen_all!(co, |co_inner| self.sub(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x93 => {
                // SUB E
                gen_all!(co, |co_inner| self.sub(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x94 => {
                // SUB H
                gen_all!(co, |co_inner| self.sub(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x95 => {
                // SUB L
                gen_all!(co, |co_inner| self.sub(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x96 => {
                // SUB (HL)
                gen_all!(co, |co_inner| self.sub(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x97 => {
                // SUB A
                gen_all!(co, |co_inner| self.sub(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }
            0x98 => {
                // SBC B
                gen_all!(co, |co_inner| self.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x99 => {
                // SBC C
                gen_all!(co, |co_inner| self.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x9A => {
                // SBC D
                gen_all!(co, |co_inner| self.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x9B => {
                // SBC E
                gen_all!(co, |co_inner| self.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x9C => {
                // SBC H
                gen_all!(co, |co_inner| self.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x9D => {
                // SBC L
                gen_all!(co, |co_inner| self.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x9E => {
                // SBC (HL)
                gen_all!(co, |co_inner| self.sbc(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x9F => {
                // SBC A
                gen_all!(co, |co_inner| self.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }

            0xA0 => {
                // AND B
                gen_all!(co, |co_inner| self.and(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0xA1 => {
                // AND C
                gen_all!(co, |co_inner| self.and(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0xA2 => {
                // AND D
                gen_all!(co, |co_inner| self.and(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0xA3 => {
                // AND E
                gen_all!(co, |co_inner| self.and(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0xA4 => {
                // AND H
                gen_all!(co, |co_inner| self.and(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0xA5 => {
                // AND L
                gen_all!(co, |co_inner| self.and(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0xA6 => {
                // AND (HL)
                gen_all!(co, |co_inner| self.and(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xA7 => {
                // AND A
                gen_all!(co, |co_inner| self.and(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }
            0xA8 => {
                // XOR B
                gen_all!(co, |co_inner| self.xor(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0xA9 => {
                // XOR C
                gen_all!(co, |co_inner| self.xor(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0xAA => {
                // XOR D
                gen_all!(co, |co_inner| self.xor(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0xAB => {
                // XOR E
                gen_all!(co, |co_inner| self.xor(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0xAC => {
                // XOR H
                gen_all!(co, |co_inner| self.xor(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0xAD => {
                // XOR L
                gen_all!(co, |co_inner| self.xor(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0xAE => {
                // XOR (HL)
                gen_all!(co, |co_inner| self.xor(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xAF => {
                // XOR A
                gen_all!(co, |co_inner| self.xor(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }

            0xB0 => {
                // OR B
                gen_all!(co, |co_inner| self.or(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0xB1 => {
                // OR C
                gen_all!(co, |co_inner| self.or(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0xB2 => {
                // OR D
                gen_all!(co, |co_inner| self.or(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0xB3 => {
                // OR E
                gen_all!(co, |co_inner| self.or(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0xB4 => {
                // OR H
                gen_all!(co, |co_inner| self.or(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0xB5 => {
                // OR L
                gen_all!(co, |co_inner| self.or(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0xB6 => {
                // OR (HL)
                gen_all!(co, |co_inner| self.or(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xB7 => {
                // OR A
                gen_all!(co, |co_inner| self.or(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }
            0xB8 => {
                // CP B
                gen_all!(co, |co_inner| self.cp(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0xB9 => {
                // CP C
                gen_all!(co, |co_inner| self.cp(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0xBA => {
                // CP D
                gen_all!(co, |co_inner| self.cp(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0xBB => {
                // CP E
                gen_all!(co, |co_inner| self.cp(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0xBC => {
                // CP H
                gen_all!(co, |co_inner| self.cp(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0xBD => {
                // CP L
                gen_all!(co, |co_inner| self.cp(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0xBE => {
                // CP (HL)
                gen_all!(co, |co_inner| self.cp(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xBF => {
                // CP A
                gen_all!(co, |co_inner| self.cp(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }

            0xC0 => {
                // RET NZ
                gen_all!(co, |co_inner| self.ret(memory, co_inner, CondOperand::NZ));
            }
            0xC1 => {
                // POP BC
                gen_all!(co, |co_inner| self.pop(memory, co_inner, RegisterOperand16(cpu::Register16::BC)));
            }
            0xC2 => {
                // JP NZ, nn
                gen_all!(co, |co_inner| self.jp(memory, co_inner, CondOperand::NZ, ImmediateOperand16));
            }
            0xC3 => {
                // JP nn
                gen_all!(co, |co_inner| self.jp(memory, co_inner, CondOperand::Unconditional, ImmediateOperand16));
            }
            0xC4 => {
                // CALL NZ, nn
                gen_all!(co, |co_inner| self.call(memory, co_inner, CondOperand::NZ, ImmediateOperand16));
            }
            0xC5 => {
                // PUSH BC
                gen_all!(co, |co_inner| self.push(memory, co_inner, RegisterOperand16(cpu::Register16::BC)));
            }
            0xC6 => {
                // ADD n
                gen_all!(co, |co_inner| self.add(memory, co_inner, ImmediateOperand8));
            }
            0xC7 => {
                // RST 0x00
                gen_all!(co, |co_inner| self.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0000)));
            }
            0xC8 => {
                // RET Z
                gen_all!(co, |co_inner| self.ret(memory, co_inner, CondOperand::Z));
            }
            0xC9 => {
                // RET
                gen_all!(co, |co_inner| self.ret(memory, co_inner, CondOperand::Unconditional));
            }
            0xCA => {
                // JP Z, nn
                gen_all!(co, |co_inner| self.jp(memory, co_inner, CondOperand::Z, ImmediateOperand16));
            }
            0xCB => {
                // CB op
                self.ir = gen_all!(&co, |co_inner| self.step_u8(memory, co_inner));
                gen_all!(co, |co_inner| self.cb_gen(memory, co_inner));
            }
            0xCC => {
                // CALL Z, nn
                gen_all!(co, |co_inner| self.call(memory, co_inner, CondOperand::Z, ImmediateOperand16));
            }
            0xCD => {
                // CALL nn
                gen_all!(co, |co_inner| self.call(memory, co_inner, CondOperand::Unconditional, ImmediateOperand16));
            }
            0xCE => {
                // ADC n
                gen_all!(co, |co_inner| self.adc(memory, co_inner, ImmediateOperand8));
            }
            0xCF => {
                // RST 0x08
                gen_all!(co, |co_inner| self.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0008)));
            }

            0xD0 => {
                // RET NC
                gen_all!(co, |co_inner| self.ret(memory, co_inner, CondOperand::NC));
            }
            0xD1 => {
                // POP DE
                gen_all!(co, |co_inner| self.pop(memory, co_inner, RegisterOperand16(cpu::Register16::DE)));
            }
            0xD2 => {
                // JP NC, nn
                gen_all!(co, |co_inner| self.jp(memory, co_inner, CondOperand::NC, ImmediateOperand16));
            }
            // 0xD3 (invalid)
            0xD4 => {
                // CALL NC, nn
                gen_all!(co, |co_inner| self.call(memory, co_inner, CondOperand::NC, ImmediateOperand16));
            }
            0xD5 => {
                // PUSH DE
                gen_all!(co, |co_inner| self.push(memory, co_inner, RegisterOperand16(cpu::Register16::DE)));
            }
            0xD6 => {
                // SUB n
                gen_all!(co, |co_inner| self.sub(memory, co_inner, ImmediateOperand8));
            }
            0xD7 => {
                // RST 0x10
                gen_all!(co, |co_inner| self.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0010)));
            }
            0xD8 => {
                // RET C
                gen_all!(co, |co_inner| self.ret(memory, co_inner, CondOperand::C));
            }
            0xD9 => {
                // RETI
                gen_all!(co, |co_inner| self.reti(memory, co_inner));
            }
            0xDA => {
                // JP C, nn
                gen_all!(co, |co_inner| self.jp(memory, co_inner, CondOperand::C, ImmediateOperand16));
            }
            // 0xDB (invalid)
            0xDC => {
                // CALL C, nn
                gen_all!(co, |co_inner| self.call(memory, co_inner, CondOperand::C, ImmediateOperand16));
            }
            // 0xDD (invalid)
            0xDE => {
                // SBC n
                gen_all!(co, |co_inner| self.sbc(memory, co_inner, ImmediateOperand8));
            }
            0xDF => {
                // RST 0x18
                gen_all!(co, |co_inner| self.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0018)));
            }

            0xE0 => {
                // LDH (n), A
                gen_all!(co, |co_inner| self.ld(memory, co_inner, HramIndirectOperand(ImmediateOperand8), RegisterOperand8(cpu::Register8::A)));
            }
            0xE1 => {
                // POP HL
                gen_all!(co, |co_inner| self.pop(memory, co_inner, RegisterOperand16(cpu::Register16::HL)));
            }
            0xE2 => {
                // LDH (C), A
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    HramIndirectOperand(RegisterOperand8(cpu::Register8::C)),
                    RegisterOperand8(cpu::Register8::A)
                ));
            }
            // 0xE3 (invalid)
            // 0xE4 (invalid)
            0xE5 => {
                // PUSH HL
                gen_all!(co, |co_inner| self.push(memory, co_inner, RegisterOperand16(cpu::Register16::HL)));
            }
            0xE6 => {
                // AND n
                gen_all!(co, |co_inner| self.and(memory, co_inner, ImmediateOperand8));
            }
            0xE7 => {
                // RST 0x20
                gen_all!(co, |co_inner| self.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0020)));
            }
            0xE8 => {
                // ADD SP, e
                gen_all!(co, |co_inner| self.add_spe(memory, co_inner));
            }
            0xE9 => {
                // JP HL
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand16(cpu::Register16::PC), RegisterOperand16(cpu::Register16::HL)));
            }
            0xEA => {
                // LD (nn), A
                gen_all!(co, |co_inner| self.ld(memory, co_inner, IndirectOperand8(ImmediateOperand16), RegisterOperand8(cpu::Register8::A)));
            }
            // 0xEB (invalid)
            // 0xEC (invalid)
            // 0xED (invalid)
            0xEE => {
                // XOR n
                gen_all!(co, |co_inner| self.xor(memory, co_inner, ImmediateOperand8));
            }
            0xEF => {
                // RST 0x28
                gen_all!(co, |co_inner| self.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0028)));
            }

            0xF0 => {
                // LDH A, (n)
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), HramIndirectOperand(ImmediateOperand8)));
            }
            0xF1 => {
                // POP AF
                gen_all!(co, |co_inner| self.pop(memory, co_inner, RegisterOperand16(cpu::Register16::AF)));
            }
            0xF2 => {
                // LDH A, (C)
                gen_all!(co, |co_inner| self.ld(
                    memory,
                    co_inner,
                    RegisterOperand8(cpu::Register8::A),
                    HramIndirectOperand(RegisterOperand8(cpu::Register8::C))
                ));
            }
            0xF3 => {
                // DI
                gen_all!(co, |co_inner| self.di(memory, co_inner));
            }
            // 0xF4 (invalid)
            0xF5 => {
                // PUSH AF
                gen_all!(co, |co_inner| self.push(memory, co_inner, RegisterOperand16(cpu::Register16::AF)));
            }
            0xF6 => {
                // OR n
                gen_all!(co, |co_inner| self.or(memory, co_inner, ImmediateOperand8));
            }
            0xF7 => {
                // RST 0x30
                gen_all!(co, |co_inner| self.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0030)));
            }
            0xF8 => {
                // LD HL, SP+e
                gen_all!(co, |co_inner| self.ld_hlspe(memory, co_inner));
            }
            0xF9 => {
                // LD SP, HL
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand16(cpu::Register16::SP), RegisterOperand16(cpu::Register16::HL)));
                co.yield_(()).await;
            }
            0xFA => {
                // LD A, (nn)
                gen_all!(co, |co_inner| self.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), IndirectOperand8(ImmediateOperand16)));
            }
            0xFB => {
                // EI
                gen_all!(co, |co_inner| self.ei(memory, co_inner));
            }
            // 0xFC (invalid)
            // 0xFD (invalid)
            0xFE => {
                // CP n
                gen_all!(co, |co_inner| self.cp(memory, co_inner, ImmediateOperand8));
            }
            0xFF => {
                // RST 0x38
                gen_all!(co, |co_inner| self.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0038)));
            }
            _ => panic!("Unknown opcode"),
        }
    }

    pub async fn cb_gen(&mut self, memory: &mut Memory, co: Co<'_, ()>) {
        match self.ir {
            0x00 => {
                // RLC B
                gen_all!(co, |co_inner| self.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x01 => {
                // RLC C
                gen_all!(co, |co_inner| self.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x02 => {
                // RLC D
                gen_all!(co, |co_inner| self.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x03 => {
                // RLC E
                gen_all!(co, |co_inner| self.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x04 => {
                // RLC H
                gen_all!(co, |co_inner| self.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x05 => {
                // RLC L
                gen_all!(co, |co_inner| self.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x06 => {
                // RLC (HL)
                gen_all!(co, |co_inner| self.rlc(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x07 => {
                // RLC A
                gen_all!(co, |co_inner| self.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }
            0x08 => {
                // RRC B
                gen_all!(co, |co_inner| self.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x09 => {
                // RRC C
                gen_all!(co, |co_inner| self.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x0A => {
                // RRC D
                gen_all!(co, |co_inner| self.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x0B => {
                // RRC E
                gen_all!(co, |co_inner| self.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x0C => {
                // RRC H
                gen_all!(co, |co_inner| self.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x0D => {
                // RRC L
                gen_all!(co, |co_inner| self.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x0E => {
                // RRC (HL)
                gen_all!(co, |co_inner| self.rrc(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x0F => {
                // RRC A
                gen_all!(co, |co_inner| self.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }

            0x10 => {
                // RL B
                gen_all!(co, |co_inner| self.rl(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x11 => {
                // RL C
                gen_all!(co, |co_inner| self.rl(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x12 => {
                // RL D
                gen_all!(co, |co_inner| self.rl(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x13 => {
                // RL E
                gen_all!(co, |co_inner| self.rl(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x14 => {
                // RL H
                gen_all!(co, |co_inner| self.rl(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x15 => {
                // RL L
                gen_all!(co, |co_inner| self.rl(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x16 => {
                // RL (HL)
                gen_all!(co, |co_inner| self.rl(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x17 => {
                // RL A
                gen_all!(co, |co_inner| self.rl(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }
            0x18 => {
                // RR B
                gen_all!(co, |co_inner| self.rr(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x19 => {
                // RR C
                gen_all!(co, |co_inner| self.rr(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x1A => {
                // RR D
                gen_all!(co, |co_inner| self.rr(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x1B => {
                // RR E
                gen_all!(co, |co_inner| self.rr(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x1C => {
                // RR H
                gen_all!(co, |co_inner| self.rr(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x1D => {
                // RR L
                gen_all!(co, |co_inner| self.rr(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x1E => {
                // RR (HL)
                gen_all!(co, |co_inner| self.rr(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x1F => {
                // RR A
                gen_all!(co, |co_inner| self.rr(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }

            0x20 => {
                // SLA B
                gen_all!(co, |co_inner| self.sla(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x21 => {
                // SLA C
                gen_all!(co, |co_inner| self.sla(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x22 => {
                // SLA D
                gen_all!(co, |co_inner| self.sla(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x23 => {
                // SLA E
                gen_all!(co, |co_inner| self.sla(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x24 => {
                // SLA H
                gen_all!(co, |co_inner| self.sla(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x25 => {
                // SLA L
                gen_all!(co, |co_inner| self.sla(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x26 => {
                // SLA (HL)
                gen_all!(co, |co_inner| self.sla(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x27 => {
                // SLA A
                gen_all!(co, |co_inner| self.sla(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }
            0x28 => {
                // SRA B
                gen_all!(co, |co_inner| self.sra(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x29 => {
                // SRA C
                gen_all!(co, |co_inner| self.sra(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x2A => {
                // SRA D
                gen_all!(co, |co_inner| self.sra(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x2B => {
                // SRA E
                gen_all!(co, |co_inner| self.sra(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x2C => {
                // SRA H
                gen_all!(co, |co_inner| self.sra(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x2D => {
                // SRA L
                gen_all!(co, |co_inner| self.sra(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x2E => {
                // SRA (HL)
                gen_all!(co, |co_inner| self.sra(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x2F => {
                // SRA A
                gen_all!(co, |co_inner| self.sra(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }

            0x30 => {
                // SWAP B
                gen_all!(co, |co_inner| self.swap(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x31 => {
                // SWAP C
                gen_all!(co, |co_inner| self.swap(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x32 => {
                // SWAP D
                gen_all!(co, |co_inner| self.swap(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x33 => {
                // SWAP E
                gen_all!(co, |co_inner| self.swap(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x34 => {
                // SWAP H
                gen_all!(co, |co_inner| self.swap(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x35 => {
                // SWAP L
                gen_all!(co, |co_inner| self.swap(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x36 => {
                // SWAP (HL)
                gen_all!(co, |co_inner| self.swap(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x37 => {
                // SWAP A
                gen_all!(co, |co_inner| self.swap(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }
            0x38 => {
                // SRL B
                gen_all!(co, |co_inner| self.srl(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
            }
            0x39 => {
                // SRL C
                gen_all!(co, |co_inner| self.srl(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
            }
            0x3A => {
                // SRL D
                gen_all!(co, |co_inner| self.srl(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
            }
            0x3B => {
                // SRL E
                gen_all!(co, |co_inner| self.srl(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
            }
            0x3C => {
                // SRL H
                gen_all!(co, |co_inner| self.srl(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
            }
            0x3D => {
                // SRL L
                gen_all!(co, |co_inner| self.srl(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
            }
            0x3E => {
                // SRL (HL)
                gen_all!(co, |co_inner| self.srl(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x3F => {
                // SRL A
                gen_all!(co, |co_inner| self.srl(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
            }

            0x40 => {
                // BIT 0, B
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::B)));
            }
            0x41 => {
                // BIT 0, C
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::C)));
            }
            0x42 => {
                // BIT 0, D
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::D)));
            }
            0x43 => {
                // BIT 0, E
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::E)));
            }
            0x44 => {
                // BIT 0, H
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::H)));
            }
            0x45 => {
                // BIT 0, L
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::L)));
            }
            0x46 => {
                // BIT 0, (HL)
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x47 => {
                // BIT 0, A
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::A)));
            }
            0x48 => {
                // BIT 1, B
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::B)));
            }
            0x49 => {
                // BIT 1, C
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::C)));
            }
            0x4A => {
                // BIT 1, D
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::D)));
            }
            0x4B => {
                // BIT 1, E
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::E)));
            }
            0x4C => {
                // BIT 1, H
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::H)));
            }
            0x4D => {
                // BIT 1, L
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::L)));
            }
            0x4E => {
                // BIT 1, (HL)
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x4F => {
                // BIT 1, A
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::A)));
            }

            0x50 => {
                // BIT 2, B
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::B)));
            }
            0x51 => {
                // BIT 2, C
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::C)));
            }
            0x52 => {
                // BIT 2, D
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::D)));
            }
            0x53 => {
                // BIT 2, E
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::E)));
            }
            0x54 => {
                // BIT 2, H
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::H)));
            }
            0x55 => {
                // BIT 2, L
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::L)));
            }
            0x56 => {
                // BIT 2, (HL)
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x57 => {
                // BIT 2, A
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::A)));
            }
            0x58 => {
                // BIT 3, B
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::B)));
            }
            0x59 => {
                // BIT 3, C
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::C)));
            }
            0x5A => {
                // BIT 3, D
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::D)));
            }
            0x5B => {
                // BIT 3, E
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::E)));
            }
            0x5C => {
                // BIT 3, H
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::H)));
            }
            0x5D => {
                // BIT 3, L
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::L)));
            }
            0x5E => {
                // BIT 3, (HL)
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x5F => {
                // BIT 3, A
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::A)));
            }

            0x60 => {
                // BIT 4, B
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::B)));
            }
            0x61 => {
                // BIT 4, C
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::C)));
            }
            0x62 => {
                // BIT 4, D
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::D)));
            }
            0x63 => {
                // BIT 4, E
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::E)));
            }
            0x64 => {
                // BIT 4, H
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::H)));
            }
            0x65 => {
                // BIT 4, L
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::L)));
            }
            0x66 => {
                // BIT 4, (HL)
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x67 => {
                // BIT 4, A
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::A)));
            }
            0x68 => {
                // BIT 5, B
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::B)));
            }
            0x69 => {
                // BIT 5, C
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::C)));
            }
            0x6A => {
                // BIT 5, D
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::D)));
            }
            0x6B => {
                // BIT 5, E
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::E)));
            }
            0x6C => {
                // BIT 5, H
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::H)));
            }
            0x6D => {
                // BIT 5, L
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::L)));
            }
            0x6E => {
                // BIT 5, (HL)
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x6F => {
                // BIT 5, A
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::A)));
            }

            0x70 => {
                // BIT 6, B
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::B)));
            }
            0x71 => {
                // BIT 6, C
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::C)));
            }
            0x72 => {
                // BIT 6, D
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::D)));
            }
            0x73 => {
                // BIT 6, E
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::E)));
            }
            0x74 => {
                // BIT 6, H
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::H)));
            }
            0x75 => {
                // BIT 6, L
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::L)));
            }
            0x76 => {
                // BIT 6, (HL)
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x77 => {
                // BIT 6, A
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::A)));
            }
            0x78 => {
                // BIT 7, B
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::B)));
            }
            0x79 => {
                // BIT 7, C
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::C)));
            }
            0x7A => {
                // BIT 7, D
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::D)));
            }
            0x7B => {
                // BIT 7, E
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::E)));
            }
            0x7C => {
                // BIT 7, H
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::H)));
            }
            0x7D => {
                // BIT 7, L
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::L)));
            }
            0x7E => {
                // BIT 7, (HL)
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x7F => {
                // BIT 7, A
                gen_all!(co, |co_inner| self.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::A)));
            }

            0x80 => {
                // RES 0, B
                gen_all!(co, |co_inner| self.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::B)));
            }
            0x81 => {
                // RES 0, C
                gen_all!(co, |co_inner| self.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::C)));
            }
            0x82 => {
                // RES 0, D
                gen_all!(co, |co_inner| self.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::D)));
            }
            0x83 => {
                // RES 0, E
                gen_all!(co, |co_inner| self.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::E)));
            }
            0x84 => {
                // RES 0, H
                gen_all!(co, |co_inner| self.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::H)));
            }
            0x85 => {
                // RES 0, L
                gen_all!(co, |co_inner| self.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::L)));
            }
            0x86 => {
                // RES 0, (HL)
                gen_all!(co, |co_inner| self.res(memory, co_inner, 0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x87 => {
                // RES 0, A
                gen_all!(co, |co_inner| self.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::A)));
            }
            0x88 => {
                // RES 1, B
                gen_all!(co, |co_inner| self.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::B)));
            }
            0x89 => {
                // RES 1, C
                gen_all!(co, |co_inner| self.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::C)));
            }
            0x8A => {
                // RES 1, D
                gen_all!(co, |co_inner| self.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::D)));
            }
            0x8B => {
                // RES 1, E
                gen_all!(co, |co_inner| self.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::E)));
            }
            0x8C => {
                // RES 1, H
                gen_all!(co, |co_inner| self.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::H)));
            }
            0x8D => {
                // RES 1, L
                gen_all!(co, |co_inner| self.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::L)));
            }
            0x8E => {
                // RES 1, (HL)
                gen_all!(co, |co_inner| self.res(memory, co_inner, 1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x8F => {
                // RES 1, A
                gen_all!(co, |co_inner| self.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::A)));
            }

            0x90 => {
                // RES 2, B
                gen_all!(co, |co_inner| self.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::B)));
            }
            0x91 => {
                // RES 2, C
                gen_all!(co, |co_inner| self.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::C)));
            }
            0x92 => {
                // RES 2, D
                gen_all!(co, |co_inner| self.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::D)));
            }
            0x93 => {
                // RES 2, E
                gen_all!(co, |co_inner| self.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::E)));
            }
            0x94 => {
                // RES 2, H
                gen_all!(co, |co_inner| self.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::H)));
            }
            0x95 => {
                // RES 2, L
                gen_all!(co, |co_inner| self.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::L)));
            }
            0x96 => {
                // RES 2, (HL)
                gen_all!(co, |co_inner| self.res(memory, co_inner, 2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x97 => {
                // RES 2, A
                gen_all!(co, |co_inner| self.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::A)));
            }
            0x98 => {
                // RES 3, B
                gen_all!(co, |co_inner| self.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::B)));
            }
            0x99 => {
                // RES 3, C
                gen_all!(co, |co_inner| self.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::C)));
            }
            0x9A => {
                // RES 3, D
                gen_all!(co, |co_inner| self.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::D)));
            }
            0x9B => {
                // RES 3, E
                gen_all!(co, |co_inner| self.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::E)));
            }
            0x9C => {
                // RES 3, H
                gen_all!(co, |co_inner| self.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::H)));
            }
            0x9D => {
                // RES 3, L
                gen_all!(co, |co_inner| self.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::L)));
            }
            0x9E => {
                // RES 3, (HL)
                gen_all!(co, |co_inner| self.res(memory, co_inner, 3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0x9F => {
                // RES 3, A
                gen_all!(co, |co_inner| self.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::A)));
            }

            0xA0 => {
                // RES 4, B
                gen_all!(co, |co_inner| self.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::B)));
            }
            0xA1 => {
                // RES 4, C
                gen_all!(co, |co_inner| self.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::C)));
            }
            0xA2 => {
                // RES 4, D
                gen_all!(co, |co_inner| self.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::D)));
            }
            0xA3 => {
                // RES 4, E
                gen_all!(co, |co_inner| self.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::E)));
            }
            0xA4 => {
                // RES 4, H
                gen_all!(co, |co_inner| self.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::H)));
            }
            0xA5 => {
                // RES 4, L
                gen_all!(co, |co_inner| self.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::L)));
            }
            0xA6 => {
                // RES 4, (HL)
                gen_all!(co, |co_inner| self.res(memory, co_inner, 4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xA7 => {
                // RES 4, A
                gen_all!(co, |co_inner| self.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::A)));
            }
            0xA8 => {
                // RES 5, B
                gen_all!(co, |co_inner| self.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::B)));
            }
            0xA9 => {
                // RES 5, C
                gen_all!(co, |co_inner| self.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::C)));
            }
            0xAA => {
                // RES 5, D
                gen_all!(co, |co_inner| self.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::D)));
            }
            0xAB => {
                // RES 5, E
                gen_all!(co, |co_inner| self.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::E)));
            }
            0xAC => {
                // RES 5, H
                gen_all!(co, |co_inner| self.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::H)));
            }
            0xAD => {
                // RES 5, L
                gen_all!(co, |co_inner| self.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::L)));
            }
            0xAE => {
                // RES 5, (HL)
                gen_all!(co, |co_inner| self.res(memory, co_inner, 5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xAF => {
                // RES 5, A
                gen_all!(co, |co_inner| self.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::A)));
            }

            0xB0 => {
                // RES 6, B
                gen_all!(co, |co_inner| self.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::B)));
            }
            0xB1 => {
                // RES 6, C
                gen_all!(co, |co_inner| self.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::C)));
            }
            0xB2 => {
                // RES 6, D
                gen_all!(co, |co_inner| self.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::D)));
            }
            0xB3 => {
                // RES 6, E
                gen_all!(co, |co_inner| self.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::E)));
            }
            0xB4 => {
                // RES 6, H
                gen_all!(co, |co_inner| self.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::H)));
            }
            0xB5 => {
                // RES 6, L
                gen_all!(co, |co_inner| self.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::L)));
            }
            0xB6 => {
                // RES 6, (HL)
                gen_all!(co, |co_inner| self.res(memory, co_inner, 6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xB7 => {
                // RES 6, A
                gen_all!(co, |co_inner| self.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::A)));
            }
            0xB8 => {
                // RES 7, B
                gen_all!(co, |co_inner| self.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::B)));
            }
            0xB9 => {
                // RES 7, C
                gen_all!(co, |co_inner| self.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::C)));
            }
            0xBA => {
                // RES 7, D
                gen_all!(co, |co_inner| self.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::D)));
            }
            0xBB => {
                // RES 7, E
                gen_all!(co, |co_inner| self.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::E)));
            }
            0xBC => {
                // RES 7, H
                gen_all!(co, |co_inner| self.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::H)));
            }
            0xBD => {
                // RES 7, L
                gen_all!(co, |co_inner| self.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::L)));
            }
            0xBE => {
                // RES 7, (HL)
                gen_all!(co, |co_inner| self.res(memory, co_inner, 7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xBF => {
                // RES 7, A
                gen_all!(co, |co_inner| self.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::A)));
            }

            0xC0 => {
                // SET 0, B
                gen_all!(co, |co_inner| self.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::B)));
            }
            0xC1 => {
                // SET 0, C
                gen_all!(co, |co_inner| self.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::C)));
            }
            0xC2 => {
                // SET 0, D
                gen_all!(co, |co_inner| self.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::D)));
            }
            0xC3 => {
                // SET 0, E
                gen_all!(co, |co_inner| self.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::E)));
            }
            0xC4 => {
                // SET 0, H
                gen_all!(co, |co_inner| self.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::H)));
            }
            0xC5 => {
                // SET 0, L
                gen_all!(co, |co_inner| self.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::L)));
            }
            0xC6 => {
                // SET 0, (HL)
                gen_all!(co, |co_inner| self.set(memory, co_inner, 0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xC7 => {
                // SET 0, A
                gen_all!(co, |co_inner| self.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::A)));
            }
            0xC8 => {
                // SET 1, B
                gen_all!(co, |co_inner| self.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::B)));
            }
            0xC9 => {
                // SET 1, C
                gen_all!(co, |co_inner| self.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::C)));
            }
            0xCA => {
                // SET 1, D
                gen_all!(co, |co_inner| self.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::D)));
            }
            0xCB => {
                // SET 1, E
                gen_all!(co, |co_inner| self.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::E)));
            }
            0xCC => {
                // SET 1, H
                gen_all!(co, |co_inner| self.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::H)));
            }
            0xCD => {
                // SET 1, L
                gen_all!(co, |co_inner| self.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::L)));
            }
            0xCE => {
                // SET 1, (HL)
                gen_all!(co, |co_inner| self.set(memory, co_inner, 1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xCF => {
                // SET 1, A
                gen_all!(co, |co_inner| self.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::A)));
            }

            0xD0 => {
                // SET 2, B
                gen_all!(co, |co_inner| self.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::B)));
            }
            0xD1 => {
                // SET 2, C
                gen_all!(co, |co_inner| self.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::C)));
            }
            0xD2 => {
                // SET 2, D
                gen_all!(co, |co_inner| self.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::D)));
            }
            0xD3 => {
                // SET 2, E
                gen_all!(co, |co_inner| self.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::E)));
            }
            0xD4 => {
                // SET 2, H
                gen_all!(co, |co_inner| self.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::H)));
            }
            0xD5 => {
                // SET 2, L
                gen_all!(co, |co_inner| self.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::L)));
            }
            0xD6 => {
                // SET 2, (HL)
                gen_all!(co, |co_inner| self.set(memory, co_inner, 2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xD7 => {
                // SET 2, A
                gen_all!(co, |co_inner| self.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::A)));
            }
            0xD8 => {
                // SET 3, B
                gen_all!(co, |co_inner| self.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::B)));
            }
            0xD9 => {
                // SET 3, C
                gen_all!(co, |co_inner| self.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::C)));
            }
            0xDA => {
                // SET 3, D
                gen_all!(co, |co_inner| self.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::D)));
            }
            0xDB => {
                // SET 3, E
                gen_all!(co, |co_inner| self.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::E)));
            }
            0xDC => {
                // SET 3, H
                gen_all!(co, |co_inner| self.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::H)));
            }
            0xDD => {
                // SET 3, L
                gen_all!(co, |co_inner| self.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::L)));
            }
            0xDE => {
                // SET 3, (HL)
                gen_all!(co, |co_inner| self.set(memory, co_inner, 3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xDF => {
                // SET 3, A
                gen_all!(co, |co_inner| self.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::A)));
            }

            0xE0 => {
                // SET 4, B
                gen_all!(co, |co_inner| self.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::B)));
            }
            0xE1 => {
                // SET 4, C
                gen_all!(co, |co_inner| self.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::C)));
            }
            0xE2 => {
                // SET 4, D
                gen_all!(co, |co_inner| self.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::D)));
            }
            0xE3 => {
                // SET 4, E
                gen_all!(co, |co_inner| self.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::E)));
            }
            0xE4 => {
                // SET 4, H
                gen_all!(co, |co_inner| self.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::H)));
            }
            0xE5 => {
                // SET 4, L
                gen_all!(co, |co_inner| self.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::L)));
            }
            0xE6 => {
                // SET 4, (HL)
                gen_all!(co, |co_inner| self.set(memory, co_inner, 4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xE7 => {
                // SET 4, A
                gen_all!(co, |co_inner| self.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::A)));
            }
            0xE8 => {
                // SET 5, B
                gen_all!(co, |co_inner| self.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::B)));
            }
            0xE9 => {
                // SET 5, C
                gen_all!(co, |co_inner| self.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::C)));
            }
            0xEA => {
                // SET 5, D
                gen_all!(co, |co_inner| self.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::D)));
            }
            0xEB => {
                // SET 5, E
                gen_all!(co, |co_inner| self.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::E)));
            }
            0xEC => {
                // SET 5, H
                gen_all!(co, |co_inner| self.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::H)));
            }
            0xED => {
                // SET 5, L
                gen_all!(co, |co_inner| self.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::L)));
            }
            0xEE => {
                // SET 5, (HL)
                gen_all!(co, |co_inner| self.set(memory, co_inner, 5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xEF => {
                // SET 5, A
                gen_all!(co, |co_inner| self.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::A)));
            }

            0xF0 => {
                // SET 6, B
                gen_all!(co, |co_inner| self.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::B)));
            }
            0xF1 => {
                // SET 6, C
                gen_all!(co, |co_inner| self.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::C)));
            }
            0xF2 => {
                // SET 6, D
                gen_all!(co, |co_inner| self.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::D)));
            }
            0xF3 => {
                // SET 6, E
                gen_all!(co, |co_inner| self.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::E)));
            }
            0xF4 => {
                // SET 6, H
                gen_all!(co, |co_inner| self.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::H)));
            }
            0xF5 => {
                // SET 6, L
                gen_all!(co, |co_inner| self.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::L)));
            }
            0xF6 => {
                // SET 6, (HL)
                gen_all!(co, |co_inner| self.set(memory, co_inner, 6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xF7 => {
                // SET 6, A
                gen_all!(co, |co_inner| self.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::A)));
            }
            0xF8 => {
                // SET 7, B
                gen_all!(co, |co_inner| self.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::B)));
            }
            0xF9 => {
                // SET 7, C
                gen_all!(co, |co_inner| self.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::C)));
            }
            0xFA => {
                // SET 7, D
                gen_all!(co, |co_inner| self.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::D)));
            }
            0xFB => {
                // SET 7, E
                gen_all!(co, |co_inner| self.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::E)));
            }
            0xFC => {
                // SET 7, H
                gen_all!(co, |co_inner| self.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::H)));
            }
            0xFD => {
                // SET 7, L
                gen_all!(co, |co_inner| self.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::L)));
            }
            0xFE => {
                // SET 7, (HL)
                gen_all!(co, |co_inner| self.set(memory, co_inner, 7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
            }
            0xFF => {
                // SET 7, A
                gen_all!(co, |co_inner| self.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::A)));
            }
        }
    }
}
