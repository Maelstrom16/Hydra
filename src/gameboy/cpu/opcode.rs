use std::sync::MutexGuard;

use crate::gameboy::{
    cpu::{self, CPU},
    memory::Memory,
};

pub trait IntOperand<T> {
    fn get(&self, cpu: &mut CPU, memory: &mut Memory) -> T;
    fn set(&self, value: T, cpu: &mut CPU, memory: &mut Memory);
}

pub struct RegisterOperand8(pub cpu::Register8);
impl IntOperand<u8> for RegisterOperand8 {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU, _: &mut Memory) -> u8 {
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
    fn set(&self, value: u8, cpu: &mut CPU, _: &mut Memory) {
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
    fn get(&self, cpu: &mut CPU, memory: &mut Memory) -> u8 {
        cpu.step_u8_and_wait(memory)
    }
    #[inline(always)]
    fn set(&self, _: u8, _: &mut CPU, _: &mut Memory) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct ImmediateSignedOperand8;
impl IntOperand<i8> for ImmediateSignedOperand8 {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU, memory: &mut Memory) -> i8 {
        cpu.step_u8_and_wait(memory) as i8
    }
    #[inline(always)]
    fn set(&self, _: i8, _: &mut CPU, _: &mut Memory) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct IndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for IndirectOperand8<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU, memory: &mut Memory) -> u8 {
        let address = self.0.get(cpu, memory);
        cpu.read_u8_and_wait(memory, address)
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut CPU, memory: &mut Memory) {
        let address = self.0.get(cpu, memory);
        cpu.write_u8_and_wait(memory, address, value);
    }
}
pub struct IncIndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for IncIndirectOperand8<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU, memory: &mut Memory) -> u8 {
        let address = self.0.get(cpu, memory);
        self.0.set(address + 1, cpu, memory);
        cpu.read_u8_and_wait(memory, address)
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut CPU, memory: &mut Memory) {
        let address = self.0.get(cpu, memory);
        self.0.set(address + 1, cpu, memory);
        cpu.write_u8_and_wait(memory, address, value);
    }
}
pub struct DecIndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for DecIndirectOperand8<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU, memory: &mut Memory) -> u8 {
        let address = self.0.get(cpu, memory);
        self.0.set(address - 1, cpu, memory);
        cpu.read_u8_and_wait(memory, address)
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut CPU, memory: &mut Memory) {
        let address = self.0.get(cpu, memory);
        self.0.set(address - 1, cpu, memory);
        cpu.write_u8_and_wait(memory, address, value);
    }
}

pub struct HramIndirectOperand<O: IntOperand<u8>>(pub O);
impl<O: IntOperand<u8>> HramIndirectOperand<O> {
    #[inline(always)]
    fn as_hram_address(&self, cpu: &mut CPU, memory: &mut Memory) -> u16 {
        0xFF00 | (self.0.get(cpu, memory) as u16)
    }
}
impl<O: IntOperand<u8>> IntOperand<u8> for HramIndirectOperand<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU, memory: &mut Memory) -> u8 {
        let hram_address = self.as_hram_address(cpu, memory);
        cpu.read_u8_and_wait(memory, hram_address)
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut CPU, memory: &mut Memory) {
        let hram_address = self.as_hram_address(cpu, memory);
        cpu.write_u8_and_wait(memory, hram_address, value);
    }
}
impl<O: IntOperand<u16>> IntOperand<u16> for IndirectOperand8<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU, memory: &mut Memory) -> u16 {
        let address = self.0.get(cpu, memory);
        u16::from_le_bytes([cpu.read_u8_and_wait(memory, address), cpu.read_u8_and_wait(memory, address + 1)])
    }
    #[inline(always)]
    fn set(&self, value: u16, cpu: &mut CPU, memory: &mut Memory) {
        let address = self.0.get(cpu, memory);
        let bytes = u16::to_le_bytes(value);
        cpu.write_u8_and_wait(memory, address, bytes[0]);
        cpu.write_u8_and_wait(memory, address + 1, bytes[1]);
    }
}

pub struct RegisterOperand16(pub cpu::Register16);
impl IntOperand<u16> for RegisterOperand16 {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU, _: &mut Memory) -> u16 {
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
    fn set(&self, value: u16, cpu: &mut CPU, _: &mut Memory) {
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
    fn get(&self, cpu: &mut CPU, memory: &mut Memory) -> u16 {
        u16::from_le_bytes([cpu.step_u8_and_wait(memory), cpu.step_u8_and_wait(memory)])
    }
    #[inline(always)]
    fn set(&self, _: u16, _: &mut CPU, _: &mut Memory) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct ConstOperand16(u16);
impl IntOperand<u16> for ConstOperand16 {
    #[inline(always)]
    fn get(&self, _: &mut CPU, _: &mut Memory) -> u16 {
        self.0
    }
    #[inline(always)]
    fn set(&self, _: u16, _: &mut CPU, _: &mut Memory) {
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


// Table definitions
type OpHandler = fn(&mut CPU, memory: &mut Memory);
static OP_UNINIT: OpHandler = |_, _| panic!("Unknown opcode");

pub static OPCODE_FUNCTIONS: [OpHandler; 0x100] = init_opcode_table();
static CB_FUNCTIONS: [OpHandler; 0x100] = init_cb_table();

const fn init_opcode_table() -> [OpHandler; 0x100] {
    // Table initialization
    let mut opcode_table: [OpHandler; 0x100] = [OP_UNINIT; 0x100];
    opcode_table[0x00] = |_, _| { // NOP
        // Do nothing
    };
    opcode_table[0x01] = |cpu, memory| { // LD BC, nn
        cpu.ld(memory, RegisterOperand16(cpu::Register16::BC), ImmediateOperand16);
    };
    opcode_table[0x02] = |cpu, memory| { // LD (BC), A
        cpu.ld(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::BC)), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x03] = |cpu, memory| { // INC BC
        cpu.inc16(memory, RegisterOperand16(cpu::Register16::BC));
    };
    opcode_table[0x04] = |cpu, memory| { // INC B
        cpu.inc(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x05] = |cpu, memory| { // DEC B
        cpu.dec(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x06] = |cpu, memory| { // LD B, n
        cpu.ld(memory, RegisterOperand8(cpu::Register8::B), ImmediateOperand8);
    };
    opcode_table[0x07] = |cpu, memory| { // RLCA
        cpu.rlc(memory, RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x08] = |cpu, memory| { // LD (nn), SP
        cpu.ld(memory, IndirectOperand8(ImmediateOperand16), RegisterOperand16(cpu::Register16::SP));
    };
    opcode_table[0x09] = |cpu, memory| { // ADD HL, BC
        cpu.add_hl(memory, RegisterOperand16(cpu::Register16::BC));
    };
    opcode_table[0x0A] = |cpu, memory| { // LD A, (BC)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), IndirectOperand8(RegisterOperand16(cpu::Register16::BC)));
    };
    opcode_table[0x0B] = |cpu, memory| { // DEC BC
        cpu.dec16(memory, RegisterOperand16(cpu::Register16::BC));
    };
    opcode_table[0x0C] = |cpu, memory| { // INC C
        cpu.inc(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x0D] = |cpu, memory| { // DEC C
        cpu.dec(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x0E] = |cpu, memory| { // LD C, n
        cpu.ld(memory, RegisterOperand8(cpu::Register8::C), ImmediateOperand8)
    };
    opcode_table[0x0F] = |cpu, memory| { // RRCA
        cpu.rrc(memory, RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x10] = |cpu, memory| { // STOP
        cpu.stop(memory, );
    };
    opcode_table[0x11] = |cpu, memory| { // LD DE, nn
        cpu.ld(memory, RegisterOperand16(cpu::Register16::DE), ImmediateOperand16);
    };
    opcode_table[0x12] = |cpu, memory| { // LD (DE), A
        cpu.ld(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::DE)), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x13] = |cpu, memory| { // INC DE
        cpu.inc16(memory, RegisterOperand16(cpu::Register16::DE));
    };
    opcode_table[0x14] = |cpu, memory| { // INC D
        cpu.inc(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x15] = |cpu, memory| { // DEC D
        cpu.dec(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x16] = |cpu, memory| { // LD D, n
        cpu.ld(memory, RegisterOperand8(cpu::Register8::D), ImmediateOperand8);
    };
    opcode_table[0x17] = |cpu, memory| { // RLA
        cpu.rl(memory, RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x18] = |cpu, memory| { // JR e
        cpu.jr(memory, CondOperand::Unconditional, ImmediateSignedOperand8);
    };
    opcode_table[0x19] = |cpu, memory| { // ADD HL, DE
        cpu.add_hl(memory, RegisterOperand16(cpu::Register16::DE));
    };
    opcode_table[0x1A] = |cpu, memory| { // LD A, (DE)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), IndirectOperand8(RegisterOperand16(cpu::Register16::DE)));
    };
    opcode_table[0x1B] = |cpu, memory| { // DEC DE
        cpu.dec16(memory, RegisterOperand16(cpu::Register16::DE));
    };
    opcode_table[0x1C] = |cpu, memory| { // INC E
        cpu.inc(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x1D] = |cpu, memory| { // DEC E
        cpu.dec(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x1E] = |cpu, memory| { // LD E, n
        cpu.ld(memory, RegisterOperand8(cpu::Register8::E), ImmediateOperand8);
    };
    opcode_table[0x1F] = |cpu, memory| { // RRA
        cpu.rr(memory, RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x20] = |cpu, memory| { // JR NZ, e
        cpu.jr(memory, CondOperand::NZ, ImmediateSignedOperand8);
    };
    opcode_table[0x21] = |cpu, memory| { // LD HL, nn
        cpu.ld(memory, RegisterOperand16(cpu::Register16::HL), ImmediateOperand16);
    };
    opcode_table[0x22] = |cpu, memory| { // LD (HL+), A
        cpu.ld(memory, IncIndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x23] = |cpu, memory| { // INC HL
        cpu.inc16(memory, RegisterOperand16(cpu::Register16::HL));
    };
    opcode_table[0x24] = |cpu, memory| { // INC H
        cpu.inc(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x25] = |cpu, memory| { // DEC H
        cpu.dec(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x26] = |cpu, memory| { // LD H, n
        cpu.ld(memory, RegisterOperand8(cpu::Register8::H), ImmediateOperand8);
    };
    opcode_table[0x27] = |cpu, memory| { // DAA
        cpu.daa(memory);
    };
    opcode_table[0x28] = |cpu, memory| { // JR Z, e
        cpu.jr(memory, CondOperand::Z, ImmediateSignedOperand8);
    };
    opcode_table[0x29] = |cpu, memory| { // ADD HL, HL
        cpu.add_hl(memory, RegisterOperand16(cpu::Register16::HL));
    };
    opcode_table[0x2A] = |cpu, memory| { // LD A, (HL+)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), IncIndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x2B] = |cpu, memory| { // DEC HL
        cpu.dec16(memory, RegisterOperand16(cpu::Register16::HL));
    };
    opcode_table[0x2C] = |cpu, memory| { // INC L
        cpu.inc(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x2D] = |cpu, memory| { // DEC L
        cpu.dec(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x2E] = |cpu, memory| { // LD L, n
        cpu.ld(memory, RegisterOperand8(cpu::Register8::L), ImmediateOperand8);
    };
    opcode_table[0x2F] = |cpu, memory| { // CPL
        cpu.cpl(memory, );
    };

    opcode_table[0x30] = |cpu, memory| { // JR NC, e
        cpu.jr(memory, CondOperand::NC, ImmediateSignedOperand8);
    };
    opcode_table[0x31] = |cpu, memory| { // LD SP, nn
        cpu.ld(memory, RegisterOperand16(cpu::Register16::SP), ImmediateOperand16);
    };
    opcode_table[0x32] = |cpu, memory| { // LD (HL-), A
        cpu.ld(memory, DecIndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x33] = |cpu, memory| { // INC SP
        cpu.inc16(memory, RegisterOperand16(cpu::Register16::SP));
    };
    opcode_table[0x34] = |cpu, memory| { // INC (HL)
        cpu.inc(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x35] = |cpu, memory| { // DEC (HL)
        cpu.dec(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x36] = |cpu, memory| { // LD (HL), n
        cpu.ld(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), ImmediateOperand8);
    };
    opcode_table[0x37] = |cpu, memory| { // SCF
        cpu.scf(memory, );
    };
    opcode_table[0x38] = |cpu, memory| { // JR C, e
        cpu.jr(memory, CondOperand::C, ImmediateSignedOperand8);
    };
    opcode_table[0x39] = |cpu, memory| { // ADD HL, SP
        cpu.add_hl(memory, RegisterOperand16(cpu::Register16::SP));
    };
    opcode_table[0x3A] = |cpu, memory| { // LD A, (HL-)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), DecIndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x3B] = |cpu, memory| { // DEC SP
        cpu.dec16(memory, RegisterOperand16(cpu::Register16::SP));
    };
    opcode_table[0x3C] = |cpu, memory| { // INC A
        cpu.inc(memory, RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x3D] = |cpu, memory| { // DEC A
        cpu.dec(memory, RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x3E] = |cpu, memory| { // LD A, n
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), ImmediateOperand8);
    };
    opcode_table[0x3F] = |cpu, memory| { // CCF
        cpu.ccf(memory, );
    };

    opcode_table[0x40] = |cpu, memory| { // LD B, B
        cpu.ld(memory, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x41] = |cpu, memory| { // LD B, C
        cpu.ld(memory, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x42] = |cpu, memory| { // LD B, D
        cpu.ld(memory, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x43] = |cpu, memory| { // LD B, E
        cpu.ld(memory, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x44] = |cpu, memory| { // LD B, H
        cpu.ld(memory, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x45] = |cpu, memory| { // LD B, L
        cpu.ld(memory, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x46] = |cpu, memory| { // LD B, (HL)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::B), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x47] = |cpu, memory| { // LD B, A
        cpu.ld(memory, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x48] = |cpu, memory| { // LD C, B
        cpu.ld(memory, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x49] = |cpu, memory| { // LD C, C
        cpu.ld(memory, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x4A] = |cpu, memory| { // LD C, D
        cpu.ld(memory, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x4B] = |cpu, memory| { // LD C, E
        cpu.ld(memory, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x4C] = |cpu, memory| { // LD C, H
        cpu.ld(memory, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x4D] = |cpu, memory| { // LD C, L
        cpu.ld(memory, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x4E] = |cpu, memory| { // LD C, (HL)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::C), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x4F] = |cpu, memory| { // LD C, A
        cpu.ld(memory, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x50] = |cpu, memory| { // LD D, B
        cpu.ld(memory, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x51] = |cpu, memory| { // LD D, C
        cpu.ld(memory, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x52] = |cpu, memory| { // LD D, D
        cpu.ld(memory, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x53] = |cpu, memory| { // LD D, E
        cpu.ld(memory, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x54] = |cpu, memory| { // LD D, H
        cpu.ld(memory, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x55] = |cpu, memory| { // LD D, L
        cpu.ld(memory, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x56] = |cpu, memory| { // LD D, (HL)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::D), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x57] = |cpu, memory| { // LD D, A
        cpu.ld(memory, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x58] = |cpu, memory| { // LD E, B
        cpu.ld(memory, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x59] = |cpu, memory| { // LD E, C
        cpu.ld(memory, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x5A] = |cpu, memory| { // LD E, D
        cpu.ld(memory, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x5B] = |cpu, memory| { // LD E, E
        cpu.ld(memory, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x5C] = |cpu, memory| { // LD E, H
        cpu.ld(memory, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x5D] = |cpu, memory| { // LD E, L
        cpu.ld(memory, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x5E] = |cpu, memory| { // LD E, (HL)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::E), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x5F] = |cpu, memory| { // LD E, A
        cpu.ld(memory, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x60] = |cpu, memory| { // LD H, B
        cpu.ld(memory, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x61] = |cpu, memory| { // LD H, C
        cpu.ld(memory, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x62] = |cpu, memory| { // LD H, D
        cpu.ld(memory, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x63] = |cpu, memory| { // LD H, E
        cpu.ld(memory, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x64] = |cpu, memory| { // LD H, H
        cpu.ld(memory, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x65] = |cpu, memory| { // LD H, L
        cpu.ld(memory, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x66] = |cpu, memory| { // LD H, (HL)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::H), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x67] = |cpu, memory| { // LD H, A
        cpu.ld(memory, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x68] = |cpu, memory| { // LD L, B
        cpu.ld(memory, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x69] = |cpu, memory| { // LD L, C
        cpu.ld(memory, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x6A] = |cpu, memory| { // LD L, D
        cpu.ld(memory, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x6B] = |cpu, memory| { // LD L, E
        cpu.ld(memory, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x6C] = |cpu, memory| { // LD L, H
        cpu.ld(memory, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x6D] = |cpu, memory| { // LD L, L
        cpu.ld(memory, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x6E] = |cpu, memory| { // LD L, (HL)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::L), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x6F] = |cpu, memory| { // LD L, A
        cpu.ld(memory, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x70] = |cpu, memory| { // LD (HL), B
        cpu.ld(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x71] = |cpu, memory| { // LD (HL), C
        cpu.ld(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x72] = |cpu, memory| { // LD (HL), D
        cpu.ld(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x73] = |cpu, memory| { // LD (HL), E
        cpu.ld(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x74] = |cpu, memory| { // LD (HL), H
        cpu.ld(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x75] = |cpu, memory| { // LD (HL), L
        cpu.ld(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x76] = |cpu, memory| { // HALT
        cpu.halt(memory, );
    };
    opcode_table[0x77] = |cpu, memory| { // LD (HL), A
        cpu.ld(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x78] = |cpu, memory| { // LD A, B
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x79] = |cpu, memory| { // LD A, C
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x7A] = |cpu, memory| { // LD A, D
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x7B] = |cpu, memory| { // LD A, E
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x7C] = |cpu, memory| { // LD A, H
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x7D] = |cpu, memory| { // LD A, L
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x7E] = |cpu, memory| { // LD A, (HL)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x7F] = |cpu, memory| { // LD A, A
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x80] = |cpu, memory| { // ADD B
        cpu.add(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x81] = |cpu, memory| { // ADD C
        cpu.add(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x82] = |cpu, memory| { // ADD D
        cpu.add(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x83] = |cpu, memory| { // ADD E
        cpu.add(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x84] = |cpu, memory| { // ADD H
        cpu.add(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x85] = |cpu, memory| { // ADD L
        cpu.add(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x86] = |cpu, memory| { // ADD (HL)
        cpu.add(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x87] = |cpu, memory| { // ADD A
        cpu.add(memory, RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x88] = |cpu, memory| { // ADC B
        cpu.adc(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x89] = |cpu, memory| { // ADC C
        cpu.adc(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x8A] = |cpu, memory| { // ADC D
        cpu.adc(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x8B] = |cpu, memory| { // ADC E
        cpu.adc(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x8C] = |cpu, memory| { // ADC H
        cpu.adc(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x8D] = |cpu, memory| { // ADC L
        cpu.adc(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x8E] = |cpu, memory| { // ADC (HL)
        cpu.adc(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x8F] = |cpu, memory| { // ADC A
        cpu.adc(memory, RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x90] = |cpu, memory| { // SUB B
        cpu.sub(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x91] = |cpu, memory| { // SUB C
        cpu.sub(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x92] = |cpu, memory| { // SUB D
        cpu.sub(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x93] = |cpu, memory| { // SUB E
        cpu.sub(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x94] = |cpu, memory| { // SUB H
        cpu.sub(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x95] = |cpu, memory| { // SUB L
        cpu.sub(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x96] = |cpu, memory| { // SUB (HL)
        cpu.sub(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x97] = |cpu, memory| { // SUB A
        cpu.sub(memory, RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x98] = |cpu, memory| { // SBC B
        cpu.sbc(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x99] = |cpu, memory| { // SBC C
        cpu.sbc(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x9A] = |cpu, memory| { // SBC D
        cpu.sbc(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x9B] = |cpu, memory| { // SBC E
        cpu.sbc(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x9C] = |cpu, memory| { // SBC H
        cpu.sbc(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x9D] = |cpu, memory| { // SBC L
        cpu.sbc(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x9E] = |cpu, memory| { // SBC (HL)
        cpu.sbc(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x9F] = |cpu, memory| { // SBC A
        cpu.sbc(memory, RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0xA0] = |cpu, memory| { // AND B
        cpu.and(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0xA1] = |cpu, memory| { // AND C
        cpu.and(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0xA2] = |cpu, memory| { // AND D
        cpu.and(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0xA3] = |cpu, memory| { // AND E
        cpu.and(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0xA4] = |cpu, memory| { // AND H
        cpu.and(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0xA5] = |cpu, memory| { // AND L
        cpu.and(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0xA6] = |cpu, memory| { // AND (HL)
        cpu.and(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xA7] = |cpu, memory| { // AND A
        cpu.and(memory, RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0xA8] = |cpu, memory| { // XOR B
        cpu.xor(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0xA9] = |cpu, memory| { // XOR C
        cpu.xor(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0xAA] = |cpu, memory| { // XOR D
        cpu.xor(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0xAB] = |cpu, memory| { // XOR E
        cpu.xor(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0xAC] = |cpu, memory| { // XOR H
        cpu.xor(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0xAD] = |cpu, memory| { // XOR L
        cpu.xor(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0xAE] = |cpu, memory| { // XOR (HL)
        cpu.xor(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xAF] = |cpu, memory| { // XOR A
        cpu.xor(memory, RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0xB0] = |cpu, memory| { // OR B
        cpu.or(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0xB1] = |cpu, memory| { // OR C
        cpu.or(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0xB2] = |cpu, memory| { // OR D
        cpu.or(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0xB3] = |cpu, memory| { // OR E
        cpu.or(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0xB4] = |cpu, memory| { // OR H
        cpu.or(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0xB5] = |cpu, memory| { // OR L
        cpu.or(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0xB6] = |cpu, memory| { // OR (HL)
        cpu.or(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xB7] = |cpu, memory| { // OR A
        cpu.or(memory, RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0xB8] = |cpu, memory| { // CP B
        cpu.cp(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0xB9] = |cpu, memory| { // CP C
        cpu.cp(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0xBA] = |cpu, memory| { // CP D
        cpu.cp(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0xBB] = |cpu, memory| { // CP E
        cpu.cp(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0xBC] = |cpu, memory| { // CP H
        cpu.cp(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0xBD] = |cpu, memory| { // CP L
        cpu.cp(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0xBE] = |cpu, memory| { // CP (HL)
        cpu.cp(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xBF] = |cpu, memory| { // CP A
        cpu.cp(memory, RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0xC0] = |cpu, memory| { // RET NZ
        cpu.ret(memory, CondOperand::NZ);
    };
    opcode_table[0xC1] = |cpu, memory| { // POP BC
        cpu.pop(memory, RegisterOperand16(cpu::Register16::BC));
    };
    opcode_table[0xC2] = |cpu, memory| { // JP NZ, nn
        cpu.jp(memory, CondOperand::NZ, ImmediateOperand16);
    };
    opcode_table[0xC3] = |cpu, memory| { // JP nn
        cpu.jp(memory, CondOperand::Unconditional, ImmediateOperand16);
    };
    opcode_table[0xC4] = |cpu, memory| { // CALL NZ, nn
        cpu.call(memory, CondOperand::NZ, ImmediateOperand16);
    };
    opcode_table[0xC5] = |cpu, memory| { // PUSH BC
        cpu.push(memory, RegisterOperand16(cpu::Register16::BC));
    };
    opcode_table[0xC6] = |cpu, memory| { // ADD n
        cpu.add(memory, ImmediateOperand8);
    };
    opcode_table[0xC7] = |cpu, memory| { // RST 0x00
        cpu.call(memory, CondOperand::Unconditional, ConstOperand16(0x0000));
    };
    opcode_table[0xC8] = |cpu, memory| { // RET Z
        cpu.ret(memory, CondOperand::Z);
    };
    opcode_table[0xC9] = |cpu, memory| { // RET
        cpu.ret(memory, CondOperand::Unconditional);
    };
    opcode_table[0xCA] = |cpu, memory| { // JP Z, nn
        cpu.jp(memory, CondOperand::Z, ImmediateOperand16);
    };
    opcode_table[0xCB] = |cpu, memory| { // CB op
        cpu.ir = cpu.step_u8_and_wait(memory);
        CB_FUNCTIONS[cpu.ir as usize](cpu, memory);
    };
    opcode_table[0xCC] = |cpu, memory| { // CALL Z, nn
        cpu.call(memory, CondOperand::Z, ImmediateOperand16);
    };
    opcode_table[0xCD] = |cpu, memory| { // CALL nn
        cpu.call(memory, CondOperand::Unconditional, ImmediateOperand16);
    };
    opcode_table[0xCE] = |cpu, memory| { // ADC n
        cpu.adc(memory, ImmediateOperand8);
    };
    opcode_table[0xCF] = |cpu, memory| { // RST 0x08
        cpu.call(memory, CondOperand::Unconditional, ConstOperand16(0x0008));
    };

    opcode_table[0xD0] = |cpu, memory| { // RET NC
        cpu.ret(memory, CondOperand::NC);
    };
    opcode_table[0xD1] = |cpu, memory| { // POP DE
        cpu.pop(memory, RegisterOperand16(cpu::Register16::DE));
    };
    opcode_table[0xD2] = |cpu, memory| { // JP NC, nn
        cpu.jp(memory, CondOperand::NC, ImmediateOperand16);
    };
    // opcode_table[0xD3] = (invalid)
    opcode_table[0xD4] = |cpu, memory| { // CALL NC, nn
        cpu.call(memory, CondOperand::NC, ImmediateOperand16);
    };
    opcode_table[0xD5] = |cpu, memory| { // PUSH DE
        cpu.push(memory, RegisterOperand16(cpu::Register16::DE));
    };
    opcode_table[0xD6] = |cpu, memory| { // SUB n
        cpu.sub(memory, ImmediateOperand8);
    };
    opcode_table[0xD7] = |cpu, memory| { // RST 0x10
        cpu.call(memory, CondOperand::Unconditional, ConstOperand16(0x0010));
    };
    opcode_table[0xD8] = |cpu, memory| { // RET C
        cpu.ret(memory, CondOperand::C);
    };
    opcode_table[0xD9] = |cpu, memory| { // RETI
        cpu.reti(memory, );
    };
    opcode_table[0xDA] = |cpu, memory| { // JP C, nn
        cpu.jp(memory, CondOperand::C, ImmediateOperand16);
    };
    // opcode_table[0xDB] = (invalid)
    opcode_table[0xDC] = |cpu, memory| { // CALL C, nn
        cpu.call(memory, CondOperand::C, ImmediateOperand16);
    };
    // opcode_table[0xDD] = (invalid)
    opcode_table[0xDE] = |cpu, memory| { // SBC n
        cpu.sbc(memory, ImmediateOperand8);
    };
    opcode_table[0xDF] = |cpu, memory| { // RST 0x18
        cpu.call(memory, CondOperand::Unconditional, ConstOperand16(0x0018));
    };

    opcode_table[0xE0] = |cpu, memory| { // LDH (n), A
        cpu.ld(memory, HramIndirectOperand(ImmediateOperand8), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0xE1] = |cpu, memory| { // POP HL
        cpu.pop(memory, RegisterOperand16(cpu::Register16::HL));
    };
    opcode_table[0xE2] = |cpu, memory| { // LDH (C), A
        cpu.ld(memory, HramIndirectOperand(RegisterOperand8(cpu::Register8::C)), RegisterOperand8(cpu::Register8::A));
    };
    // opcode_table[0xE3] = (invalid)
    // opcode_table[0xE4] = (invalid)
    opcode_table[0xE5] = |cpu, memory| { // PUSH HL
        cpu.push(memory, RegisterOperand16(cpu::Register16::HL));
    };
    opcode_table[0xE6] = |cpu, memory| { // AND n
        cpu.and(memory, ImmediateOperand8);
    };
    opcode_table[0xE7] = |cpu, memory| { // RST 0x20
        cpu.call(memory, CondOperand::Unconditional, ConstOperand16(0x0020));
    };
    opcode_table[0xE8] = |cpu, memory| { // ADD SP, e
        cpu.add_spe(memory, );
    };
    opcode_table[0xE9] = |cpu, memory| { // JP HL
        cpu.ld(memory, RegisterOperand16(cpu::Register16::PC), RegisterOperand16(cpu::Register16::HL));
    };
    opcode_table[0xEA] = |cpu, memory| { // LD (nn), A
        cpu.ld(memory, IndirectOperand8(ImmediateOperand16), RegisterOperand8(cpu::Register8::A));
    };
    // opcode_table[0xEB] = (invalid)
    // opcode_table[0xEC] = (invalid)
    // opcode_table[0xED] = (invalid)
    opcode_table[0xEE] = |cpu, memory| { // XOR n
        cpu.xor(memory, ImmediateOperand8);
    };
    opcode_table[0xEF] = |cpu, memory| { // RST 0x28
        cpu.call(memory, CondOperand::Unconditional, ConstOperand16(0x0028));
    };

    opcode_table[0xF0] = |cpu, memory| { // LDH A, (n)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), HramIndirectOperand(ImmediateOperand8));
    };
    opcode_table[0xF1] = |cpu, memory| { // POP AF
        cpu.pop(memory, RegisterOperand16(cpu::Register16::AF));
    };
    opcode_table[0xF2] = |cpu, memory| { // LDH A, (C)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), HramIndirectOperand(RegisterOperand8(cpu::Register8::C)));
    };
    opcode_table[0xF3] = |cpu, memory| { // DI
        cpu.di(memory, );
    };
    // opcode_table[0xF4] = (invalid)
    opcode_table[0xF5] = |cpu, memory| { // PUSH AF
        cpu.push(memory, RegisterOperand16(cpu::Register16::AF));
    };
    opcode_table[0xF6] = |cpu, memory| { // OR n
        cpu.or(memory, ImmediateOperand8);
    };
    opcode_table[0xF7] = |cpu, memory| { // RST 0x30
        cpu.call(memory, CondOperand::Unconditional, ConstOperand16(0x0030));
    };
    opcode_table[0xF8] = |cpu, memory| { // LD HL, SP+e
        cpu.ld_hlspe(memory, );
    };
    opcode_table[0xF9] = |cpu, memory| { // LD SP, HL
        cpu.ld(memory, RegisterOperand16(cpu::Register16::SP), RegisterOperand16(cpu::Register16::HL));
        
    };
    opcode_table[0xFA] = |cpu, memory| { // LD A, (nn)
        cpu.ld(memory, RegisterOperand8(cpu::Register8::A), IndirectOperand8(ImmediateOperand16));
    };
    opcode_table[0xFB] = |cpu, memory| { // EI
        cpu.ei(memory, );
    };
    // opcode_table[0xFC] = (invalid)
    // opcode_table[0xFD] = (invalid)
    opcode_table[0xFE] = |cpu, memory| { // CP n
        cpu.cp(memory, ImmediateOperand8);
    };
    opcode_table[0xFF] = |cpu, memory| { // RST 0x38
        cpu.call(memory, CondOperand::Unconditional, ConstOperand16(0x0038));
    };

    opcode_table
}

const fn init_cb_table() -> [OpHandler; 0x100] {
    let mut opcode_table: [OpHandler; 0x100] = [OP_UNINIT; 0x100];
    opcode_table[0x00] = |cpu, memory| { // RLC B
        cpu.rlc(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x01] = |cpu, memory| { // RLC C
        cpu.rlc(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x02] = |cpu, memory| { // RLC D
        cpu.rlc(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x03] = |cpu, memory| { // RLC E
        cpu.rlc(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x04] = |cpu, memory| { // RLC H
        cpu.rlc(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x05] = |cpu, memory| { // RLC L
        cpu.rlc(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x06] = |cpu, memory| { // RLC (HL)
        cpu.rlc(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x07] = |cpu, memory| { // RLC A
        cpu.rlc(memory, RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x08] = |cpu, memory| { // RRC B
        cpu.rrc(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x09] = |cpu, memory| { // RRC C
        cpu.rrc(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x0A] = |cpu, memory| { // RRC D
        cpu.rrc(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x0B] = |cpu, memory| { // RRC E
        cpu.rrc(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x0C] = |cpu, memory| { // RRC H
        cpu.rrc(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x0D] = |cpu, memory| { // RRC L
        cpu.rrc(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x0E] = |cpu, memory| { // RRC (HL)
        cpu.rrc(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x0F] = |cpu, memory| { // RRC A
        cpu.rrc(memory, RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x10] = |cpu, memory| { // RL B
        cpu.rl(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x11] = |cpu, memory| { // RL C
        cpu.rl(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x12] = |cpu, memory| { // RL D
        cpu.rl(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x13] = |cpu, memory| { // RL E
        cpu.rl(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x14] = |cpu, memory| { // RL H
        cpu.rl(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x15] = |cpu, memory| { // RL L
        cpu.rl(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x16] = |cpu, memory| { // RL (HL)
        cpu.rl(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x17] = |cpu, memory| { // RL A
        cpu.rl(memory, RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x18] = |cpu, memory| { // RR B
        cpu.rr(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x19] = |cpu, memory| { // RR C
        cpu.rr(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x1A] = |cpu, memory| { // RR D
        cpu.rr(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x1B] = |cpu, memory| { // RR E
        cpu.rr(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x1C] = |cpu, memory| { // RR H
        cpu.rr(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x1D] = |cpu, memory| { // RR L
        cpu.rr(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x1E] = |cpu, memory| { // RR (HL)
        cpu.rr(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x1F] = |cpu, memory| { // RR A
        cpu.rr(memory, RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x20] = |cpu, memory| { // SLA B
        cpu.sla(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x21] = |cpu, memory| { // SLA C
        cpu.sla(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x22] = |cpu, memory| { // SLA D
        cpu.sla(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x23] = |cpu, memory| { // SLA E
        cpu.sla(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x24] = |cpu, memory| { // SLA H
        cpu.sla(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x25] = |cpu, memory| { // SLA L
        cpu.sla(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x26] = |cpu, memory| { // SLA (HL)
        cpu.sla(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x27] = |cpu, memory| { // SLA A
        cpu.sla(memory, RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x28] = |cpu, memory| { // SRA B
        cpu.sra(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x29] = |cpu, memory| { // SRA C
        cpu.sra(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x2A] = |cpu, memory| { // SRA D
        cpu.sra(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x2B] = |cpu, memory| { // SRA E
        cpu.sra(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x2C] = |cpu, memory| { // SRA H
        cpu.sra(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x2D] = |cpu, memory| { // SRA L
        cpu.sra(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x2E] = |cpu, memory| { // SRA (HL)
        cpu.sra(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x2F] = |cpu, memory| { // SRA A
        cpu.sra(memory, RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x30] = |cpu, memory| { // SWAP B
        cpu.swap(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x31] = |cpu, memory| { // SWAP C
        cpu.swap(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x32] = |cpu, memory| { // SWAP D
        cpu.swap(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x33] = |cpu, memory| { // SWAP E
        cpu.swap(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x34] = |cpu, memory| { // SWAP H
        cpu.swap(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x35] = |cpu, memory| { // SWAP L
        cpu.swap(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x36] = |cpu, memory| { // SWAP (HL)
        cpu.swap(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x37] = |cpu, memory| { // SWAP A
        cpu.swap(memory, RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x38] = |cpu, memory| { // SRL B
        cpu.srl(memory, RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x39] = |cpu, memory| { // SRL C
        cpu.srl(memory, RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x3A] = |cpu, memory| { // SRL D
        cpu.srl(memory, RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x3B] = |cpu, memory| { // SRL E
        cpu.srl(memory, RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x3C] = |cpu, memory| { // SRL H
        cpu.srl(memory, RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x3D] = |cpu, memory| { // SRL L
        cpu.srl(memory, RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x3E] = |cpu, memory| { // SRL (HL)
        cpu.srl(memory, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x3F] = |cpu, memory| { // SRL A
        cpu.srl(memory, RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x40] = |cpu, memory| { // BIT 0, B
		cpu.bit(memory, 0, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x41] = |cpu, memory| { // BIT 0, C
		cpu.bit(memory, 0, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x42] = |cpu, memory| { // BIT 0, D
		cpu.bit(memory, 0, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x43] = |cpu, memory| { // BIT 0, E
		cpu.bit(memory, 0, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x44] = |cpu, memory| { // BIT 0, H
		cpu.bit(memory, 0, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x45] = |cpu, memory| { // BIT 0, L
		cpu.bit(memory, 0, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x46] = |cpu, memory| { // BIT 0, (HL)
        cpu.bit(memory, 0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x47] = |cpu, memory| { // BIT 0, A
		cpu.bit(memory, 0, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0x48] = |cpu, memory| { // BIT 1, B
		cpu.bit(memory, 1, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x49] = |cpu, memory| { // BIT 1, C
		cpu.bit(memory, 1, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x4A] = |cpu, memory| { // BIT 1, D
		cpu.bit(memory, 1, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x4B] = |cpu, memory| { // BIT 1, E
		cpu.bit(memory, 1, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x4C] = |cpu, memory| { // BIT 1, H
		cpu.bit(memory, 1, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x4D] = |cpu, memory| { // BIT 1, L
		cpu.bit(memory, 1, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x4E] = |cpu, memory| { // BIT 1, (HL)
        cpu.bit(memory, 1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x4F] = |cpu, memory| { // BIT 1, A
		cpu.bit(memory, 1, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0x50] = |cpu, memory| { // BIT 2, B
		cpu.bit(memory, 2, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x51] = |cpu, memory| { // BIT 2, C
		cpu.bit(memory, 2, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x52] = |cpu, memory| { // BIT 2, D
		cpu.bit(memory, 2, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x53] = |cpu, memory| { // BIT 2, E
		cpu.bit(memory, 2, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x54] = |cpu, memory| { // BIT 2, H
		cpu.bit(memory, 2, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x55] = |cpu, memory| { // BIT 2, L
		cpu.bit(memory, 2, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x56] = |cpu, memory| { // BIT 2, (HL)
        cpu.bit(memory, 2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x57] = |cpu, memory| { // BIT 2, A
		cpu.bit(memory, 2, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0x58] = |cpu, memory| { // BIT 3, B
		cpu.bit(memory, 3, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x59] = |cpu, memory| { // BIT 3, C
		cpu.bit(memory, 3, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x5A] = |cpu, memory| { // BIT 3, D
		cpu.bit(memory, 3, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x5B] = |cpu, memory| { // BIT 3, E
		cpu.bit(memory, 3, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x5C] = |cpu, memory| { // BIT 3, H
		cpu.bit(memory, 3, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x5D] = |cpu, memory| { // BIT 3, L
		cpu.bit(memory, 3, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x5E] = |cpu, memory| { // BIT 3, (HL)
        cpu.bit(memory, 3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x5F] = |cpu, memory| { // BIT 3, A
		cpu.bit(memory, 3, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0x60] = |cpu, memory| { // BIT 4, B
		cpu.bit(memory, 4, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x61] = |cpu, memory| { // BIT 4, C
		cpu.bit(memory, 4, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x62] = |cpu, memory| { // BIT 4, D
		cpu.bit(memory, 4, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x63] = |cpu, memory| { // BIT 4, E
		cpu.bit(memory, 4, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x64] = |cpu, memory| { // BIT 4, H
		cpu.bit(memory, 4, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x65] = |cpu, memory| { // BIT 4, L
		cpu.bit(memory, 4, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x66] = |cpu, memory| { // BIT 4, (HL)
        cpu.bit(memory, 4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x67] = |cpu, memory| { // BIT 4, A
		cpu.bit(memory, 4, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0x68] = |cpu, memory| { // BIT 5, B
		cpu.bit(memory, 5, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x69] = |cpu, memory| { // BIT 5, C
		cpu.bit(memory, 5, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x6A] = |cpu, memory| { // BIT 5, D
		cpu.bit(memory, 5, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x6B] = |cpu, memory| { // BIT 5, E
		cpu.bit(memory, 5, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x6C] = |cpu, memory| { // BIT 5, H
		cpu.bit(memory, 5, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x6D] = |cpu, memory| { // BIT 5, L
		cpu.bit(memory, 5, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x6E] = |cpu, memory| { // BIT 5, (HL)
        cpu.bit(memory, 5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x6F] = |cpu, memory| { // BIT 5, A
		cpu.bit(memory, 5, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0x70] = |cpu, memory| { // BIT 6, B
		cpu.bit(memory, 6, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x71] = |cpu, memory| { // BIT 6, C
		cpu.bit(memory, 6, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x72] = |cpu, memory| { // BIT 6, D
		cpu.bit(memory, 6, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x73] = |cpu, memory| { // BIT 6, E
		cpu.bit(memory, 6, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x74] = |cpu, memory| { // BIT 6, H
		cpu.bit(memory, 6, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x75] = |cpu, memory| { // BIT 6, L
		cpu.bit(memory, 6, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x76] = |cpu, memory| { // BIT 6, (HL)
        cpu.bit(memory, 6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x77] = |cpu, memory| { // BIT 6, A
		cpu.bit(memory, 6, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0x78] = |cpu, memory| { // BIT 7, B
		cpu.bit(memory, 7, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x79] = |cpu, memory| { // BIT 7, C
		cpu.bit(memory, 7, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x7A] = |cpu, memory| { // BIT 7, D
		cpu.bit(memory, 7, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x7B] = |cpu, memory| { // BIT 7, E
		cpu.bit(memory, 7, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x7C] = |cpu, memory| { // BIT 7, H
		cpu.bit(memory, 7, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x7D] = |cpu, memory| { // BIT 7, L
		cpu.bit(memory, 7, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x7E] = |cpu, memory| { // BIT 7, (HL)
        cpu.bit(memory, 7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x7F] = |cpu, memory| { // BIT 7, A
		cpu.bit(memory, 7, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0x80] = |cpu, memory| { // RES 0, B
		cpu.res(memory, 0, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x81] = |cpu, memory| { // RES 0, C
		cpu.res(memory, 0, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x82] = |cpu, memory| { // RES 0, D
		cpu.res(memory, 0, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x83] = |cpu, memory| { // RES 0, E
		cpu.res(memory, 0, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x84] = |cpu, memory| { // RES 0, H
		cpu.res(memory, 0, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x85] = |cpu, memory| { // RES 0, L
		cpu.res(memory, 0, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x86] = |cpu, memory| { // RES 0, (HL)
        cpu.res(memory, 0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x87] = |cpu, memory| { // RES 0, A
		cpu.res(memory, 0, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0x88] = |cpu, memory| { // RES 1, B
		cpu.res(memory, 1, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x89] = |cpu, memory| { // RES 1, C
		cpu.res(memory, 1, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x8A] = |cpu, memory| { // RES 1, D
		cpu.res(memory, 1, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x8B] = |cpu, memory| { // RES 1, E
		cpu.res(memory, 1, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x8C] = |cpu, memory| { // RES 1, H
		cpu.res(memory, 1, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x8D] = |cpu, memory| { // RES 1, L
		cpu.res(memory, 1, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x8E] = |cpu, memory| { // RES 1, (HL)
        cpu.res(memory, 1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x8F] = |cpu, memory| { // RES 1, A
		cpu.res(memory, 1, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0x90] = |cpu, memory| { // RES 2, B
		cpu.res(memory, 2, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x91] = |cpu, memory| { // RES 2, C
		cpu.res(memory, 2, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x92] = |cpu, memory| { // RES 2, D
		cpu.res(memory, 2, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x93] = |cpu, memory| { // RES 2, E
		cpu.res(memory, 2, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x94] = |cpu, memory| { // RES 2, H
		cpu.res(memory, 2, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x95] = |cpu, memory| { // RES 2, L
		cpu.res(memory, 2, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x96] = |cpu, memory| { // RES 2, (HL)
        cpu.res(memory, 2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x97] = |cpu, memory| { // RES 2, A
		cpu.res(memory, 2, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0x98] = |cpu, memory| { // RES 3, B
		cpu.res(memory, 3, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x99] = |cpu, memory| { // RES 3, C
		cpu.res(memory, 3, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x9A] = |cpu, memory| { // RES 3, D
		cpu.res(memory, 3, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x9B] = |cpu, memory| { // RES 3, E
		cpu.res(memory, 3, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x9C] = |cpu, memory| { // RES 3, H
		cpu.res(memory, 3, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x9D] = |cpu, memory| { // RES 3, L
		cpu.res(memory, 3, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x9E] = |cpu, memory| { // RES 3, (HL)
        cpu.res(memory, 3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x9F] = |cpu, memory| { // RES 3, A
		cpu.res(memory, 3, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0xA0] = |cpu, memory| { // RES 4, B
		cpu.res(memory, 4, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xA1] = |cpu, memory| { // RES 4, C
		cpu.res(memory, 4, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xA2] = |cpu, memory| { // RES 4, D
		cpu.res(memory, 4, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xA3] = |cpu, memory| { // RES 4, E
		cpu.res(memory, 4, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xA4] = |cpu, memory| { // RES 4, H
		cpu.res(memory, 4, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xA5] = |cpu, memory| { // RES 4, L
		cpu.res(memory, 4, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xA6] = |cpu, memory| { // RES 4, (HL)
        cpu.res(memory, 4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xA7] = |cpu, memory| { // RES 4, A
		cpu.res(memory, 4, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0xA8] = |cpu, memory| { // RES 5, B
		cpu.res(memory, 5, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xA9] = |cpu, memory| { // RES 5, C
		cpu.res(memory, 5, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xAA] = |cpu, memory| { // RES 5, D
		cpu.res(memory, 5, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xAB] = |cpu, memory| { // RES 5, E
		cpu.res(memory, 5, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xAC] = |cpu, memory| { // RES 5, H
		cpu.res(memory, 5, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xAD] = |cpu, memory| { // RES 5, L
		cpu.res(memory, 5, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xAE] = |cpu, memory| { // RES 5, (HL)
        cpu.res(memory, 5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xAF] = |cpu, memory| { // RES 5, A
		cpu.res(memory, 5, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0xB0] = |cpu, memory| { // RES 6, B
		cpu.res(memory, 6, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xB1] = |cpu, memory| { // RES 6, C
		cpu.res(memory, 6, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xB2] = |cpu, memory| { // RES 6, D
		cpu.res(memory, 6, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xB3] = |cpu, memory| { // RES 6, E
		cpu.res(memory, 6, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xB4] = |cpu, memory| { // RES 6, H
		cpu.res(memory, 6, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xB5] = |cpu, memory| { // RES 6, L
		cpu.res(memory, 6, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xB6] = |cpu, memory| { // RES 6, (HL)
        cpu.res(memory, 6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xB7] = |cpu, memory| { // RES 6, A
		cpu.res(memory, 6, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0xB8] = |cpu, memory| { // RES 7, B
		cpu.res(memory, 7, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xB9] = |cpu, memory| { // RES 7, C
		cpu.res(memory, 7, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xBA] = |cpu, memory| { // RES 7, D
		cpu.res(memory, 7, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xBB] = |cpu, memory| { // RES 7, E
		cpu.res(memory, 7, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xBC] = |cpu, memory| { // RES 7, H
		cpu.res(memory, 7, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xBD] = |cpu, memory| { // RES 7, L
		cpu.res(memory, 7, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xBE] = |cpu, memory| { // RES 7, (HL)
        cpu.res(memory, 7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xBF] = |cpu, memory| { // RES 7, A
		cpu.res(memory, 7, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0xC0] = |cpu, memory| { // SET 0, B
		cpu.set(memory, 0, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xC1] = |cpu, memory| { // SET 0, C
		cpu.set(memory, 0, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xC2] = |cpu, memory| { // SET 0, D
		cpu.set(memory, 0, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xC3] = |cpu, memory| { // SET 0, E
		cpu.set(memory, 0, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xC4] = |cpu, memory| { // SET 0, H
		cpu.set(memory, 0, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xC5] = |cpu, memory| { // SET 0, L
		cpu.set(memory, 0, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xC6] = |cpu, memory| { // SET 0, (HL)
        cpu.set(memory, 0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xC7] = |cpu, memory| { // SET 0, A
		cpu.set(memory, 0, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0xC8] = |cpu, memory| { // SET 1, B
		cpu.set(memory, 1, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xC9] = |cpu, memory| { // SET 1, C
		cpu.set(memory, 1, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xCA] = |cpu, memory| { // SET 1, D
		cpu.set(memory, 1, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xCB] = |cpu, memory| { // SET 1, E
		cpu.set(memory, 1, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xCC] = |cpu, memory| { // SET 1, H
		cpu.set(memory, 1, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xCD] = |cpu, memory| { // SET 1, L
		cpu.set(memory, 1, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xCE] = |cpu, memory| { // SET 1, (HL)
        cpu.set(memory, 1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xCF] = |cpu, memory| { // SET 1, A
		cpu.set(memory, 1, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0xD0] = |cpu, memory| { // SET 2, B
		cpu.set(memory, 2, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xD1] = |cpu, memory| { // SET 2, C
		cpu.set(memory, 2, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xD2] = |cpu, memory| { // SET 2, D
		cpu.set(memory, 2, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xD3] = |cpu, memory| { // SET 2, E
		cpu.set(memory, 2, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xD4] = |cpu, memory| { // SET 2, H
		cpu.set(memory, 2, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xD5] = |cpu, memory| { // SET 2, L
		cpu.set(memory, 2, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xD6] = |cpu, memory| { // SET 2, (HL)
        cpu.set(memory, 2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xD7] = |cpu, memory| { // SET 2, A
		cpu.set(memory, 2, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0xD8] = |cpu, memory| { // SET 3, B
		cpu.set(memory, 3, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xD9] = |cpu, memory| { // SET 3, C
		cpu.set(memory, 3, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xDA] = |cpu, memory| { // SET 3, D
		cpu.set(memory, 3, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xDB] = |cpu, memory| { // SET 3, E
		cpu.set(memory, 3, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xDC] = |cpu, memory| { // SET 3, H
		cpu.set(memory, 3, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xDD] = |cpu, memory| { // SET 3, L
		cpu.set(memory, 3, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xDE] = |cpu, memory| { // SET 3, (HL)
        cpu.set(memory, 3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xDF] = |cpu, memory| { // SET 3, A
		cpu.set(memory, 3, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0xE0] = |cpu, memory| { // SET 4, B
		cpu.set(memory, 4, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xE1] = |cpu, memory| { // SET 4, C
		cpu.set(memory, 4, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xE2] = |cpu, memory| { // SET 4, D
		cpu.set(memory, 4, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xE3] = |cpu, memory| { // SET 4, E
		cpu.set(memory, 4, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xE4] = |cpu, memory| { // SET 4, H
		cpu.set(memory, 4, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xE5] = |cpu, memory| { // SET 4, L
		cpu.set(memory, 4, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xE6] = |cpu, memory| { // SET 4, (HL)
        cpu.set(memory, 4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xE7] = |cpu, memory| { // SET 4, A
		cpu.set(memory, 4, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0xE8] = |cpu, memory| { // SET 5, B
		cpu.set(memory, 5, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xE9] = |cpu, memory| { // SET 5, C
		cpu.set(memory, 5, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xEA] = |cpu, memory| { // SET 5, D
		cpu.set(memory, 5, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xEB] = |cpu, memory| { // SET 5, E
		cpu.set(memory, 5, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xEC] = |cpu, memory| { // SET 5, H
		cpu.set(memory, 5, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xED] = |cpu, memory| { // SET 5, L
		cpu.set(memory, 5, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xEE] = |cpu, memory| { // SET 5, (HL)
        cpu.set(memory, 5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xEF] = |cpu, memory| { // SET 5, A
		cpu.set(memory, 5, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0xF0] = |cpu, memory| { // SET 6, B
		cpu.set(memory, 6, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xF1] = |cpu, memory| { // SET 6, C
		cpu.set(memory, 6, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xF2] = |cpu, memory| { // SET 6, D
		cpu.set(memory, 6, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xF3] = |cpu, memory| { // SET 6, E
		cpu.set(memory, 6, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xF4] = |cpu, memory| { // SET 6, H
		cpu.set(memory, 6, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xF5] = |cpu, memory| { // SET 6, L
		cpu.set(memory, 6, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xF6] = |cpu, memory| { // SET 6, (HL)
        cpu.set(memory, 6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xF7] = |cpu, memory| { // SET 6, A
		cpu.set(memory, 6, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0xF8] = |cpu, memory| { // SET 7, B
		cpu.set(memory, 7, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xF9] = |cpu, memory| { // SET 7, C
		cpu.set(memory, 7, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xFA] = |cpu, memory| { // SET 7, D
		cpu.set(memory, 7, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xFB] = |cpu, memory| { // SET 7, E
		cpu.set(memory, 7, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xFC] = |cpu, memory| { // SET 7, H
		cpu.set(memory, 7, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xFD] = |cpu, memory| { // SET 7, L
		cpu.set(memory, 7, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xFE] = |cpu, memory| { // SET 7, (HL)
        cpu.set(memory, 7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xFF] = |cpu, memory| { // SET 7, A
		cpu.set(memory, 7, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table
}