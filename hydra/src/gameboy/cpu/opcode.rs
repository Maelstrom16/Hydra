use std::{cell::RefCell, pin::Pin, rc::Rc};

use futures::FutureExt;
use genawaiter::stack::Co;

use crate::{
    gameboy::{
        cpu::{self, Cpu},
        memory::MemoryMap,
    },
    gen_all,
};

pub trait IntOperand<T> {
    async fn get(&self, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) -> T;
    async fn set(&self, value: T, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>);
}

pub struct RegisterOperand8(pub cpu::Register8);
impl IntOperand<u8> for RegisterOperand8 {
    #[inline(always)]
    async fn get(&self, cpu: &mut Cpu, _: &Rc<RefCell<MemoryMap>>, _co: Co<'_, ()>) -> u8 {
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
    async fn set(&self, value: u8, cpu: &mut Cpu, _: &Rc<RefCell<MemoryMap>>, _co: Co<'_, ()>) {
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
    async fn get(&self, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) -> u8 {
        gen_all!(&co, |co_inner| cpu.step_u8(memory, co_inner))
    }
    #[inline(always)]
    async fn set(&self, _: u8, _: &mut Cpu, _: &Rc<RefCell<MemoryMap>>, _co: Co<'_, ()>) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct ImmediateSignedOperand8;
impl IntOperand<i8> for ImmediateSignedOperand8 {
    #[inline(always)]
    async fn get(&self, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) -> i8 {
        gen_all!(&co, |co_inner| cpu.step_u8(memory, co_inner)) as i8
    }
    #[inline(always)]
    async fn set(&self, _: i8, _: &mut Cpu, _: &Rc<RefCell<MemoryMap>>, _co: Co<'_, ()>) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct IndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for IndirectOperand8<O> {
    #[inline(always)]
    async fn get(&self, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) -> u8 {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.read_u8(address, memory, co_inner))
    }
    #[inline(always)]
    async fn set(&self, value: u8, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.write_u8(address, value, memory, co_inner));
    }
}
pub struct IncIndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for IncIndirectOperand8<O> {
    #[inline(always)]
    async fn get(&self, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) -> u8 {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        gen_all!(co, |co_inner| self.0.set(address + 1, cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.read_u8(address, memory, co_inner))
    }
    #[inline(always)]
    async fn set(&self, value: u8, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        gen_all!(co, |co_inner| self.0.set(address + 1, cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.write_u8(address, value, memory, co_inner));
    }
}
pub struct DecIndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for DecIndirectOperand8<O> {
    #[inline(always)]
    async fn get(&self, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) -> u8 {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        gen_all!(co, |co_inner| self.0.set(address - 1, cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.read_u8(address, memory, co_inner))
    }
    #[inline(always)]
    async fn set(&self, value: u8, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        gen_all!(co, |co_inner| self.0.set(address - 1, cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.write_u8(address, value, memory, co_inner));
    }
}

pub struct HramIndirectOperand<O: IntOperand<u8>>(pub O);
impl<O: IntOperand<u8>> HramIndirectOperand<O> {
    #[inline(always)]
    async fn as_hram_address(&self, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) -> u16 {
        0xFF00 | (gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner)) as u16)
    }
}
impl<O: IntOperand<u8>> IntOperand<u8> for HramIndirectOperand<O> {
    #[inline(always)]
    async fn get(&self, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) -> u8 {
        let hram_address = gen_all!(co, |co_inner| self.as_hram_address(cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.read_u8(hram_address, memory, co_inner))
    }
    #[inline(always)]
    async fn set(&self, value: u8, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        let hram_address = gen_all!(co, |co_inner| self.as_hram_address(cpu, memory, co_inner));
        gen_all!(&co, |co_inner| cpu.write_u8(hram_address, value, memory, co_inner));
    }
}
impl<O: IntOperand<u16>> IntOperand<u16> for IndirectOperand8<O> {
    #[inline(always)]
    async fn get(&self, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) -> u16 {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        u16::from_le_bytes([
            gen_all!(&co, |co_inner| cpu.read_u8(address, memory, co_inner)),
            gen_all!(&co, |co_inner| cpu.read_u8(address + 1, memory, co_inner)),
        ])
    }
    #[inline(always)]
    async fn set(&self, value: u16, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        let address = gen_all!(co, |co_inner| self.0.get(cpu, memory, co_inner));
        let bytes = u16::to_le_bytes(value);
        gen_all!(&co, |co_inner| cpu.write_u8(address, bytes[0], memory, co_inner));
        gen_all!(&co, |co_inner| cpu.write_u8(address + 1, bytes[1], memory, co_inner));
    }
}

pub struct RegisterOperand16(pub cpu::Register16);
impl IntOperand<u16> for RegisterOperand16 {
    #[inline(always)]
    async fn get(&self, cpu: &mut Cpu, _: &Rc<RefCell<MemoryMap>>, _co: Co<'_, ()>) -> u16 {
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
    async fn set(&self, value: u16, cpu: &mut Cpu, _: &Rc<RefCell<MemoryMap>>, _co: Co<'_, ()>) {
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
    async fn get(&self, cpu: &mut Cpu, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) -> u16 {
        u16::from_le_bytes([gen_all!(&co, |co_inner| cpu.step_u8(memory, co_inner)), gen_all!(&co, |co_inner| cpu.step_u8(memory, co_inner))])
    }
    #[inline(always)]
    async fn set(&self, _: u16, _: &mut Cpu, _: &Rc<RefCell<MemoryMap>>, _co: Co<'_, ()>) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct ConstOperand16(pub u16);
impl IntOperand<u16> for ConstOperand16 {
    #[inline(always)]
    async fn get(&self, _: &mut Cpu, _: &Rc<RefCell<MemoryMap>>, _co: Co<'_, ()>) -> u16 {
        self.0
    }
    #[inline(always)]
    async fn set(&self, _: u16, _: &mut Cpu, _: &Rc<RefCell<MemoryMap>>, _co: Co<'_, ()>) {
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
    pub fn evaluate(&self, cpu: &Cpu) -> bool {
        match self {
            Self::Unconditional => true,
            Self::NZ => cpu.af[0] & 0b10000000 == 0,
            Self::Z => cpu.af[0] & 0b10000000 != 0,
            Self::NC => cpu.af[0] & 0b00010000 == 0,
            Self::C => cpu.af[0] & 0b00010000 != 0,
        }
    }
}

pub type OpcodeFuture<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;
pub type OpcodeFn = &'static (dyn for<'a> Fn(&'a mut Cpu, &'a Rc<RefCell<MemoryMap>>, Co<'a, ()>) -> OpcodeFuture<'a> + Sync);
pub type LocalOpcodeFn<'a> = Box<dyn FnOnce(&'a mut Cpu, &'a Rc<RefCell<MemoryMap>>, Co<'a, ()>) -> OpcodeFuture<'a>>;

pub static OP_TABLE: [OpcodeFn; 0x100] = generate_op();
static CB_TABLE: [OpcodeFn; 0x100] = generate_cb();
static INVALID: OpcodeFn = &|_, _, _| async move {panic!("Unknown opcode")}.boxed_local();

const fn generate_op() -> [OpcodeFn; 0x100] {
    let mut op_table = [INVALID; 0x100];
    op_table[0x00] = &|cpu, memory, co| async move { 
        // NOP
    }.boxed_local();
    op_table[0x01] = &|cpu, memory, co| async move {
        // LD BC, nn
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand16(cpu::Register16::BC), ImmediateOperand16));
    }.boxed_local();
    op_table[0x02] = &|cpu, memory, co| async move {
        // LD (BC), A
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            IndirectOperand8(RegisterOperand16(cpu::Register16::BC)),
            RegisterOperand8(cpu::Register8::A)
        ));
    }.boxed_local();
    op_table[0x03] = &|cpu, memory, co| async move {
        // INC BC
        gen_all!(co, |co_inner| cpu.inc16(memory, co_inner, RegisterOperand16(cpu::Register16::BC)));
    }.boxed_local();
    op_table[0x04] = &|cpu, memory, co| async move {
        // INC B
        gen_all!(co, |co_inner| cpu.inc(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x05] = &|cpu, memory, co| async move {
        // DEC B
        gen_all!(co, |co_inner| cpu.dec(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x06] = &|cpu, memory, co| async move {
        // LD B, n
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), ImmediateOperand8));
    }.boxed_local();
    op_table[0x07] = &|cpu, memory, co| async move {
        // RLCA
        gen_all!(co, |co_inner| cpu.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::A), false));
    }.boxed_local();
    op_table[0x08] = &|cpu, memory, co| async move {
        // LD (nn), SP
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, IndirectOperand8(ImmediateOperand16), RegisterOperand16(cpu::Register16::SP)));
    }.boxed_local();
    op_table[0x09] = &|cpu, memory, co| async move {
        // ADD HL, BC
        gen_all!(co, |co_inner| cpu.add_hl(memory, co_inner, RegisterOperand16(cpu::Register16::BC)));
    }.boxed_local();
    op_table[0x0A] = &|cpu, memory, co| async move {
        // LD A, (BC)
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            RegisterOperand8(cpu::Register8::A),
            IndirectOperand8(RegisterOperand16(cpu::Register16::BC))
        ));
    }.boxed_local();
    op_table[0x0B] = &|cpu, memory, co| async move {
        // DEC BC
        gen_all!(co, |co_inner| cpu.dec16(memory, co_inner, RegisterOperand16(cpu::Register16::BC)));
    }.boxed_local();
    op_table[0x0C] = &|cpu, memory, co| async move {
        // INC C
        gen_all!(co, |co_inner| cpu.inc(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x0D] = &|cpu, memory, co| async move {
        // DEC C
        gen_all!(co, |co_inner| cpu.dec(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x0E] = &|cpu, memory, co| async move {
        // LD C, n
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), ImmediateOperand8));
    }.boxed_local();
    op_table[0x0F] = &|cpu, memory, co| async move {
        // RRCA
        gen_all!(co, |co_inner| cpu.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::A), false));
    }.boxed_local();

    op_table[0x10] = &|cpu, memory, co| async move {
        // STOP
        gen_all!(co, |co_inner| cpu.stop(memory, co_inner));
    }.boxed_local();
    op_table[0x11] = &|cpu, memory, co| async move {
        // LD DE, nn
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand16(cpu::Register16::DE), ImmediateOperand16));
    }.boxed_local();
    op_table[0x12] = &|cpu, memory, co| async move {
        // LD (DE), A
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            IndirectOperand8(RegisterOperand16(cpu::Register16::DE)),
            RegisterOperand8(cpu::Register8::A)
        ));
    }.boxed_local();
    op_table[0x13] = &|cpu, memory, co| async move {
        // INC DE
        gen_all!(co, |co_inner| cpu.inc16(memory, co_inner, RegisterOperand16(cpu::Register16::DE)));
    }.boxed_local();
    op_table[0x14] = &|cpu, memory, co| async move {
        // INC D
        gen_all!(co, |co_inner| cpu.inc(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x15] = &|cpu, memory, co| async move {
        // DEC D
        gen_all!(co, |co_inner| cpu.dec(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x16] = &|cpu, memory, co| async move {
        // LD D, n
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), ImmediateOperand8));
    }.boxed_local();
    op_table[0x17] = &|cpu, memory, co| async move {
        // RLA
        gen_all!(co, |co_inner| cpu.rl(memory, co_inner, RegisterOperand8(cpu::Register8::A), false));
    }.boxed_local();
    op_table[0x18] = &|cpu, memory, co| async move {
        // JR e
        gen_all!(co, |co_inner| cpu.jr(memory, co_inner, CondOperand::Unconditional, ImmediateSignedOperand8));
    }.boxed_local();
    op_table[0x19] = &|cpu, memory, co| async move {
        // ADD HL, DE
        gen_all!(co, |co_inner| cpu.add_hl(memory, co_inner, RegisterOperand16(cpu::Register16::DE)));
    }.boxed_local();
    op_table[0x1A] = &|cpu, memory, co| async move {
        // LD A, (DE)
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            RegisterOperand8(cpu::Register8::A),
            IndirectOperand8(RegisterOperand16(cpu::Register16::DE))
        ));
    }.boxed_local();
    op_table[0x1B] = &|cpu, memory, co| async move {
        // DEC DE
        gen_all!(co, |co_inner| cpu.dec16(memory, co_inner, RegisterOperand16(cpu::Register16::DE)));
    }.boxed_local();
    op_table[0x1C] = &|cpu, memory, co| async move {
        // INC E
        gen_all!(co, |co_inner| cpu.inc(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x1D] = &|cpu, memory, co| async move {
        // DEC E
        gen_all!(co, |co_inner| cpu.dec(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x1E] = &|cpu, memory, co| async move {
        // LD E, n
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), ImmediateOperand8));
    }.boxed_local();
    op_table[0x1F] = &|cpu, memory, co| async move {
        // RRA
        gen_all!(co, |co_inner| cpu.rr(memory, co_inner, RegisterOperand8(cpu::Register8::A), false));
    }.boxed_local();

    op_table[0x20] = &|cpu, memory, co| async move {
        // JR NZ, e
        gen_all!(co, |co_inner| cpu.jr(memory, co_inner, CondOperand::NZ, ImmediateSignedOperand8));
    }.boxed_local();
    op_table[0x21] = &|cpu, memory, co| async move {
        // LD HL, nn
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand16(cpu::Register16::HL), ImmediateOperand16));
    }.boxed_local();
    op_table[0x22] = &|cpu, memory, co| async move {
        // LD (HL+), A
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            IncIndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
            RegisterOperand8(cpu::Register8::A)
        ));
    }.boxed_local();
    op_table[0x23] = &|cpu, memory, co| async move {
        // INC HL
        gen_all!(co, |co_inner| cpu.inc16(memory, co_inner, RegisterOperand16(cpu::Register16::HL)));
    }.boxed_local();
    op_table[0x24] = &|cpu, memory, co| async move {
        // INC H
        gen_all!(co, |co_inner| cpu.inc(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x25] = &|cpu, memory, co| async move {
        // DEC H
        gen_all!(co, |co_inner| cpu.dec(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x26] = &|cpu, memory, co| async move {
        // LD H, n
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), ImmediateOperand8));
    }.boxed_local();
    op_table[0x27] = &|cpu, memory, co| async move {
        // DAA
        gen_all!(co, |co_inner| cpu.daa(memory, co_inner));
    }.boxed_local();
    op_table[0x28] = &|cpu, memory, co| async move {
        // JR Z, e
        gen_all!(co, |co_inner| cpu.jr(memory, co_inner, CondOperand::Z, ImmediateSignedOperand8));
    }.boxed_local();
    op_table[0x29] = &|cpu, memory, co| async move {
        // ADD HL, HL
        gen_all!(co, |co_inner| cpu.add_hl(memory, co_inner, RegisterOperand16(cpu::Register16::HL)));
    }.boxed_local();
    op_table[0x2A] = &|cpu, memory, co| async move {
        // LD A, (HL+)
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            RegisterOperand8(cpu::Register8::A),
            IncIndirectOperand8(RegisterOperand16(cpu::Register16::HL))
        ));
    }.boxed_local();
    op_table[0x2B] = &|cpu, memory, co| async move {
        // DEC HL
        gen_all!(co, |co_inner| cpu.dec16(memory, co_inner, RegisterOperand16(cpu::Register16::HL)));
    }.boxed_local();
    op_table[0x2C] = &|cpu, memory, co| async move {
        // INC L
        gen_all!(co, |co_inner| cpu.inc(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x2D] = &|cpu, memory, co| async move {
        // DEC L
        gen_all!(co, |co_inner| cpu.dec(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x2E] = &|cpu, memory, co| async move {
        // LD L, n
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), ImmediateOperand8));
    }.boxed_local();
    op_table[0x2F] = &|cpu, memory, co| async move {
        // CPL
        gen_all!(co, |co_inner| cpu.cpl(memory, co_inner));
    }.boxed_local();

    op_table[0x30] = &|cpu, memory, co| async move {
        // JR NC, e
        gen_all!(co, |co_inner| cpu.jr(memory, co_inner, CondOperand::NC, ImmediateSignedOperand8));
    }.boxed_local();
    op_table[0x31] = &|cpu, memory, co| async move {
        // LD SP, nn
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand16(cpu::Register16::SP), ImmediateOperand16));
    }.boxed_local();
    op_table[0x32] = &|cpu, memory, co| async move {
        // LD (HL-), A
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            DecIndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
            RegisterOperand8(cpu::Register8::A)
        ));
    }.boxed_local();
    op_table[0x33] = &|cpu, memory, co| async move {
        // INC SP
        gen_all!(co, |co_inner| cpu.inc16(memory, co_inner, RegisterOperand16(cpu::Register16::SP)));
    }.boxed_local();
    op_table[0x34] = &|cpu, memory, co| async move {
        // INC (HL)
        gen_all!(co, |co_inner| cpu.inc(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x35] = &|cpu, memory, co| async move {
        // DEC (HL)
        gen_all!(co, |co_inner| cpu.dec(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x36] = &|cpu, memory, co| async move {
        // LD (HL), n
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), ImmediateOperand8));
    }.boxed_local();
    op_table[0x37] = &|cpu, memory, co| async move {
        // SCF
        gen_all!(co, |co_inner| cpu.scf(memory, co_inner));
    }.boxed_local();
    op_table[0x38] = &|cpu, memory, co| async move {
        // JR C, e
        gen_all!(co, |co_inner| cpu.jr(memory, co_inner, CondOperand::C, ImmediateSignedOperand8));
    }.boxed_local();
    op_table[0x39] = &|cpu, memory, co| async move {
        // ADD HL, SP
        gen_all!(co, |co_inner| cpu.add_hl(memory, co_inner, RegisterOperand16(cpu::Register16::SP)));
    }.boxed_local();
    op_table[0x3A] = &|cpu, memory, co| async move {
        // LD A, (HL-)
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            RegisterOperand8(cpu::Register8::A),
            DecIndirectOperand8(RegisterOperand16(cpu::Register16::HL))
        ));
    }.boxed_local();
    op_table[0x3B] = &|cpu, memory, co| async move {
        // DEC SP
        gen_all!(co, |co_inner| cpu.dec16(memory, co_inner, RegisterOperand16(cpu::Register16::SP)));
    }.boxed_local();
    op_table[0x3C] = &|cpu, memory, co| async move {
        // INC A
        gen_all!(co, |co_inner| cpu.inc(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x3D] = &|cpu, memory, co| async move {
        // DEC A
        gen_all!(co, |co_inner| cpu.dec(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x3E] = &|cpu, memory, co| async move {
        // LD A, n
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), ImmediateOperand8));
    }.boxed_local();
    op_table[0x3F] = &|cpu, memory, co| async move {
        // CCF
        gen_all!(co, |co_inner| cpu.ccf(memory, co_inner));
    }.boxed_local();

    op_table[0x40] = &|cpu, memory, co| async move {
        // LD B, B
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x41] = &|cpu, memory, co| async move {
        // LD B, C
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x42] = &|cpu, memory, co| async move {
        // LD B, D
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x43] = &|cpu, memory, co| async move {
        // LD B, E
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x44] = &|cpu, memory, co| async move {
        // LD B, H
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x45] = &|cpu, memory, co| async move {
        // LD B, L
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x46] = &|cpu, memory, co| async move {
        // LD B, (HL)
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            RegisterOperand8(cpu::Register8::B),
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
        ));
    }.boxed_local();
    op_table[0x47] = &|cpu, memory, co| async move {
        // LD B, A
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x48] = &|cpu, memory, co| async move {
        // LD C, B
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x49] = &|cpu, memory, co| async move {
        // LD C, C
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x4A] = &|cpu, memory, co| async move {
        // LD C, D
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x4B] = &|cpu, memory, co| async move {
        // LD C, E
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x4C] = &|cpu, memory, co| async move {
        // LD C, H
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x4D] = &|cpu, memory, co| async move {
        // LD C, L
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x4E] = &|cpu, memory, co| async move {
        // LD C, (HL)
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            RegisterOperand8(cpu::Register8::C),
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
        ));
    }.boxed_local();
    op_table[0x4F] = &|cpu, memory, co| async move {
        // LD C, A
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0x50] = &|cpu, memory, co| async move {
        // LD D, B
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x51] = &|cpu, memory, co| async move {
        // LD D, C
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x52] = &|cpu, memory, co| async move {
        // LD D, D
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x53] = &|cpu, memory, co| async move {
        // LD D, E
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x54] = &|cpu, memory, co| async move {
        // LD D, H
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x55] = &|cpu, memory, co| async move {
        // LD D, L
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x56] = &|cpu, memory, co| async move {
        // LD D, (HL)
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            RegisterOperand8(cpu::Register8::D),
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
        ));
    }.boxed_local();
    op_table[0x57] = &|cpu, memory, co| async move {
        // LD D, A
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x58] = &|cpu, memory, co| async move {
        // LD E, B
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x59] = &|cpu, memory, co| async move {
        // LD E, C
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x5A] = &|cpu, memory, co| async move {
        // LD E, D
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x5B] = &|cpu, memory, co| async move {
        // LD E, E
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x5C] = &|cpu, memory, co| async move {
        // LD E, H
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x5D] = &|cpu, memory, co| async move {
        // LD E, L
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x5E] = &|cpu, memory, co| async move {
        // LD E, (HL)
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            RegisterOperand8(cpu::Register8::E),
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
        ));
    }.boxed_local();
    op_table[0x5F] = &|cpu, memory, co| async move {
        // LD E, A
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0x60] = &|cpu, memory, co| async move {
        // LD H, B
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x61] = &|cpu, memory, co| async move {
        // LD H, C
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x62] = &|cpu, memory, co| async move {
        // LD H, D
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x63] = &|cpu, memory, co| async move {
        // LD H, E
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x64] = &|cpu, memory, co| async move {
        // LD H, H
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x65] = &|cpu, memory, co| async move {
        // LD H, L
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x66] = &|cpu, memory, co| async move {
        // LD H, (HL)
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            RegisterOperand8(cpu::Register8::H),
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
        ));
    }.boxed_local();
    op_table[0x67] = &|cpu, memory, co| async move {
        // LD H, A
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x68] = &|cpu, memory, co| async move {
        // LD L, B
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x69] = &|cpu, memory, co| async move {
        // LD L, C
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x6A] = &|cpu, memory, co| async move {
        // LD L, D
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x6B] = &|cpu, memory, co| async move {
        // LD L, E
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x6C] = &|cpu, memory, co| async move {
        // LD L, H
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x6D] = &|cpu, memory, co| async move {
        // LD L, L
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x6E] = &|cpu, memory, co| async move {
        // LD L, (HL)
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            RegisterOperand8(cpu::Register8::L),
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
        ));
    }.boxed_local();
    op_table[0x6F] = &|cpu, memory, co| async move {
        // LD L, A
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0x70] = &|cpu, memory, co| async move {
        // LD (HL), B
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
            RegisterOperand8(cpu::Register8::B)
        ));
    }.boxed_local();
    op_table[0x71] = &|cpu, memory, co| async move {
        // LD (HL), C
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
            RegisterOperand8(cpu::Register8::C)
        ));
    }.boxed_local();
    op_table[0x72] = &|cpu, memory, co| async move {
        // LD (HL), D
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
            RegisterOperand8(cpu::Register8::D)
        ));
    }.boxed_local();
    op_table[0x73] = &|cpu, memory, co| async move {
        // LD (HL), E
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
            RegisterOperand8(cpu::Register8::E)
        ));
    }.boxed_local();
    op_table[0x74] = &|cpu, memory, co| async move {
        // LD (HL), H
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
            RegisterOperand8(cpu::Register8::H)
        ));
    }.boxed_local();
    op_table[0x75] = &|cpu, memory, co| async move {
        // LD (HL), L
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
            RegisterOperand8(cpu::Register8::L)
        ));
    }.boxed_local();
    op_table[0x76] = &|cpu, memory, co| async move {
        // HALT
        gen_all!(co, |co_inner| cpu.halt(memory, co_inner));
    }.boxed_local();
    op_table[0x77] = &|cpu, memory, co| async move {
        // LD (HL), A
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL)),
            RegisterOperand8(cpu::Register8::A)
        ));
    }.boxed_local();
    op_table[0x78] = &|cpu, memory, co| async move {
        // LD A, B
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x79] = &|cpu, memory, co| async move {
        // LD A, C
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x7A] = &|cpu, memory, co| async move {
        // LD A, D
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x7B] = &|cpu, memory, co| async move {
        // LD A, E
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x7C] = &|cpu, memory, co| async move {
        // LD A, H
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x7D] = &|cpu, memory, co| async move {
        // LD A, L
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x7E] = &|cpu, memory, co| async move {
        // LD A, (HL)
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            RegisterOperand8(cpu::Register8::A),
            IndirectOperand8(RegisterOperand16(cpu::Register16::HL))
        ));
    }.boxed_local();
    op_table[0x7F] = &|cpu, memory, co| async move {
        // LD A, A
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0x80] = &|cpu, memory, co| async move {
        // ADD B
        gen_all!(co, |co_inner| cpu.add(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x81] = &|cpu, memory, co| async move {
        // ADD C
        gen_all!(co, |co_inner| cpu.add(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x82] = &|cpu, memory, co| async move {
        // ADD D
        gen_all!(co, |co_inner| cpu.add(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x83] = &|cpu, memory, co| async move {
        // ADD E
        gen_all!(co, |co_inner| cpu.add(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x84] = &|cpu, memory, co| async move {
        // ADD H
        gen_all!(co, |co_inner| cpu.add(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x85] = &|cpu, memory, co| async move {
        // ADD L
        gen_all!(co, |co_inner| cpu.add(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x86] = &|cpu, memory, co| async move {
        // ADD (HL)
        gen_all!(co, |co_inner| cpu.add(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x87] = &|cpu, memory, co| async move {
        // ADD A
        gen_all!(co, |co_inner| cpu.add(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x88] = &|cpu, memory, co| async move {
        // ADC B
        gen_all!(co, |co_inner| cpu.adc(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x89] = &|cpu, memory, co| async move {
        // ADC C
        gen_all!(co, |co_inner| cpu.adc(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x8A] = &|cpu, memory, co| async move {
        // ADC D
        gen_all!(co, |co_inner| cpu.adc(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x8B] = &|cpu, memory, co| async move {
        // ADC E
        gen_all!(co, |co_inner| cpu.adc(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x8C] = &|cpu, memory, co| async move {
        // ADC H
        gen_all!(co, |co_inner| cpu.adc(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x8D] = &|cpu, memory, co| async move {
        // ADC L
        gen_all!(co, |co_inner| cpu.adc(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x8E] = &|cpu, memory, co| async move {
        // ADC (HL)
        gen_all!(co, |co_inner| cpu.adc(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x8F] = &|cpu, memory, co| async move {
        // ADC A
        gen_all!(co, |co_inner| cpu.adc(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0x90] = &|cpu, memory, co| async move {
        // SUB B
        gen_all!(co, |co_inner| cpu.sub(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x91] = &|cpu, memory, co| async move {
        // SUB C
        gen_all!(co, |co_inner| cpu.sub(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x92] = &|cpu, memory, co| async move {
        // SUB D
        gen_all!(co, |co_inner| cpu.sub(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x93] = &|cpu, memory, co| async move {
        // SUB E
        gen_all!(co, |co_inner| cpu.sub(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x94] = &|cpu, memory, co| async move {
        // SUB H
        gen_all!(co, |co_inner| cpu.sub(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x95] = &|cpu, memory, co| async move {
        // SUB L
        gen_all!(co, |co_inner| cpu.sub(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x96] = &|cpu, memory, co| async move {
        // SUB (HL)
        gen_all!(co, |co_inner| cpu.sub(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x97] = &|cpu, memory, co| async move {
        // SUB A
        gen_all!(co, |co_inner| cpu.sub(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x98] = &|cpu, memory, co| async move {
        // SBC B
        gen_all!(co, |co_inner| cpu.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x99] = &|cpu, memory, co| async move {
        // SBC C
        gen_all!(co, |co_inner| cpu.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x9A] = &|cpu, memory, co| async move {
        // SBC D
        gen_all!(co, |co_inner| cpu.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x9B] = &|cpu, memory, co| async move {
        // SBC E
        gen_all!(co, |co_inner| cpu.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x9C] = &|cpu, memory, co| async move {
        // SBC H
        gen_all!(co, |co_inner| cpu.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x9D] = &|cpu, memory, co| async move {
        // SBC L
        gen_all!(co, |co_inner| cpu.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x9E] = &|cpu, memory, co| async move {
        // SBC (HL)
        gen_all!(co, |co_inner| cpu.sbc(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x9F] = &|cpu, memory, co| async move {
        // SBC A
        gen_all!(co, |co_inner| cpu.sbc(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0xA0] = &|cpu, memory, co| async move {
        // AND B
        gen_all!(co, |co_inner| cpu.and(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xA1] = &|cpu, memory, co| async move {
        // AND C
        gen_all!(co, |co_inner| cpu.and(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xA2] = &|cpu, memory, co| async move {
        // AND D
        gen_all!(co, |co_inner| cpu.and(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xA3] = &|cpu, memory, co| async move {
        // AND E
        gen_all!(co, |co_inner| cpu.and(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xA4] = &|cpu, memory, co| async move {
        // AND H
        gen_all!(co, |co_inner| cpu.and(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xA5] = &|cpu, memory, co| async move {
        // AND L
        gen_all!(co, |co_inner| cpu.and(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xA6] = &|cpu, memory, co| async move {
        // AND (HL)
        gen_all!(co, |co_inner| cpu.and(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xA7] = &|cpu, memory, co| async move {
        // AND A
        gen_all!(co, |co_inner| cpu.and(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0xA8] = &|cpu, memory, co| async move {
        // XOR B
        gen_all!(co, |co_inner| cpu.xor(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xA9] = &|cpu, memory, co| async move {
        // XOR C
        gen_all!(co, |co_inner| cpu.xor(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xAA] = &|cpu, memory, co| async move {
        // XOR D
        gen_all!(co, |co_inner| cpu.xor(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xAB] = &|cpu, memory, co| async move {
        // XOR E
        gen_all!(co, |co_inner| cpu.xor(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xAC] = &|cpu, memory, co| async move {
        // XOR H
        gen_all!(co, |co_inner| cpu.xor(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xAD] = &|cpu, memory, co| async move {
        // XOR L
        gen_all!(co, |co_inner| cpu.xor(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xAE] = &|cpu, memory, co| async move {
        // XOR (HL)
        gen_all!(co, |co_inner| cpu.xor(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xAF] = &|cpu, memory, co| async move {
        // XOR A
        gen_all!(co, |co_inner| cpu.xor(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0xB0] = &|cpu, memory, co| async move {
        // OR B
        gen_all!(co, |co_inner| cpu.or(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xB1] = &|cpu, memory, co| async move {
        // OR C
        gen_all!(co, |co_inner| cpu.or(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xB2] = &|cpu, memory, co| async move {
        // OR D
        gen_all!(co, |co_inner| cpu.or(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xB3] = &|cpu, memory, co| async move {
        // OR E
        gen_all!(co, |co_inner| cpu.or(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xB4] = &|cpu, memory, co| async move {
        // OR H
        gen_all!(co, |co_inner| cpu.or(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xB5] = &|cpu, memory, co| async move {
        // OR L
        gen_all!(co, |co_inner| cpu.or(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xB6] = &|cpu, memory, co| async move {
        // OR (HL)
        gen_all!(co, |co_inner| cpu.or(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xB7] = &|cpu, memory, co| async move {
        // OR A
        gen_all!(co, |co_inner| cpu.or(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0xB8] = &|cpu, memory, co| async move {
        // CP B
        gen_all!(co, |co_inner| cpu.cp(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xB9] = &|cpu, memory, co| async move {
        // CP C
        gen_all!(co, |co_inner| cpu.cp(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xBA] = &|cpu, memory, co| async move {
        // CP D
        gen_all!(co, |co_inner| cpu.cp(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xBB] = &|cpu, memory, co| async move {
        // CP E
        gen_all!(co, |co_inner| cpu.cp(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xBC] = &|cpu, memory, co| async move {
        // CP H
        gen_all!(co, |co_inner| cpu.cp(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xBD] = &|cpu, memory, co| async move {
        // CP L
        gen_all!(co, |co_inner| cpu.cp(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xBE] = &|cpu, memory, co| async move {
        // CP (HL)
        gen_all!(co, |co_inner| cpu.cp(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xBF] = &|cpu, memory, co| async move {
        // CP A
        gen_all!(co, |co_inner| cpu.cp(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0xC0] = &|cpu, memory, co| async move {
        // RET NZ
        gen_all!(co, |co_inner| cpu.ret(memory, co_inner, CondOperand::NZ));
    }.boxed_local();
    op_table[0xC1] = &|cpu, memory, co| async move {
        // POP BC
        gen_all!(co, |co_inner| cpu.pop(memory, co_inner, RegisterOperand16(cpu::Register16::BC)));
    }.boxed_local();
    op_table[0xC2] = &|cpu, memory, co| async move {
        // JP NZ, nn
        gen_all!(co, |co_inner| cpu.jp(memory, co_inner, CondOperand::NZ, ImmediateOperand16));
    }.boxed_local();
    op_table[0xC3] = &|cpu, memory, co| async move {
        // JP nn
        gen_all!(co, |co_inner| cpu.jp(memory, co_inner, CondOperand::Unconditional, ImmediateOperand16));
    }.boxed_local();
    op_table[0xC4] = &|cpu, memory, co| async move {
        // CALL NZ, nn
        gen_all!(co, |co_inner| cpu.call(memory, co_inner, CondOperand::NZ, ImmediateOperand16));
    }.boxed_local();
    op_table[0xC5] = &|cpu, memory, co| async move {
        // PUSH BC
        gen_all!(co, |co_inner| cpu.push(memory, co_inner, RegisterOperand16(cpu::Register16::BC)));
    }.boxed_local();
    op_table[0xC6] = &|cpu, memory, co| async move {
        // ADD n
        gen_all!(co, |co_inner| cpu.add(memory, co_inner, ImmediateOperand8));
    }.boxed_local();
    op_table[0xC7] = &|cpu, memory, co| async move {
        // RST 0x00
        gen_all!(co, |co_inner| cpu.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0000)));
    }.boxed_local();
    op_table[0xC8] = &|cpu, memory, co| async move {
        // RET Z
        gen_all!(co, |co_inner| cpu.ret(memory, co_inner, CondOperand::Z));
    }.boxed_local();
    op_table[0xC9] = &|cpu, memory, co| async move {
        // RET
        gen_all!(co, |co_inner| cpu.ret(memory, co_inner, CondOperand::Unconditional));
    }.boxed_local();
    op_table[0xCA] = &|cpu, memory, co| async move {
        // JP Z, nn
        gen_all!(co, |co_inner| cpu.jp(memory, co_inner, CondOperand::Z, ImmediateOperand16));
    }.boxed_local();
    op_table[0xCB] = &|cpu, memory, co| async move {
        // CB op
        cpu.ir = gen_all!(&co, |co_inner| cpu.step_u8(memory, co_inner));
        gen_all!(co, |co_inner| CB_TABLE[cpu.ir as usize](cpu, memory, co_inner));
    }.boxed_local();
    op_table[0xCC] = &|cpu, memory, co| async move {
        // CALL Z, nn
        gen_all!(co, |co_inner| cpu.call(memory, co_inner, CondOperand::Z, ImmediateOperand16));
    }.boxed_local();
    op_table[0xCD] = &|cpu, memory, co| async move {
        // CALL nn
        gen_all!(co, |co_inner| cpu.call(memory, co_inner, CondOperand::Unconditional, ImmediateOperand16));
    }.boxed_local();
    op_table[0xCE] = &|cpu, memory, co| async move {
        // ADC n
        gen_all!(co, |co_inner| cpu.adc(memory, co_inner, ImmediateOperand8));
    }.boxed_local();
    op_table[0xCF] = &|cpu, memory, co| async move {
        // RST 0x08
        gen_all!(co, |co_inner| cpu.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0008)));
    }.boxed_local();

    op_table[0xD0] = &|cpu, memory, co| async move {
        // RET NC
        gen_all!(co, |co_inner| cpu.ret(memory, co_inner, CondOperand::NC));
    }.boxed_local();
    op_table[0xD1] = &|cpu, memory, co| async move {
        // POP DE
        gen_all!(co, |co_inner| cpu.pop(memory, co_inner, RegisterOperand16(cpu::Register16::DE)));
    }.boxed_local();
    op_table[0xD2] = &|cpu, memory, co| async move {
        // JP NC, nn
        gen_all!(co, |co_inner| cpu.jp(memory, co_inner, CondOperand::NC, ImmediateOperand16));
    }.boxed_local();
    // 0xD3 (invalid)
    op_table[0xD4] = &|cpu, memory, co| async move {
        // CALL NC, nn
        gen_all!(co, |co_inner| cpu.call(memory, co_inner, CondOperand::NC, ImmediateOperand16));
    }.boxed_local();
    op_table[0xD5] = &|cpu, memory, co| async move {
        // PUSH DE
        gen_all!(co, |co_inner| cpu.push(memory, co_inner, RegisterOperand16(cpu::Register16::DE)));
    }.boxed_local();
    op_table[0xD6] = &|cpu, memory, co| async move {
        // SUB n
        gen_all!(co, |co_inner| cpu.sub(memory, co_inner, ImmediateOperand8));
    }.boxed_local();
    op_table[0xD7] = &|cpu, memory, co| async move {
        // RST 0x10
        gen_all!(co, |co_inner| cpu.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0010)));
    }.boxed_local();
    op_table[0xD8] = &|cpu, memory, co| async move {
        // RET C
        gen_all!(co, |co_inner| cpu.ret(memory, co_inner, CondOperand::C));
    }.boxed_local();
    op_table[0xD9] = &|cpu, memory, co| async move {
        // RETI
        gen_all!(co, |co_inner| cpu.reti(memory, co_inner));
    }.boxed_local();
    op_table[0xDA] = &|cpu, memory, co| async move {
        // JP C, nn
        gen_all!(co, |co_inner| cpu.jp(memory, co_inner, CondOperand::C, ImmediateOperand16));
    }.boxed_local();
    // 0xDB (invalid)
    op_table[0xDC] = &|cpu, memory, co| async move {
        // CALL C, nn
        gen_all!(co, |co_inner| cpu.call(memory, co_inner, CondOperand::C, ImmediateOperand16));
    }.boxed_local();
    // 0xDD (invalid)
    op_table[0xDE] = &|cpu, memory, co| async move {
        // SBC n
        gen_all!(co, |co_inner| cpu.sbc(memory, co_inner, ImmediateOperand8));
    }.boxed_local();
    op_table[0xDF] = &|cpu, memory, co| async move {
        // RST 0x18
        gen_all!(co, |co_inner| cpu.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0018)));
    }.boxed_local();

    op_table[0xE0] = &|cpu, memory, co| async move {
        // LDH (n), A
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, HramIndirectOperand(ImmediateOperand8), RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0xE1] = &|cpu, memory, co| async move {
        // POP HL
        gen_all!(co, |co_inner| cpu.pop(memory, co_inner, RegisterOperand16(cpu::Register16::HL)));
    }.boxed_local();
    op_table[0xE2] = &|cpu, memory, co| async move {
        // LDH (C), A
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            HramIndirectOperand(RegisterOperand8(cpu::Register8::C)),
            RegisterOperand8(cpu::Register8::A)
        ));
    }.boxed_local();
    // 0xE3 (invalid)
    // 0xE4 (invalid)
    op_table[0xE5] = &|cpu, memory, co| async move {
        // PUSH HL
        gen_all!(co, |co_inner| cpu.push(memory, co_inner, RegisterOperand16(cpu::Register16::HL)));
    }.boxed_local();
    op_table[0xE6] = &|cpu, memory, co| async move {
        // AND n
        gen_all!(co, |co_inner| cpu.and(memory, co_inner, ImmediateOperand8));
    }.boxed_local();
    op_table[0xE7] = &|cpu, memory, co| async move {
        // RST 0x20
        gen_all!(co, |co_inner| cpu.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0020)));
    }.boxed_local();
    op_table[0xE8] = &|cpu, memory, co| async move {
        // ADD SP, e
        gen_all!(co, |co_inner| cpu.add_spe(memory, co_inner));
    }.boxed_local();
    op_table[0xE9] = &|cpu, memory, co| async move {
        // JP HL
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand16(cpu::Register16::PC), RegisterOperand16(cpu::Register16::HL)));
    }.boxed_local();
    op_table[0xEA] = &|cpu, memory, co| async move {
        // LD (nn), A
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, IndirectOperand8(ImmediateOperand16), RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    // 0xEB (invalid)
    // 0xEC (invalid)
    // 0xED (invalid)
    op_table[0xEE] = &|cpu, memory, co| async move {
        // XOR n
        gen_all!(co, |co_inner| cpu.xor(memory, co_inner, ImmediateOperand8));
    }.boxed_local();
    op_table[0xEF] = &|cpu, memory, co| async move {
        // RST 0x28
        gen_all!(co, |co_inner| cpu.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0028)));
    }.boxed_local();

    op_table[0xF0] = &|cpu, memory, co| async move {
        // LDH A, (n)
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), HramIndirectOperand(ImmediateOperand8)));
    }.boxed_local();
    op_table[0xF1] = &|cpu, memory, co| async move {
        // POP AF
        gen_all!(co, |co_inner| cpu.pop(memory, co_inner, RegisterOperand16(cpu::Register16::AF)));
    }.boxed_local();
    op_table[0xF2] = &|cpu, memory, co| async move {
        // LDH A, (C)
        gen_all!(co, |co_inner| cpu.ld(
            memory,
            co_inner,
            RegisterOperand8(cpu::Register8::A),
            HramIndirectOperand(RegisterOperand8(cpu::Register8::C))
        ));
    }.boxed_local();
    op_table[0xF3] = &|cpu, memory, co| async move {
        // DI
        gen_all!(co, |co_inner| cpu.di(memory, co_inner));
    }.boxed_local();
    // 0xF4 (invalid)
    op_table[0xF5] = &|cpu, memory, co| async move {
        // PUSH AF
        gen_all!(co, |co_inner| cpu.push(memory, co_inner, RegisterOperand16(cpu::Register16::AF)));
    }.boxed_local();
    op_table[0xF6] = &|cpu, memory, co| async move {
        // OR n
        gen_all!(co, |co_inner| cpu.or(memory, co_inner, ImmediateOperand8));
    }.boxed_local();
    op_table[0xF7] = &|cpu, memory, co| async move {
        // RST 0x30
        gen_all!(co, |co_inner| cpu.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0030)));
    }.boxed_local();
    op_table[0xF8] = &|cpu, memory, co| async move {
        // LD HL, SP+e
        gen_all!(co, |co_inner| cpu.ld_hlspe(memory, co_inner));
    }.boxed_local();
    op_table[0xF9] = &|cpu, memory, co| async move {
        // LD SP, HL
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand16(cpu::Register16::SP), RegisterOperand16(cpu::Register16::HL)));
        co.yield_(()).await;
    }.boxed_local();
    op_table[0xFA] = &|cpu, memory, co| async move {
        // LD A, (nn)
        gen_all!(co, |co_inner| cpu.ld(memory, co_inner, RegisterOperand8(cpu::Register8::A), IndirectOperand8(ImmediateOperand16)));
    }.boxed_local();
    op_table[0xFB] = &|cpu, memory, co| async move {
        // EI
        gen_all!(co, |co_inner| cpu.ei(memory, co_inner));
    }.boxed_local();
    // 0xFC (invalid)
    // 0xFD (invalid)
    op_table[0xFE] = &|cpu, memory, co| async move {
        // CP n
        gen_all!(co, |co_inner| cpu.cp(memory, co_inner, ImmediateOperand8));
    }.boxed_local();
    op_table[0xFF] = &|cpu, memory, co| async move {
        // RST 0x38
        gen_all!(co, |co_inner| cpu.call(memory, co_inner, CondOperand::Unconditional, ConstOperand16(0x0038)));
    }.boxed_local();

    op_table
}

const fn generate_cb() -> [OpcodeFn; 0x100] {
    let mut op_table = [INVALID; 0x100];
    op_table[0x00] = &|cpu, memory, co| async move {
        // RLC B
        gen_all!(co, |co_inner| cpu.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::B), true));
    }.boxed_local();
    op_table[0x01] = &|cpu, memory, co| async move {
        // RLC C
        gen_all!(co, |co_inner| cpu.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::C), true));
    }.boxed_local();
    op_table[0x02] = &|cpu, memory, co| async move {
        // RLC D
        gen_all!(co, |co_inner| cpu.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::D), true));
    }.boxed_local();
    op_table[0x03] = &|cpu, memory, co| async move {
        // RLC E
        gen_all!(co, |co_inner| cpu.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::E), true));
    }.boxed_local();
    op_table[0x04] = &|cpu, memory, co| async move {
        // RLC H
        gen_all!(co, |co_inner| cpu.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::H), true));
    }.boxed_local();
    op_table[0x05] = &|cpu, memory, co| async move {
        // RLC L
        gen_all!(co, |co_inner| cpu.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::L), true));
    }.boxed_local();
    op_table[0x06] = &|cpu, memory, co| async move {
        // RLC (HL)
        gen_all!(co, |co_inner| cpu.rlc(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), true));
    }.boxed_local();
    op_table[0x07] = &|cpu, memory, co| async move {
        // RLC A
        gen_all!(co, |co_inner| cpu.rlc(memory, co_inner, RegisterOperand8(cpu::Register8::A), true));
    }.boxed_local();
    op_table[0x08] = &|cpu, memory, co| async move {
        // RRC B
        gen_all!(co, |co_inner| cpu.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::B), true));
    }.boxed_local();
    op_table[0x09] = &|cpu, memory, co| async move {
        // RRC C
        gen_all!(co, |co_inner| cpu.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::C), true));
    }.boxed_local();
    op_table[0x0A] = &|cpu, memory, co| async move {
        // RRC D
        gen_all!(co, |co_inner| cpu.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::D), true));
    }.boxed_local();
    op_table[0x0B] = &|cpu, memory, co| async move {
        // RRC E
        gen_all!(co, |co_inner| cpu.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::E), true));
    }.boxed_local();
    op_table[0x0C] = &|cpu, memory, co| async move {
        // RRC H
        gen_all!(co, |co_inner| cpu.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::H), true));
    }.boxed_local();
    op_table[0x0D] = &|cpu, memory, co| async move {
        // RRC L
        gen_all!(co, |co_inner| cpu.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::L), true));
    }.boxed_local();
    op_table[0x0E] = &|cpu, memory, co| async move {
        // RRC (HL)
        gen_all!(co, |co_inner| cpu.rrc(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), true));
    }.boxed_local();
    op_table[0x0F] = &|cpu, memory, co| async move {
        // RRC A
        gen_all!(co, |co_inner| cpu.rrc(memory, co_inner, RegisterOperand8(cpu::Register8::A), true));
    }.boxed_local();

    op_table[0x10] = &|cpu, memory, co| async move {
        // RL B
        gen_all!(co, |co_inner| cpu.rl(memory, co_inner, RegisterOperand8(cpu::Register8::B), true));
    }.boxed_local();
    op_table[0x11] = &|cpu, memory, co| async move {
        // RL C
        gen_all!(co, |co_inner| cpu.rl(memory, co_inner, RegisterOperand8(cpu::Register8::C), true));
    }.boxed_local();
    op_table[0x12] = &|cpu, memory, co| async move {
        // RL D
        gen_all!(co, |co_inner| cpu.rl(memory, co_inner, RegisterOperand8(cpu::Register8::D), true));
    }.boxed_local();
    op_table[0x13] = &|cpu, memory, co| async move {
        // RL E
        gen_all!(co, |co_inner| cpu.rl(memory, co_inner, RegisterOperand8(cpu::Register8::E), true));
    }.boxed_local();
    op_table[0x14] = &|cpu, memory, co| async move {
        // RL H
        gen_all!(co, |co_inner| cpu.rl(memory, co_inner, RegisterOperand8(cpu::Register8::H), true));
    }.boxed_local();
    op_table[0x15] = &|cpu, memory, co| async move {
        // RL L
        gen_all!(co, |co_inner| cpu.rl(memory, co_inner, RegisterOperand8(cpu::Register8::L), true));
    }.boxed_local();
    op_table[0x16] = &|cpu, memory, co| async move {
        // RL (HL)
        gen_all!(co, |co_inner| cpu.rl(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), true));
    }.boxed_local();
    op_table[0x17] = &|cpu, memory, co| async move {
        // RL A
        gen_all!(co, |co_inner| cpu.rl(memory, co_inner, RegisterOperand8(cpu::Register8::A), true));
    }.boxed_local();
    op_table[0x18] = &|cpu, memory, co| async move {
        // RR B
        gen_all!(co, |co_inner| cpu.rr(memory, co_inner, RegisterOperand8(cpu::Register8::B), true));
    }.boxed_local();
    op_table[0x19] = &|cpu, memory, co| async move {
        // RR C
        gen_all!(co, |co_inner| cpu.rr(memory, co_inner, RegisterOperand8(cpu::Register8::C), true));
    }.boxed_local();
    op_table[0x1A] = &|cpu, memory, co| async move {
        // RR D
        gen_all!(co, |co_inner| cpu.rr(memory, co_inner, RegisterOperand8(cpu::Register8::D), true));
    }.boxed_local();
    op_table[0x1B] = &|cpu, memory, co| async move {
        // RR E
        gen_all!(co, |co_inner| cpu.rr(memory, co_inner, RegisterOperand8(cpu::Register8::E), true));
    }.boxed_local();
    op_table[0x1C] = &|cpu, memory, co| async move {
        // RR H
        gen_all!(co, |co_inner| cpu.rr(memory, co_inner, RegisterOperand8(cpu::Register8::H), true));
    }.boxed_local();
    op_table[0x1D] = &|cpu, memory, co| async move {
        // RR L
        gen_all!(co, |co_inner| cpu.rr(memory, co_inner, RegisterOperand8(cpu::Register8::L), true));
    }.boxed_local();
    op_table[0x1E] = &|cpu, memory, co| async move {
        // RR (HL)
        gen_all!(co, |co_inner| cpu.rr(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), true));
    }.boxed_local();
    op_table[0x1F] = &|cpu, memory, co| async move {
        // RR A
        gen_all!(co, |co_inner| cpu.rr(memory, co_inner, RegisterOperand8(cpu::Register8::A), true));
    }.boxed_local();

    op_table[0x20] = &|cpu, memory, co| async move {
        // SLA B
        gen_all!(co, |co_inner| cpu.sla(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x21] = &|cpu, memory, co| async move {
        // SLA C
        gen_all!(co, |co_inner| cpu.sla(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x22] = &|cpu, memory, co| async move {
        // SLA D
        gen_all!(co, |co_inner| cpu.sla(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x23] = &|cpu, memory, co| async move {
        // SLA E
        gen_all!(co, |co_inner| cpu.sla(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x24] = &|cpu, memory, co| async move {
        // SLA H
        gen_all!(co, |co_inner| cpu.sla(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x25] = &|cpu, memory, co| async move {
        // SLA L
        gen_all!(co, |co_inner| cpu.sla(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x26] = &|cpu, memory, co| async move {
        // SLA (HL)
        gen_all!(co, |co_inner| cpu.sla(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x27] = &|cpu, memory, co| async move {
        // SLA A
        gen_all!(co, |co_inner| cpu.sla(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x28] = &|cpu, memory, co| async move {
        // SRA B
        gen_all!(co, |co_inner| cpu.sra(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x29] = &|cpu, memory, co| async move {
        // SRA C
        gen_all!(co, |co_inner| cpu.sra(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x2A] = &|cpu, memory, co| async move {
        // SRA D
        gen_all!(co, |co_inner| cpu.sra(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x2B] = &|cpu, memory, co| async move {
        // SRA E
        gen_all!(co, |co_inner| cpu.sra(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x2C] = &|cpu, memory, co| async move {
        // SRA H
        gen_all!(co, |co_inner| cpu.sra(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x2D] = &|cpu, memory, co| async move {
        // SRA L
        gen_all!(co, |co_inner| cpu.sra(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x2E] = &|cpu, memory, co| async move {
        // SRA (HL)
        gen_all!(co, |co_inner| cpu.sra(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x2F] = &|cpu, memory, co| async move {
        // SRA A
        gen_all!(co, |co_inner| cpu.sra(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0x30] = &|cpu, memory, co| async move {
        // SWAP B
        gen_all!(co, |co_inner| cpu.swap(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x31] = &|cpu, memory, co| async move {
        // SWAP C
        gen_all!(co, |co_inner| cpu.swap(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x32] = &|cpu, memory, co| async move {
        // SWAP D
        gen_all!(co, |co_inner| cpu.swap(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x33] = &|cpu, memory, co| async move {
        // SWAP E
        gen_all!(co, |co_inner| cpu.swap(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x34] = &|cpu, memory, co| async move {
        // SWAP H
        gen_all!(co, |co_inner| cpu.swap(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x35] = &|cpu, memory, co| async move {
        // SWAP L
        gen_all!(co, |co_inner| cpu.swap(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x36] = &|cpu, memory, co| async move {
        // SWAP (HL)
        gen_all!(co, |co_inner| cpu.swap(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x37] = &|cpu, memory, co| async move {
        // SWAP A
        gen_all!(co, |co_inner| cpu.swap(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x38] = &|cpu, memory, co| async move {
        // SRL B
        gen_all!(co, |co_inner| cpu.srl(memory, co_inner, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x39] = &|cpu, memory, co| async move {
        // SRL C
        gen_all!(co, |co_inner| cpu.srl(memory, co_inner, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x3A] = &|cpu, memory, co| async move {
        // SRL D
        gen_all!(co, |co_inner| cpu.srl(memory, co_inner, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x3B] = &|cpu, memory, co| async move {
        // SRL E
        gen_all!(co, |co_inner| cpu.srl(memory, co_inner, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x3C] = &|cpu, memory, co| async move {
        // SRL H
        gen_all!(co, |co_inner| cpu.srl(memory, co_inner, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x3D] = &|cpu, memory, co| async move {
        // SRL L
        gen_all!(co, |co_inner| cpu.srl(memory, co_inner, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x3E] = &|cpu, memory, co| async move {
        // SRL (HL)
        gen_all!(co, |co_inner| cpu.srl(memory, co_inner, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x3F] = &|cpu, memory, co| async move {
        // SRL A
        gen_all!(co, |co_inner| cpu.srl(memory, co_inner, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0x40] = &|cpu, memory, co| async move {
        // BIT 0, B
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x41] = &|cpu, memory, co| async move {
        // BIT 0, C
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x42] = &|cpu, memory, co| async move {
        // BIT 0, D
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x43] = &|cpu, memory, co| async move {
        // BIT 0, E
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x44] = &|cpu, memory, co| async move {
        // BIT 0, H
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x45] = &|cpu, memory, co| async move {
        // BIT 0, L
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x46] = &|cpu, memory, co| async move {
        // BIT 0, (HL)
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x47] = &|cpu, memory, co| async move {
        // BIT 0, A
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 0, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x48] = &|cpu, memory, co| async move {
        // BIT 1, B
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x49] = &|cpu, memory, co| async move {
        // BIT 1, C
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x4A] = &|cpu, memory, co| async move {
        // BIT 1, D
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x4B] = &|cpu, memory, co| async move {
        // BIT 1, E
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x4C] = &|cpu, memory, co| async move {
        // BIT 1, H
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x4D] = &|cpu, memory, co| async move {
        // BIT 1, L
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x4E] = &|cpu, memory, co| async move {
        // BIT 1, (HL)
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x4F] = &|cpu, memory, co| async move {
        // BIT 1, A
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 1, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0x50] = &|cpu, memory, co| async move {
        // BIT 2, B
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x51] = &|cpu, memory, co| async move {
        // BIT 2, C
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x52] = &|cpu, memory, co| async move {
        // BIT 2, D
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x53] = &|cpu, memory, co| async move {
        // BIT 2, E
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x54] = &|cpu, memory, co| async move {
        // BIT 2, H
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x55] = &|cpu, memory, co| async move {
        // BIT 2, L
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x56] = &|cpu, memory, co| async move {
        // BIT 2, (HL)
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x57] = &|cpu, memory, co| async move {
        // BIT 2, A
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 2, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x58] = &|cpu, memory, co| async move {
        // BIT 3, B
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x59] = &|cpu, memory, co| async move {
        // BIT 3, C
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x5A] = &|cpu, memory, co| async move {
        // BIT 3, D
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x5B] = &|cpu, memory, co| async move {
        // BIT 3, E
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x5C] = &|cpu, memory, co| async move {
        // BIT 3, H
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x5D] = &|cpu, memory, co| async move {
        // BIT 3, L
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x5E] = &|cpu, memory, co| async move {
        // BIT 3, (HL)
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x5F] = &|cpu, memory, co| async move {
        // BIT 3, A
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 3, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0x60] = &|cpu, memory, co| async move {
        // BIT 4, B
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x61] = &|cpu, memory, co| async move {
        // BIT 4, C
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x62] = &|cpu, memory, co| async move {
        // BIT 4, D
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x63] = &|cpu, memory, co| async move {
        // BIT 4, E
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x64] = &|cpu, memory, co| async move {
        // BIT 4, H
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x65] = &|cpu, memory, co| async move {
        // BIT 4, L
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x66] = &|cpu, memory, co| async move {
        // BIT 4, (HL)
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x67] = &|cpu, memory, co| async move {
        // BIT 4, A
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 4, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x68] = &|cpu, memory, co| async move {
        // BIT 5, B
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x69] = &|cpu, memory, co| async move {
        // BIT 5, C
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x6A] = &|cpu, memory, co| async move {
        // BIT 5, D
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x6B] = &|cpu, memory, co| async move {
        // BIT 5, E
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x6C] = &|cpu, memory, co| async move {
        // BIT 5, H
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x6D] = &|cpu, memory, co| async move {
        // BIT 5, L
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x6E] = &|cpu, memory, co| async move {
        // BIT 5, (HL)
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x6F] = &|cpu, memory, co| async move {
        // BIT 5, A
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 5, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0x70] = &|cpu, memory, co| async move {
        // BIT 6, B
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x71] = &|cpu, memory, co| async move {
        // BIT 6, C
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x72] = &|cpu, memory, co| async move {
        // BIT 6, D
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x73] = &|cpu, memory, co| async move {
        // BIT 6, E
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x74] = &|cpu, memory, co| async move {
        // BIT 6, H
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x75] = &|cpu, memory, co| async move {
        // BIT 6, L
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x76] = &|cpu, memory, co| async move {
        // BIT 6, (HL)
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x77] = &|cpu, memory, co| async move {
        // BIT 6, A
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 6, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x78] = &|cpu, memory, co| async move {
        // BIT 7, B
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x79] = &|cpu, memory, co| async move {
        // BIT 7, C
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x7A] = &|cpu, memory, co| async move {
        // BIT 7, D
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x7B] = &|cpu, memory, co| async move {
        // BIT 7, E
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x7C] = &|cpu, memory, co| async move {
        // BIT 7, H
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x7D] = &|cpu, memory, co| async move {
        // BIT 7, L
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x7E] = &|cpu, memory, co| async move {
        // BIT 7, (HL)
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x7F] = &|cpu, memory, co| async move {
        // BIT 7, A
        gen_all!(co, |co_inner| cpu.bit(memory, co_inner, 7, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0x80] = &|cpu, memory, co| async move {
        // RES 0, B
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x81] = &|cpu, memory, co| async move {
        // RES 0, C
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x82] = &|cpu, memory, co| async move {
        // RES 0, D
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x83] = &|cpu, memory, co| async move {
        // RES 0, E
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x84] = &|cpu, memory, co| async move {
        // RES 0, H
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x85] = &|cpu, memory, co| async move {
        // RES 0, L
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x86] = &|cpu, memory, co| async move {
        // RES 0, (HL)
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x87] = &|cpu, memory, co| async move {
        // RES 0, A
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 0, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x88] = &|cpu, memory, co| async move {
        // RES 1, B
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x89] = &|cpu, memory, co| async move {
        // RES 1, C
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x8A] = &|cpu, memory, co| async move {
        // RES 1, D
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x8B] = &|cpu, memory, co| async move {
        // RES 1, E
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x8C] = &|cpu, memory, co| async move {
        // RES 1, H
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x8D] = &|cpu, memory, co| async move {
        // RES 1, L
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x8E] = &|cpu, memory, co| async move {
        // RES 1, (HL)
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x8F] = &|cpu, memory, co| async move {
        // RES 1, A
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 1, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0x90] = &|cpu, memory, co| async move {
        // RES 2, B
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x91] = &|cpu, memory, co| async move {
        // RES 2, C
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x92] = &|cpu, memory, co| async move {
        // RES 2, D
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x93] = &|cpu, memory, co| async move {
        // RES 2, E
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x94] = &|cpu, memory, co| async move {
        // RES 2, H
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x95] = &|cpu, memory, co| async move {
        // RES 2, L
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x96] = &|cpu, memory, co| async move {
        // RES 2, (HL)
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x97] = &|cpu, memory, co| async move {
        // RES 2, A
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 2, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0x98] = &|cpu, memory, co| async move {
        // RES 3, B
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0x99] = &|cpu, memory, co| async move {
        // RES 3, C
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0x9A] = &|cpu, memory, co| async move {
        // RES 3, D
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0x9B] = &|cpu, memory, co| async move {
        // RES 3, E
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0x9C] = &|cpu, memory, co| async move {
        // RES 3, H
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0x9D] = &|cpu, memory, co| async move {
        // RES 3, L
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0x9E] = &|cpu, memory, co| async move {
        // RES 3, (HL)
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0x9F] = &|cpu, memory, co| async move {
        // RES 3, A
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 3, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0xA0] = &|cpu, memory, co| async move {
        // RES 4, B
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xA1] = &|cpu, memory, co| async move {
        // RES 4, C
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xA2] = &|cpu, memory, co| async move {
        // RES 4, D
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xA3] = &|cpu, memory, co| async move {
        // RES 4, E
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xA4] = &|cpu, memory, co| async move {
        // RES 4, H
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xA5] = &|cpu, memory, co| async move {
        // RES 4, L
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xA6] = &|cpu, memory, co| async move {
        // RES 4, (HL)
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xA7] = &|cpu, memory, co| async move {
        // RES 4, A
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 4, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0xA8] = &|cpu, memory, co| async move {
        // RES 5, B
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xA9] = &|cpu, memory, co| async move {
        // RES 5, C
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xAA] = &|cpu, memory, co| async move {
        // RES 5, D
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xAB] = &|cpu, memory, co| async move {
        // RES 5, E
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xAC] = &|cpu, memory, co| async move {
        // RES 5, H
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xAD] = &|cpu, memory, co| async move {
        // RES 5, L
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xAE] = &|cpu, memory, co| async move {
        // RES 5, (HL)
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xAF] = &|cpu, memory, co| async move {
        // RES 5, A
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 5, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0xB0] = &|cpu, memory, co| async move {
        // RES 6, B
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xB1] = &|cpu, memory, co| async move {
        // RES 6, C
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xB2] = &|cpu, memory, co| async move {
        // RES 6, D
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xB3] = &|cpu, memory, co| async move {
        // RES 6, E
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xB4] = &|cpu, memory, co| async move {
        // RES 6, H
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xB5] = &|cpu, memory, co| async move {
        // RES 6, L
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xB6] = &|cpu, memory, co| async move {
        // RES 6, (HL)
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xB7] = &|cpu, memory, co| async move {
        // RES 6, A
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 6, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0xB8] = &|cpu, memory, co| async move {
        // RES 7, B
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xB9] = &|cpu, memory, co| async move {
        // RES 7, C
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xBA] = &|cpu, memory, co| async move {
        // RES 7, D
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xBB] = &|cpu, memory, co| async move {
        // RES 7, E
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xBC] = &|cpu, memory, co| async move {
        // RES 7, H
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xBD] = &|cpu, memory, co| async move {
        // RES 7, L
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xBE] = &|cpu, memory, co| async move {
        // RES 7, (HL)
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xBF] = &|cpu, memory, co| async move {
        // RES 7, A
        gen_all!(co, |co_inner| cpu.res(memory, co_inner, 7, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0xC0] = &|cpu, memory, co| async move {
        // SET 0, B
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xC1] = &|cpu, memory, co| async move {
        // SET 0, C
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xC2] = &|cpu, memory, co| async move {
        // SET 0, D
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xC3] = &|cpu, memory, co| async move {
        // SET 0, E
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xC4] = &|cpu, memory, co| async move {
        // SET 0, H
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xC5] = &|cpu, memory, co| async move {
        // SET 0, L
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xC6] = &|cpu, memory, co| async move {
        // SET 0, (HL)
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xC7] = &|cpu, memory, co| async move {
        // SET 0, A
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 0, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0xC8] = &|cpu, memory, co| async move {
        // SET 1, B
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xC9] = &|cpu, memory, co| async move {
        // SET 1, C
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xCA] = &|cpu, memory, co| async move {
        // SET 1, D
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xCB] = &|cpu, memory, co| async move {
        // SET 1, E
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xCC] = &|cpu, memory, co| async move {
        // SET 1, H
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xCD] = &|cpu, memory, co| async move {
        // SET 1, L
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xCE] = &|cpu, memory, co| async move {
        // SET 1, (HL)
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xCF] = &|cpu, memory, co| async move {
        // SET 1, A
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 1, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0xD0] = &|cpu, memory, co| async move {
        // SET 2, B
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xD1] = &|cpu, memory, co| async move {
        // SET 2, C
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xD2] = &|cpu, memory, co| async move {
        // SET 2, D
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xD3] = &|cpu, memory, co| async move {
        // SET 2, E
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xD4] = &|cpu, memory, co| async move {
        // SET 2, H
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xD5] = &|cpu, memory, co| async move {
        // SET 2, L
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xD6] = &|cpu, memory, co| async move {
        // SET 2, (HL)
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xD7] = &|cpu, memory, co| async move {
        // SET 2, A
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 2, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0xD8] = &|cpu, memory, co| async move {
        // SET 3, B
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xD9] = &|cpu, memory, co| async move {
        // SET 3, C
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xDA] = &|cpu, memory, co| async move {
        // SET 3, D
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xDB] = &|cpu, memory, co| async move {
        // SET 3, E
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xDC] = &|cpu, memory, co| async move {
        // SET 3, H
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xDD] = &|cpu, memory, co| async move {
        // SET 3, L
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xDE] = &|cpu, memory, co| async move {
        // SET 3, (HL)
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xDF] = &|cpu, memory, co| async move {
        // SET 3, A
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 3, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0xE0] = &|cpu, memory, co| async move {
        // SET 4, B
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xE1] = &|cpu, memory, co| async move {
        // SET 4, C
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xE2] = &|cpu, memory, co| async move {
        // SET 4, D
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xE3] = &|cpu, memory, co| async move {
        // SET 4, E
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xE4] = &|cpu, memory, co| async move {
        // SET 4, H
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xE5] = &|cpu, memory, co| async move {
        // SET 4, L
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xE6] = &|cpu, memory, co| async move {
        // SET 4, (HL)
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xE7] = &|cpu, memory, co| async move {
        // SET 4, A
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 4, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0xE8] = &|cpu, memory, co| async move {
        // SET 5, B
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xE9] = &|cpu, memory, co| async move {
        // SET 5, C
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xEA] = &|cpu, memory, co| async move {
        // SET 5, D
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xEB] = &|cpu, memory, co| async move {
        // SET 5, E
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xEC] = &|cpu, memory, co| async move {
        // SET 5, H
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xED] = &|cpu, memory, co| async move {
        // SET 5, L
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xEE] = &|cpu, memory, co| async move {
        // SET 5, (HL)
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xEF] = &|cpu, memory, co| async move {
        // SET 5, A
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 5, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();

    op_table[0xF0] = &|cpu, memory, co| async move {
        // SET 6, B
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xF1] = &|cpu, memory, co| async move {
        // SET 6, C
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xF2] = &|cpu, memory, co| async move {
        // SET 6, D
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xF3] = &|cpu, memory, co| async move {
        // SET 6, E
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xF4] = &|cpu, memory, co| async move {
        // SET 6, H
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xF5] = &|cpu, memory, co| async move {
        // SET 6, L
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xF6] = &|cpu, memory, co| async move {
        // SET 6, (HL)
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xF7] = &|cpu, memory, co| async move {
        // SET 6, A
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 6, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    op_table[0xF8] = &|cpu, memory, co| async move {
        // SET 7, B
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::B)));
    }.boxed_local();
    op_table[0xF9] = &|cpu, memory, co| async move {
        // SET 7, C
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::C)));
    }.boxed_local();
    op_table[0xFA] = &|cpu, memory, co| async move {
        // SET 7, D
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::D)));
    }.boxed_local();
    op_table[0xFB] = &|cpu, memory, co| async move {
        // SET 7, E
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::E)));
    }.boxed_local();
    op_table[0xFC] = &|cpu, memory, co| async move {
        // SET 7, H
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::H)));
    }.boxed_local();
    op_table[0xFD] = &|cpu, memory, co| async move {
        // SET 7, L
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::L)));
    }.boxed_local();
    op_table[0xFE] = &|cpu, memory, co| async move {
        // SET 7, (HL)
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL))));
    }.boxed_local();
    op_table[0xFF] = &|cpu, memory, co| async move {
        // SET 7, A
        gen_all!(co, |co_inner| cpu.set(memory, co_inner, 7, RegisterOperand8(cpu::Register8::A)));
    }.boxed_local();
    
    op_table
}