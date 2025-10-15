use crate::gameboy::cpu::{self, CPU};

pub trait IntOperand<T> {
    fn get(&self, cpu: &mut CPU) -> T;
    fn set(&self, value: T, cpu: &mut CPU);
}

pub struct RegisterOperand8(pub cpu::Register8);
impl IntOperand<u8> for RegisterOperand8 {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU) -> u8 {
        match self.0 {
            cpu::Register8::A => cpu.af[1],
            cpu::Register8::F => cpu.af[0],
            cpu::Register8::B => cpu.bc[1],
            cpu::Register8::C => cpu.bc[0],
            cpu::Register8::D => cpu.de[1],
            cpu::Register8::E => cpu.de[0],
            cpu::Register8::H => cpu.hl[1],
            cpu::Register8::L => cpu.hl[0]
        }
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut CPU) {
        match self.0 {
            cpu::Register8::A => cpu.af[1] = value,
            cpu::Register8::F => cpu.af[0] = value,
            cpu::Register8::B => cpu.bc[1] = value,
            cpu::Register8::C => cpu.bc[0] = value,
            cpu::Register8::D => cpu.de[1] = value,
            cpu::Register8::E => cpu.de[0] = value,
            cpu::Register8::H => cpu.hl[1] = value,
            cpu::Register8::L => cpu.hl[0] = value
        };
    }
}

pub struct ImmediateOperand8;
impl IntOperand<u8> for ImmediateOperand8 {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU) -> u8 {
        cpu.step_u8_and_wait()
    }
    #[inline(always)]
    fn set(&self, _: u8, _: &mut CPU) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct ImmediateSignedOperand8;
impl IntOperand<i8> for ImmediateSignedOperand8 {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU) -> i8 {
        cpu.step_u8_and_wait() as i8
    }
    #[inline(always)]
    fn set(&self, _: i8, _: &mut CPU) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct IndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for IndirectOperand8<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU) -> u8 {
        let address = self.0.get(cpu);
        cpu.read_u8_and_wait(address)
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut CPU) {
        let address = self.0.get(cpu);
        cpu.write_u8_and_wait(address, value);
    }
}
pub struct IncIndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for IncIndirectOperand8<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU) -> u8 {
        let address = self.0.get(cpu);
        self.0.set(address+1, cpu);
        cpu.read_u8_and_wait(address)
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut CPU) {
        let address = self.0.get(cpu);
        self.0.set(address+1, cpu);
        cpu.write_u8_and_wait(address, value);
    }
}
pub struct DecIndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for DecIndirectOperand8<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU) -> u8 {
        let address = self.0.get(cpu);
        self.0.set(address-1, cpu);
        cpu.read_u8_and_wait(address)
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut CPU) {
        let address = self.0.get(cpu);
        self.0.set(address-1, cpu);
        cpu.write_u8_and_wait(address, value);
    }
}

pub struct HramIndirectOperand<O: IntOperand<u8>>(pub O);
impl<O: IntOperand<u8>> HramIndirectOperand<O> {
    #[inline(always)]
    fn as_hram_address(&self, cpu: &mut CPU) -> u16 {
        0xFF00 & (self.0.get(cpu) as u16)
    }
}
impl<O: IntOperand<u8>> IntOperand<u8> for HramIndirectOperand<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU) -> u8 {
        let hram_address = self.as_hram_address(cpu);
        cpu.read_u8_and_wait(hram_address)
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut CPU) {
        let hram_address = self.as_hram_address(cpu);
        cpu.write_u8_and_wait(hram_address, value);
    }
}
impl<O: IntOperand<u16>> IntOperand<u16> for IndirectOperand8<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU) -> u16 {
        let address = self.0.get(cpu);
        u16::from_le_bytes([cpu.read_u8_and_wait(address), cpu.read_u8_and_wait(address+1)])
    }
    #[inline(always)]
    fn set(&self, value: u16, cpu: &mut CPU) {
        let address = self.0.get(cpu);
        let bytes = u16::to_le_bytes(value);
        cpu.write_u8_and_wait(address, bytes[0]);
        cpu.write_u8_and_wait(address+1, bytes[1]);
    }
}

pub struct RegisterOperand16(pub cpu::Register16);
impl IntOperand<u16> for RegisterOperand16 {
    #[inline(always)]
    fn get(&self, cpu: &mut CPU) -> u16 {
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
    fn set(&self, value: u16, cpu: &mut CPU) {
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
    fn get(&self, cpu: &mut CPU) -> u16 {
        u16::from_le_bytes([cpu.step_u8_and_wait(), cpu.step_u8_and_wait()])
    }
    #[inline(always)]
    fn set(&self, _: u16, _: &mut CPU) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct ConstOperand16(u16);
impl IntOperand<u16> for ConstOperand16 {
    #[inline(always)]
    fn get(&self, _: &mut CPU) -> u16 {
        self.0
    }
    #[inline(always)]
    fn set(&self, _: u16, _: &mut CPU) {
        panic!("Cannot write to constant operand")
    }
}

pub enum CondOperand {
    Unconditional,
    NZ,
    Z,
    NC,
    C
}
impl CondOperand {
    #[inline(always)]
    pub fn evaluate(&self, cpu: &CPU) -> bool {
        match self {
            Self::Unconditional => true,
            Self::NZ => cpu.af[0] & 0b10000000 == 0,
            Self::Z => cpu.af[0] & 0b10000000 != 0,
            Self::NC => cpu.af[0] & 0b00010000 == 0,
            Self::C => cpu.af[0] & 0b00010000 != 0
        }
    }
}

type OpHandler = fn(&mut CPU);
static OP_UNINIT: OpHandler = |_| {panic!("Unknown opcode")};

// Table definition
pub static OPCODE_TABLE: [OpHandler; 0x100] = init_opcode_table();
static CB_TABLE: [OpHandler; 0x100] = init_cb_table();

const fn init_opcode_table() -> [OpHandler; 0x100] {
    // Table initialization
    let mut opcode_table: [OpHandler; 0x100] = [OP_UNINIT; 0x100];
    opcode_table[0x00] = |_| { // NOP
        // Do nothing
    };
    opcode_table[0x01] = |cpu| { // LD BC, nn
        cpu.ld(RegisterOperand16(cpu::Register16::BC), ImmediateOperand16);
    };
    opcode_table[0x02] = |cpu| { // LD (BC), A
        cpu.ld(IndirectOperand8(RegisterOperand16(cpu::Register16::BC)), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x03] = |cpu| { // INC BC
        cpu.inc16(RegisterOperand16(cpu::Register16::BC));
    };
    opcode_table[0x04] = |cpu| { // INC B
        cpu.inc(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x05] = |cpu| { // DEC B
        cpu.dec(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x06] = |cpu| { // LD B, n
        cpu.ld(RegisterOperand8(cpu::Register8::B), ImmediateOperand8);
    };
    opcode_table[0x07] = |cpu| { // RLCA
        cpu.rlc(RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x08] = |cpu| { // LD (nn), SP
        cpu.ld(IndirectOperand8(ImmediateOperand16), RegisterOperand16(cpu::Register16::SP));
    };
    opcode_table[0x09] = |cpu| { // ADD HL, BC
        cpu.add_hl(RegisterOperand16(cpu::Register16::BC));
    };
    opcode_table[0x0A] = |cpu| { // LD A, (BC)
        cpu.ld(RegisterOperand8(cpu::Register8::A), IndirectOperand8(RegisterOperand16(cpu::Register16::BC)));
    };
    opcode_table[0x0B] = |cpu| { // DEC BC
        cpu.dec16(RegisterOperand16(cpu::Register16::BC));
    };
    opcode_table[0x0C] = |cpu| { // INC C
        cpu.inc(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x0D] = |cpu| { // DEC C
        cpu.dec(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x0E] = |cpu| { // LD C, n
        cpu.ld(RegisterOperand8(cpu::Register8::C), ImmediateOperand8)
    };
    opcode_table[0x0F] = |cpu| { // RRCA
        cpu.rrc(RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x10] = |cpu| { // STOP
        cpu.stop();
    };
    opcode_table[0x11] = |cpu| { // LD DE, nn
        cpu.ld(RegisterOperand16(cpu::Register16::DE), ImmediateOperand16);
    };
    opcode_table[0x12] = |cpu| { // LD (DE), A
        cpu.ld(IndirectOperand8(RegisterOperand16(cpu::Register16::DE)), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x13] = |cpu| { // INC DE
        cpu.inc16(RegisterOperand16(cpu::Register16::DE));
    };
    opcode_table[0x14] = |cpu| { // INC D
        cpu.inc(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x15] = |cpu| { // DEC D
        cpu.dec(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x16] = |cpu| { // LD D, n
        cpu.ld(RegisterOperand8(cpu::Register8::D), ImmediateOperand8);
    };
    opcode_table[0x17] = |cpu| { // RLA
        cpu.rl(RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x18] = |cpu| { // JR e
        cpu.jr(CondOperand::Unconditional, ImmediateSignedOperand8);
    };
    opcode_table[0x19] = |cpu| { // ADD HL, DE
        cpu.add_hl(RegisterOperand16(cpu::Register16::DE));
    };
    opcode_table[0x1A] = |cpu| { // LD A, (DE)
        cpu.ld(RegisterOperand8(cpu::Register8::A), IndirectOperand8(RegisterOperand16(cpu::Register16::DE)));
    };
    opcode_table[0x1B] = |cpu| { // DEC DE
        cpu.dec16(RegisterOperand16(cpu::Register16::DE));
    };
    opcode_table[0x1C] = |cpu| { // INC E
        cpu.inc(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x1D] = |cpu| { // DEC E
        cpu.dec(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x1E] = |cpu| { // LD E, n
        cpu.ld(RegisterOperand8(cpu::Register8::E), ImmediateOperand8);
    };
    opcode_table[0x1F] = |cpu| { // RRA
        cpu.rr(RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x20] = |cpu| { // JR NZ, e
        cpu.jr(CondOperand::NZ, ImmediateSignedOperand8);
    };
    opcode_table[0x21] = |cpu| { // LD HL, nn
        cpu.ld(RegisterOperand16(cpu::Register16::HL), ImmediateOperand16);
    };
    opcode_table[0x22] = |cpu| { // LD (HL+), A
        cpu.ld(IncIndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x23] = |cpu| { // INC HL
        cpu.inc16(RegisterOperand16(cpu::Register16::HL));
    };
    opcode_table[0x24] = |cpu| { // INC H
        cpu.inc(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x25] = |cpu| { // DEC H
        cpu.dec(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x26] = |cpu| { // LD H, n
        cpu.ld(RegisterOperand8(cpu::Register8::H), ImmediateOperand8);
    };
    opcode_table[0x27] = |cpu| { // DAA
        cpu.daa()
    };
    opcode_table[0x28] = |cpu| { // JR Z, e
        cpu.jr(CondOperand::Z, ImmediateSignedOperand8);
    };
    opcode_table[0x29] = |cpu| { // ADD HL, HL
        cpu.add_hl(RegisterOperand16(cpu::Register16::HL));
    };
    opcode_table[0x2A] = |cpu| { // LD A, (HL+)
        cpu.ld(RegisterOperand8(cpu::Register8::A), IncIndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x2B] = |cpu| { // DEC HL
        cpu.dec16(RegisterOperand16(cpu::Register16::HL));
    };
    opcode_table[0x2C] = |cpu| { // INC L
        cpu.inc(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x2D] = |cpu| { // DEC L
        cpu.dec(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x2E] = |cpu| { // LD L, n
        cpu.ld(RegisterOperand8(cpu::Register8::L), ImmediateOperand8);
    };
    opcode_table[0x2F] = |cpu| { // CPL
        cpu.cpl();
    };

    opcode_table[0x30] = |cpu| { // JR NC, e
        cpu.jr(CondOperand::NC, ImmediateSignedOperand8);
    };
    opcode_table[0x31] = |cpu| { // LD SP, nn
        cpu.ld(RegisterOperand16(cpu::Register16::SP), ImmediateOperand16);
    };
    opcode_table[0x32] = |cpu| { // LD (HL-), A
        cpu.ld(DecIndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x33] = |cpu| { // INC SP
        cpu.inc16(RegisterOperand16(cpu::Register16::SP));
    };
    opcode_table[0x34] = |cpu| { // INC (HL)
        cpu.inc(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x35] = |cpu| { // DEC (HL)
        cpu.dec(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x36] = |cpu| { // LD (HL), n
        cpu.ld(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), ImmediateOperand8);
    };
    opcode_table[0x37] = |cpu| { // SCF
        cpu.scf();
    };
    opcode_table[0x38] = |cpu| { // JR C, e
        cpu.jr(CondOperand::C, ImmediateSignedOperand8);
    };
    opcode_table[0x39] = |cpu| { // ADD HL, SP
        cpu.add_hl(RegisterOperand16(cpu::Register16::SP));
    };
    opcode_table[0x3A] = |cpu| { // LD A, (HL-)
        cpu.ld(RegisterOperand8(cpu::Register8::A), DecIndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x3B] = |cpu| { // DEC SP
        cpu.dec16(RegisterOperand16(cpu::Register16::SP));
    };
    opcode_table[0x3C] = |cpu| { // INC A
        cpu.inc(RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x3D] = |cpu| { // DEC A
        cpu.dec(RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x3E] = |cpu| { // LD A, n
        cpu.ld(RegisterOperand8(cpu::Register8::A), ImmediateOperand8);
    };
    opcode_table[0x3F] = |cpu| { // CCF
        cpu.ccf();
    };

    opcode_table[0x40] = |cpu| { // LD B, B
        cpu.ld(RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x41] = |cpu| { // LD B, C
        cpu.ld(RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x42] = |cpu| { // LD B, D
        cpu.ld(RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x43] = |cpu| { // LD B, E
        cpu.ld(RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x44] = |cpu| { // LD B, H
        cpu.ld(RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x45] = |cpu| { // LD B, L
        cpu.ld(RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x46] = |cpu| { // LD B, (HL)
        cpu.ld(RegisterOperand8(cpu::Register8::B), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x47] = |cpu| { // LD B, A
        cpu.ld(RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x48] = |cpu| { // LD C, B
        cpu.ld(RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x49] = |cpu| { // LD C, C
        cpu.ld(RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x4A] = |cpu| { // LD C, D
        cpu.ld(RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x4B] = |cpu| { // LD C, E
        cpu.ld(RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x4C] = |cpu| { // LD C, H
        cpu.ld(RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x4D] = |cpu| { // LD C, L
        cpu.ld(RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x4E] = |cpu| { // LD C, (HL)
        cpu.ld(RegisterOperand8(cpu::Register8::C), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x4F] = |cpu| { // LD C, A
        cpu.ld(RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x50] = |cpu| { // LD D, B
        cpu.ld(RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x51] = |cpu| { // LD D, C
        cpu.ld(RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x52] = |cpu| { // LD D, D
        cpu.ld(RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x53] = |cpu| { // LD D, E
        cpu.ld(RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x54] = |cpu| { // LD D, H
        cpu.ld(RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x55] = |cpu| { // LD D, L
        cpu.ld(RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x56] = |cpu| { // LD D, (HL)
        cpu.ld(RegisterOperand8(cpu::Register8::D), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x57] = |cpu| { // LD D, A
        cpu.ld(RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x58] = |cpu| { // LD E, B
        cpu.ld(RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x59] = |cpu| { // LD E, C
        cpu.ld(RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x5A] = |cpu| { // LD E, D
        cpu.ld(RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x5B] = |cpu| { // LD E, E
        cpu.ld(RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x5C] = |cpu| { // LD E, H
        cpu.ld(RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x5D] = |cpu| { // LD E, L
        cpu.ld(RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x5E] = |cpu| { // LD E, (HL)
        cpu.ld(RegisterOperand8(cpu::Register8::E), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x5F] = |cpu| { // LD E, A
        cpu.ld(RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x60] = |cpu| { // LD H, B
        cpu.ld(RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x61] = |cpu| { // LD H, C
        cpu.ld(RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x62] = |cpu| { // LD H, D
        cpu.ld(RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x63] = |cpu| { // LD H, E
        cpu.ld(RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x64] = |cpu| { // LD H, H
        cpu.ld(RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x65] = |cpu| { // LD H, L
        cpu.ld(RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x66] = |cpu| { // LD H, (HL)
        cpu.ld(RegisterOperand8(cpu::Register8::H), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x67] = |cpu| { // LD H, A
        cpu.ld(RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x68] = |cpu| { // LD L, B
        cpu.ld(RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x69] = |cpu| { // LD L, C
        cpu.ld(RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x6A] = |cpu| { // LD L, D
        cpu.ld(RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x6B] = |cpu| { // LD L, E
        cpu.ld(RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x6C] = |cpu| { // LD L, H
        cpu.ld(RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x6D] = |cpu| { // LD L, L
        cpu.ld(RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x6E] = |cpu| { // LD L, (HL)
        cpu.ld(RegisterOperand8(cpu::Register8::L), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x6F] = |cpu| { // LD L, A
        cpu.ld(RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x70] = |cpu| { // LD (HL), B
        cpu.ld(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x71] = |cpu| { // LD (HL), C
        cpu.ld(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x72] = |cpu| { // LD (HL), D
        cpu.ld(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x73] = |cpu| { // LD (HL), E
        cpu.ld(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x74] = |cpu| { // LD (HL), H
        cpu.ld(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x75] = |cpu| { // LD (HL), L
        cpu.ld(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x76] = |cpu| { // HALT
        cpu.halt();
    };
    opcode_table[0x77] = |cpu| { // LD (HL), A
        cpu.ld(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x78] = |cpu| { // LD A, B
        cpu.ld(RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x79] = |cpu| { // LD A, C
        cpu.ld(RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x7A] = |cpu| { // LD A, D
        cpu.ld(RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x7B] = |cpu| { // LD A, E
        cpu.ld(RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x7C] = |cpu| { // LD A, H
        cpu.ld(RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x7D] = |cpu| { // LD A, L
        cpu.ld(RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x7E] = |cpu| { // LD A, (HL)
        cpu.ld(RegisterOperand8(cpu::Register8::A), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x7F] = |cpu| { // LD A, A
        cpu.ld(RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x80] = |cpu| { // ADD B
        cpu.add(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x81] = |cpu| { // ADD C
        cpu.add(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x82] = |cpu| { // ADD D
        cpu.add(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x83] = |cpu| { // ADD E
        cpu.add(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x84] = |cpu| { // ADD H
        cpu.add(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x85] = |cpu| { // ADD L
        cpu.add(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x86] = |cpu| { // ADD (HL)
        cpu.add(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x87] = |cpu| { // ADD A
        cpu.add(RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x88] = |cpu| { // ADC B
        cpu.adc(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x89] = |cpu| { // ADC C
        cpu.adc(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x8A] = |cpu| { // ADC D
        cpu.adc(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x8B] = |cpu| { // ADC E
        cpu.adc(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x8C] = |cpu| { // ADC H
        cpu.adc(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x8D] = |cpu| { // ADC L
        cpu.adc(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x8E] = |cpu| { // ADC (HL)
        cpu.adc(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x8F] = |cpu| { // ADC A
        cpu.adc(RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x90] = |cpu| { // SUB B
        cpu.sub(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x91] = |cpu| { // SUB C
        cpu.sub(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x92] = |cpu| { // SUB D
        cpu.sub(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x93] = |cpu| { // SUB E
        cpu.sub(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x94] = |cpu| { // SUB H
        cpu.sub(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x95] = |cpu| { // SUB L
        cpu.sub(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x96] = |cpu| { // SUB (HL)
        cpu.sub(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x97] = |cpu| { // SUB A
        cpu.sub(RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x98] = |cpu| { // SBC B
        cpu.sbc(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x99] = |cpu| { // SBC C
        cpu.sbc(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x9A] = |cpu| { // SBC D
        cpu.sbc(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x9B] = |cpu| { // SBC E
        cpu.sbc(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x9C] = |cpu| { // SBC H
        cpu.sbc(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x9D] = |cpu| { // SBC L
        cpu.sbc(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x9E] = |cpu| { // SBC (HL)
        cpu.sbc(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x9F] = |cpu| { // SBC A
        cpu.sbc(RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0xA0] = |cpu| { // AND B
        cpu.and(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0xA1] = |cpu| { // AND C
        cpu.and(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0xA2] = |cpu| { // AND D
        cpu.and(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0xA3] = |cpu| { // AND E
        cpu.and(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0xA4] = |cpu| { // AND H
        cpu.and(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0xA5] = |cpu| { // AND L
        cpu.and(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0xA6] = |cpu| { // AND (HL)
        cpu.and(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xA7] = |cpu| { // AND A
        cpu.and(RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0xA8] = |cpu| { // XOR B
        cpu.xor(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0xA9] = |cpu| { // XOR C
        cpu.xor(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0xAA] = |cpu| { // XOR D
        cpu.xor(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0xAB] = |cpu| { // XOR E
        cpu.xor(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0xAC] = |cpu| { // XOR H
        cpu.xor(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0xAD] = |cpu| { // XOR L
        cpu.xor(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0xAE] = |cpu| { // XOR (HL)
        cpu.xor(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xAF] = |cpu| { // XOR A
        cpu.xor(RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0xB0] = |cpu| { // OR B
        cpu.or(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0xB1] = |cpu| { // OR C
        cpu.or(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0xB2] = |cpu| { // OR D
        cpu.or(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0xB3] = |cpu| { // OR E
        cpu.or(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0xB4] = |cpu| { // OR H
        cpu.or(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0xB5] = |cpu| { // OR L
        cpu.or(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0xB6] = |cpu| { // OR (HL)
        cpu.or(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xB7] = |cpu| { // OR A
        cpu.or(RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0xB8] = |cpu| { // CP B
        cpu.cp(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0xB9] = |cpu| { // CP C
        cpu.cp(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0xBA] = |cpu| { // CP D
        cpu.cp(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0xBB] = |cpu| { // CP E
        cpu.cp(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0xBC] = |cpu| { // CP H
        cpu.cp(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0xBD] = |cpu| { // CP L
        cpu.cp(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0xBE] = |cpu| { // CP (HL)
        cpu.cp(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xBF] = |cpu| { // CP A
        cpu.cp(RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0xC0] = |cpu| { // RET NZ
        cpu.ret(CondOperand::NZ);
    };
    opcode_table[0xC1] = |cpu| { // POP BC
        cpu.pop(RegisterOperand16(cpu::Register16::BC));
    };
    opcode_table[0xC2] = |cpu| { // JP NZ, nn
        cpu.jp(CondOperand::NZ, ImmediateOperand16);
    };
    opcode_table[0xC3] = |cpu| { // JP nn
        cpu.jp(CondOperand::Unconditional, ImmediateOperand16);
    };
    opcode_table[0xC4] = |cpu| { // CALL NZ, nn
        cpu.call(CondOperand::NZ, ImmediateOperand16);
    };
    opcode_table[0xC5] = |cpu| { // PUSH BC
        cpu.push(RegisterOperand16(cpu::Register16::BC));
    };
    opcode_table[0xC6] = |cpu| { // ADD n
        cpu.add(ImmediateOperand8);
    };
    opcode_table[0xC7] = |cpu| { // RST 0x00
        cpu.call(CondOperand::Unconditional, ConstOperand16(0x0000));
    };
    opcode_table[0xC8] = |cpu| { // RET Z
        cpu.ret(CondOperand::Z);
    };
    opcode_table[0xC9] = |cpu| { // RET
        cpu.ret(CondOperand::Unconditional);
    };
    opcode_table[0xCA] = |cpu| { // JP Z, nn
        cpu.jp(CondOperand::Z, ImmediateOperand16);
    };
    opcode_table[0xCB] = |cpu| { // CB op
        cpu.ir = cpu.step_u8_and_wait();
        CB_TABLE[cpu.ir as usize](cpu);
    };
    opcode_table[0xCC] = |cpu| { // CALL Z, nn
        cpu.call(CondOperand::Z, ImmediateOperand16);
    };
    opcode_table[0xCD] = |cpu| { // CALL nn
        cpu.call(CondOperand::Unconditional, ImmediateOperand16);
    };
    opcode_table[0xCE] = |cpu| { // ADC n
        cpu.adc(ImmediateOperand8);
    };
    opcode_table[0xCF] = |cpu| { // RST 0x08
        cpu.call(CondOperand::Unconditional, ConstOperand16(0x0008));
    };

    opcode_table[0xD0] = |cpu| { // RET NC
        cpu.ret(CondOperand::NC);
    };
    opcode_table[0xD1] = |cpu| { // POP DE
        cpu.pop(RegisterOperand16(cpu::Register16::DE));
    };
    opcode_table[0xD2] = |cpu| { // JP NC, nn
        cpu.jp(CondOperand::NC, ImmediateOperand16);
    };
    // opcode_table[0xD3] = (invalid)
    opcode_table[0xD4] = |cpu| { // CALL NC, nn
        cpu.call(CondOperand::NC, ImmediateOperand16);
    };
    opcode_table[0xD5] = |cpu| { // PUSH DE
        cpu.push(RegisterOperand16(cpu::Register16::DE));
    };
    opcode_table[0xD6] = |cpu| { // SUB n
        cpu.sub(ImmediateOperand8);
    };
    opcode_table[0xD7] = |cpu| { // RST 0x10
        cpu.call(CondOperand::Unconditional, ConstOperand16(0x0010));
    };
    opcode_table[0xD8] = |cpu| { // RET C
        cpu.ret(CondOperand::C);
    };
    opcode_table[0xD9] = |cpu| { // RETI
        cpu.reti();
    };
    opcode_table[0xDA] = |cpu| { // JP C, nn
        cpu.jp(CondOperand::C, ImmediateOperand16);
    };
    // opcode_table[0xDB] = (invalid)
    opcode_table[0xDC] = |cpu| { // CALL C, nn
        cpu.call(CondOperand::C, ImmediateOperand16);
    };
    // opcode_table[0xDD] = (invalid)
    opcode_table[0xDE] = |cpu| { // SBC n
        cpu.sbc(ImmediateOperand8);
    };
    opcode_table[0xDF] = |cpu| { // RST 0x18
        cpu.call(CondOperand::Unconditional, ConstOperand16(0x0018));
    };

    opcode_table[0xE0] = |cpu| { // LDH (n), A
        cpu.ld(HramIndirectOperand(ImmediateOperand8), RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0xE1] = |cpu| { // POP HL
        cpu.pop(RegisterOperand16(cpu::Register16::HL));
    };
    opcode_table[0xE2] = |cpu| { // LDH (C), A
        cpu.ld(HramIndirectOperand(RegisterOperand8(cpu::Register8::C)), RegisterOperand8(cpu::Register8::A));
    };
    // opcode_table[0xE3] = (invalid)
    // opcode_table[0xE4] = (invalid)
    opcode_table[0xE5] = |cpu| { // PUSH HL
        cpu.push(RegisterOperand16(cpu::Register16::HL));
    };
    opcode_table[0xE6] = |cpu| { // AND n
        cpu.and(ImmediateOperand8);
    };
    opcode_table[0xE7] = |cpu| { // RST 0x20
        cpu.call(CondOperand::Unconditional, ConstOperand16(0x0020));
    };
    opcode_table[0xE8] = |cpu| { // ADD SP, e
        cpu.add_spe();
    };
    opcode_table[0xE9] = |cpu| { // JP HL
        cpu.ld(RegisterOperand16(cpu::Register16::PC), RegisterOperand16(cpu::Register16::HL));
    };
    opcode_table[0xEA] = |cpu| { // LD (nn), A
        cpu.ld(IndirectOperand8(ImmediateOperand16), RegisterOperand8(cpu::Register8::A));
    };
    // opcode_table[0xEB] = (invalid)
    // opcode_table[0xEC] = (invalid)
    // opcode_table[0xED] = (invalid)
    opcode_table[0xEE] = |cpu| { // XOR n
        cpu.xor(ImmediateOperand8);
    };
    opcode_table[0xEF] = |cpu| { // RST 0x28
        cpu.call(CondOperand::Unconditional, ConstOperand16(0x0028));
    };

    opcode_table[0xF0] = |cpu| { // LDH A, (n)
        cpu.ld(RegisterOperand8(cpu::Register8::A), HramIndirectOperand(ImmediateOperand8));
    };
    opcode_table[0xF1] = |cpu| { // POP AF
        cpu.pop(RegisterOperand16(cpu::Register16::AF));
    };
    opcode_table[0xF2] = |cpu| { // LDH A, (C)
        cpu.ld(RegisterOperand8(cpu::Register8::A), HramIndirectOperand(RegisterOperand8(cpu::Register8::C)));
    };
    opcode_table[0xF3] = |cpu| { // DI
        cpu.di();
    };
    // opcode_table[0xF4] = (invalid)
    opcode_table[0xF5] = |cpu| { // PUSH AF
        cpu.push(RegisterOperand16(cpu::Register16::AF));
    };
    opcode_table[0xF6] = |cpu| { // OR n
        cpu.or(ImmediateOperand8);
    };
    opcode_table[0xF7] = |cpu| { // RST 0x30
        cpu.call(CondOperand::Unconditional, ConstOperand16(0x0030));
    };
    opcode_table[0xF8] = |cpu| { // LD HL, SP+e
        cpu.ld_hlspe();
    };
    opcode_table[0xF9] = |cpu| { // LD SP, HL
        cpu.ld(RegisterOperand16(cpu::Register16::SP), RegisterOperand16(cpu::Register16::HL));
        
    };
    opcode_table[0xFA] = |cpu| { // LD A, (nn)
        cpu.ld(RegisterOperand8(cpu::Register8::A), IndirectOperand8(ImmediateOperand16));
    };
    opcode_table[0xFB] = |cpu| { // EI
        cpu.ei();
    };
    // opcode_table[0xFC] = (invalid)
    // opcode_table[0xFD] = (invalid)
    opcode_table[0xFE] = |cpu| { // CP n
        cpu.cp(ImmediateOperand8);
    };
    opcode_table[0xFF] = |cpu| { // RST 0x38
        cpu.call(CondOperand::Unconditional, ConstOperand16(0x0038));
    };

    opcode_table
}

const fn init_cb_table() -> [OpHandler; 0x100] {
    let mut opcode_table: [OpHandler; 0x100] = [OP_UNINIT; 0x100];
    opcode_table[0x00] = |cpu| { // RLC B
        cpu.rlc(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x01] = |cpu| { // RLC C
        cpu.rlc(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x02] = |cpu| { // RLC D
        cpu.rlc(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x03] = |cpu| { // RLC E
        cpu.rlc(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x04] = |cpu| { // RLC H
        cpu.rlc(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x05] = |cpu| { // RLC L
        cpu.rlc(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x06] = |cpu| { // RLC (HL)
        cpu.rlc(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x07] = |cpu| { // RLC A
        cpu.rlc(RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x08] = |cpu| { // RRC B
        cpu.rrc(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x09] = |cpu| { // RRC C
        cpu.rrc(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x0A] = |cpu| { // RRC D
        cpu.rrc(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x0B] = |cpu| { // RRC E
        cpu.rrc(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x0C] = |cpu| { // RRC H
        cpu.rrc(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x0D] = |cpu| { // RRC L
        cpu.rrc(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x0E] = |cpu| { // RRC (HL)
        cpu.rrc(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x0F] = |cpu| { // RRC A
        cpu.rrc(RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x10] = |cpu| { // RL B
        cpu.rl(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x11] = |cpu| { // RL C
        cpu.rl(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x12] = |cpu| { // RL D
        cpu.rl(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x13] = |cpu| { // RL E
        cpu.rl(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x14] = |cpu| { // RL H
        cpu.rl(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x15] = |cpu| { // RL L
        cpu.rl(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x16] = |cpu| { // RL (HL)
        cpu.rl(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x17] = |cpu| { // RL A
        cpu.rl(RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x18] = |cpu| { // RR B
        cpu.rr(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x19] = |cpu| { // RR C
        cpu.rr(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x1A] = |cpu| { // RR D
        cpu.rr(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x1B] = |cpu| { // RR E
        cpu.rr(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x1C] = |cpu| { // RR H
        cpu.rr(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x1D] = |cpu| { // RR L
        cpu.rr(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x1E] = |cpu| { // RR (HL)
        cpu.rr(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x1F] = |cpu| { // RR A
        cpu.rr(RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x20] = |cpu| { // SLA B
        cpu.sla(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x21] = |cpu| { // SLA C
        cpu.sla(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x22] = |cpu| { // SLA D
        cpu.sla(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x23] = |cpu| { // SLA E
        cpu.sla(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x24] = |cpu| { // SLA H
        cpu.sla(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x25] = |cpu| { // SLA L
        cpu.sla(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x26] = |cpu| { // SLA (HL)
        cpu.sla(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x27] = |cpu| { // SLA A
        cpu.sla(RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x28] = |cpu| { // SRA B
        cpu.sra(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x29] = |cpu| { // SRA C
        cpu.sra(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x2A] = |cpu| { // SRA D
        cpu.sra(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x2B] = |cpu| { // SRA E
        cpu.sra(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x2C] = |cpu| { // SRA H
        cpu.sra(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x2D] = |cpu| { // SRA L
        cpu.sra(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x2E] = |cpu| { // SRA (HL)
        cpu.sra(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x2F] = |cpu| { // SRA A
        cpu.sra(RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x30] = |cpu| { // SWAP B
        cpu.swap(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x31] = |cpu| { // SWAP C
        cpu.swap(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x32] = |cpu| { // SWAP D
        cpu.swap(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x33] = |cpu| { // SWAP E
        cpu.swap(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x34] = |cpu| { // SWAP H
        cpu.swap(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x35] = |cpu| { // SWAP L
        cpu.swap(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x36] = |cpu| { // SWAP (HL)
        cpu.swap(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x37] = |cpu| { // SWAP A
        cpu.swap(RegisterOperand8(cpu::Register8::A));
    };
    opcode_table[0x38] = |cpu| { // SRL B
        cpu.srl(RegisterOperand8(cpu::Register8::B));
    };
    opcode_table[0x39] = |cpu| { // SRL C
        cpu.srl(RegisterOperand8(cpu::Register8::C));
    };
    opcode_table[0x3A] = |cpu| { // SRL D
        cpu.srl(RegisterOperand8(cpu::Register8::D));
    };
    opcode_table[0x3B] = |cpu| { // SRL E
        cpu.srl(RegisterOperand8(cpu::Register8::E));
    };
    opcode_table[0x3C] = |cpu| { // SRL H
        cpu.srl(RegisterOperand8(cpu::Register8::H));
    };
    opcode_table[0x3D] = |cpu| { // SRL L
        cpu.srl(RegisterOperand8(cpu::Register8::L));
    };
    opcode_table[0x3E] = |cpu| { // SRL (HL)
        cpu.srl(IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x3F] = |cpu| { // SRL A
        cpu.srl(RegisterOperand8(cpu::Register8::A));
    };

    opcode_table[0x40] = |cpu| { // BIT 0, B
		cpu.bit(0, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x41] = |cpu| { // BIT 0, C
		cpu.bit(0, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x42] = |cpu| { // BIT 0, D
		cpu.bit(0, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x43] = |cpu| { // BIT 0, E
		cpu.bit(0, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x44] = |cpu| { // BIT 0, H
		cpu.bit(0, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x45] = |cpu| { // BIT 0, L
		cpu.bit(0, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x46] = |cpu| { // BIT 0, (HL)
        cpu.bit(0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x47] = |cpu| { // BIT 0, A
		cpu.bit(0, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0x48] = |cpu| { // BIT 1, B
		cpu.bit(1, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x49] = |cpu| { // BIT 1, C
		cpu.bit(1, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x4A] = |cpu| { // BIT 1, D
		cpu.bit(1, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x4B] = |cpu| { // BIT 1, E
		cpu.bit(1, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x4C] = |cpu| { // BIT 1, H
		cpu.bit(1, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x4D] = |cpu| { // BIT 1, L
		cpu.bit(1, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x4E] = |cpu| { // BIT 1, (HL)
        cpu.bit(1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x4F] = |cpu| { // BIT 1, A
		cpu.bit(1, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0x50] = |cpu| { // BIT 2, B
		cpu.bit(2, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x51] = |cpu| { // BIT 2, C
		cpu.bit(2, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x52] = |cpu| { // BIT 2, D
		cpu.bit(2, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x53] = |cpu| { // BIT 2, E
		cpu.bit(2, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x54] = |cpu| { // BIT 2, H
		cpu.bit(2, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x55] = |cpu| { // BIT 2, L
		cpu.bit(2, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x56] = |cpu| { // BIT 2, (HL)
        cpu.bit(2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x57] = |cpu| { // BIT 2, A
		cpu.bit(2, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0x58] = |cpu| { // BIT 3, B
		cpu.bit(3, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x59] = |cpu| { // BIT 3, C
		cpu.bit(3, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x5A] = |cpu| { // BIT 3, D
		cpu.bit(3, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x5B] = |cpu| { // BIT 3, E
		cpu.bit(3, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x5C] = |cpu| { // BIT 3, H
		cpu.bit(3, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x5D] = |cpu| { // BIT 3, L
		cpu.bit(3, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x5E] = |cpu| { // BIT 3, (HL)
        cpu.bit(3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x5F] = |cpu| { // BIT 3, A
		cpu.bit(3, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0x60] = |cpu| { // BIT 4, B
		cpu.bit(4, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x61] = |cpu| { // BIT 4, C
		cpu.bit(4, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x62] = |cpu| { // BIT 4, D
		cpu.bit(4, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x63] = |cpu| { // BIT 4, E
		cpu.bit(4, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x64] = |cpu| { // BIT 4, H
		cpu.bit(4, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x65] = |cpu| { // BIT 4, L
		cpu.bit(4, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x66] = |cpu| { // BIT 4, (HL)
        cpu.bit(4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x67] = |cpu| { // BIT 4, A
		cpu.bit(4, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0x68] = |cpu| { // BIT 5, B
		cpu.bit(5, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x69] = |cpu| { // BIT 5, C
		cpu.bit(5, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x6A] = |cpu| { // BIT 5, D
		cpu.bit(5, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x6B] = |cpu| { // BIT 5, E
		cpu.bit(5, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x6C] = |cpu| { // BIT 5, H
		cpu.bit(5, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x6D] = |cpu| { // BIT 5, L
		cpu.bit(5, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x6E] = |cpu| { // BIT 5, (HL)
        cpu.bit(5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x6F] = |cpu| { // BIT 5, A
		cpu.bit(5, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0x70] = |cpu| { // BIT 6, B
		cpu.bit(6, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x71] = |cpu| { // BIT 6, C
		cpu.bit(6, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x72] = |cpu| { // BIT 6, D
		cpu.bit(6, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x73] = |cpu| { // BIT 6, E
		cpu.bit(6, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x74] = |cpu| { // BIT 6, H
		cpu.bit(6, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x75] = |cpu| { // BIT 6, L
		cpu.bit(6, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x76] = |cpu| { // BIT 6, (HL)
        cpu.bit(6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x77] = |cpu| { // BIT 6, A
		cpu.bit(6, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0x78] = |cpu| { // BIT 7, B
		cpu.bit(7, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x79] = |cpu| { // BIT 7, C
		cpu.bit(7, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x7A] = |cpu| { // BIT 7, D
		cpu.bit(7, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x7B] = |cpu| { // BIT 7, E
		cpu.bit(7, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x7C] = |cpu| { // BIT 7, H
		cpu.bit(7, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x7D] = |cpu| { // BIT 7, L
		cpu.bit(7, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x7E] = |cpu| { // BIT 7, (HL)
        cpu.bit(7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x7F] = |cpu| { // BIT 7, A
		cpu.bit(7, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0x80] = |cpu| { // RES 0, B
		cpu.res(0, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x81] = |cpu| { // RES 0, C
		cpu.res(0, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x82] = |cpu| { // RES 0, D
		cpu.res(0, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x83] = |cpu| { // RES 0, E
		cpu.res(0, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x84] = |cpu| { // RES 0, H
		cpu.res(0, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x85] = |cpu| { // RES 0, L
		cpu.res(0, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x86] = |cpu| { // RES 0, (HL)
        cpu.res(0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x87] = |cpu| { // RES 0, A
		cpu.res(0, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0x88] = |cpu| { // RES 1, B
		cpu.res(1, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x89] = |cpu| { // RES 1, C
		cpu.res(1, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x8A] = |cpu| { // RES 1, D
		cpu.res(1, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x8B] = |cpu| { // RES 1, E
		cpu.res(1, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x8C] = |cpu| { // RES 1, H
		cpu.res(1, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x8D] = |cpu| { // RES 1, L
		cpu.res(1, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x8E] = |cpu| { // RES 1, (HL)
        cpu.res(1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x8F] = |cpu| { // RES 1, A
		cpu.res(1, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0x90] = |cpu| { // RES 2, B
		cpu.res(2, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x91] = |cpu| { // RES 2, C
		cpu.res(2, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x92] = |cpu| { // RES 2, D
		cpu.res(2, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x93] = |cpu| { // RES 2, E
		cpu.res(2, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x94] = |cpu| { // RES 2, H
		cpu.res(2, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x95] = |cpu| { // RES 2, L
		cpu.res(2, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x96] = |cpu| { // RES 2, (HL)
        cpu.res(2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x97] = |cpu| { // RES 2, A
		cpu.res(2, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0x98] = |cpu| { // RES 3, B
		cpu.res(3, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0x99] = |cpu| { // RES 3, C
		cpu.res(3, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0x9A] = |cpu| { // RES 3, D
		cpu.res(3, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0x9B] = |cpu| { // RES 3, E
		cpu.res(3, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0x9C] = |cpu| { // RES 3, H
		cpu.res(3, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0x9D] = |cpu| { // RES 3, L
		cpu.res(3, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0x9E] = |cpu| { // RES 3, (HL)
        cpu.res(3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0x9F] = |cpu| { // RES 3, A
		cpu.res(3, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0xA0] = |cpu| { // RES 4, B
		cpu.res(4, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xA1] = |cpu| { // RES 4, C
		cpu.res(4, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xA2] = |cpu| { // RES 4, D
		cpu.res(4, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xA3] = |cpu| { // RES 4, E
		cpu.res(4, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xA4] = |cpu| { // RES 4, H
		cpu.res(4, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xA5] = |cpu| { // RES 4, L
		cpu.res(4, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xA6] = |cpu| { // RES 4, (HL)
        cpu.res(4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xA7] = |cpu| { // RES 4, A
		cpu.res(4, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0xA8] = |cpu| { // RES 5, B
		cpu.res(5, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xA9] = |cpu| { // RES 5, C
		cpu.res(5, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xAA] = |cpu| { // RES 5, D
		cpu.res(5, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xAB] = |cpu| { // RES 5, E
		cpu.res(5, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xAC] = |cpu| { // RES 5, H
		cpu.res(5, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xAD] = |cpu| { // RES 5, L
		cpu.res(5, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xAE] = |cpu| { // RES 5, (HL)
        cpu.res(5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xAF] = |cpu| { // RES 5, A
		cpu.res(5, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0xB0] = |cpu| { // RES 6, B
		cpu.res(6, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xB1] = |cpu| { // RES 6, C
		cpu.res(6, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xB2] = |cpu| { // RES 6, D
		cpu.res(6, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xB3] = |cpu| { // RES 6, E
		cpu.res(6, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xB4] = |cpu| { // RES 6, H
		cpu.res(6, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xB5] = |cpu| { // RES 6, L
		cpu.res(6, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xB6] = |cpu| { // RES 6, (HL)
        cpu.res(6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xB7] = |cpu| { // RES 6, A
		cpu.res(6, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0xB8] = |cpu| { // RES 7, B
		cpu.res(7, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xB9] = |cpu| { // RES 7, C
		cpu.res(7, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xBA] = |cpu| { // RES 7, D
		cpu.res(7, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xBB] = |cpu| { // RES 7, E
		cpu.res(7, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xBC] = |cpu| { // RES 7, H
		cpu.res(7, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xBD] = |cpu| { // RES 7, L
		cpu.res(7, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xBE] = |cpu| { // RES 7, (HL)
        cpu.res(7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xBF] = |cpu| { // RES 7, A
		cpu.res(7, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0xC0] = |cpu| { // SET 0, B
		cpu.set(0, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xC1] = |cpu| { // SET 0, C
		cpu.set(0, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xC2] = |cpu| { // SET 0, D
		cpu.set(0, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xC3] = |cpu| { // SET 0, E
		cpu.set(0, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xC4] = |cpu| { // SET 0, H
		cpu.set(0, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xC5] = |cpu| { // SET 0, L
		cpu.set(0, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xC6] = |cpu| { // SET 0, (HL)
        cpu.set(0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xC7] = |cpu| { // SET 0, A
		cpu.set(0, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0xC8] = |cpu| { // SET 1, B
		cpu.set(1, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xC9] = |cpu| { // SET 1, C
		cpu.set(1, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xCA] = |cpu| { // SET 1, D
		cpu.set(1, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xCB] = |cpu| { // SET 1, E
		cpu.set(1, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xCC] = |cpu| { // SET 1, H
		cpu.set(1, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xCD] = |cpu| { // SET 1, L
		cpu.set(1, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xCE] = |cpu| { // SET 1, (HL)
        cpu.set(1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xCF] = |cpu| { // SET 1, A
		cpu.set(1, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0xD0] = |cpu| { // SET 2, B
		cpu.set(2, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xD1] = |cpu| { // SET 2, C
		cpu.set(2, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xD2] = |cpu| { // SET 2, D
		cpu.set(2, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xD3] = |cpu| { // SET 2, E
		cpu.set(2, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xD4] = |cpu| { // SET 2, H
		cpu.set(2, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xD5] = |cpu| { // SET 2, L
		cpu.set(2, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xD6] = |cpu| { // SET 2, (HL)
        cpu.set(2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xD7] = |cpu| { // SET 2, A
		cpu.set(2, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0xD8] = |cpu| { // SET 3, B
		cpu.set(3, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xD9] = |cpu| { // SET 3, C
		cpu.set(3, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xDA] = |cpu| { // SET 3, D
		cpu.set(3, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xDB] = |cpu| { // SET 3, E
		cpu.set(3, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xDC] = |cpu| { // SET 3, H
		cpu.set(3, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xDD] = |cpu| { // SET 3, L
		cpu.set(3, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xDE] = |cpu| { // SET 3, (HL)
        cpu.set(3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xDF] = |cpu| { // SET 3, A
		cpu.set(3, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0xE0] = |cpu| { // SET 4, B
		cpu.set(4, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xE1] = |cpu| { // SET 4, C
		cpu.set(4, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xE2] = |cpu| { // SET 4, D
		cpu.set(4, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xE3] = |cpu| { // SET 4, E
		cpu.set(4, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xE4] = |cpu| { // SET 4, H
		cpu.set(4, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xE5] = |cpu| { // SET 4, L
		cpu.set(4, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xE6] = |cpu| { // SET 4, (HL)
        cpu.set(4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xE7] = |cpu| { // SET 4, A
		cpu.set(4, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0xE8] = |cpu| { // SET 5, B
		cpu.set(5, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xE9] = |cpu| { // SET 5, C
		cpu.set(5, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xEA] = |cpu| { // SET 5, D
		cpu.set(5, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xEB] = |cpu| { // SET 5, E
		cpu.set(5, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xEC] = |cpu| { // SET 5, H
		cpu.set(5, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xED] = |cpu| { // SET 5, L
		cpu.set(5, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xEE] = |cpu| { // SET 5, (HL)
        cpu.set(5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xEF] = |cpu| { // SET 5, A
		cpu.set(5, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table[0xF0] = |cpu| { // SET 6, B
		cpu.set(6, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xF1] = |cpu| { // SET 6, C
		cpu.set(6, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xF2] = |cpu| { // SET 6, D
		cpu.set(6, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xF3] = |cpu| { // SET 6, E
		cpu.set(6, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xF4] = |cpu| { // SET 6, H
		cpu.set(6, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xF5] = |cpu| { // SET 6, L
		cpu.set(6, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xF6] = |cpu| { // SET 6, (HL)
        cpu.set(6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xF7] = |cpu| { // SET 6, A
		cpu.set(6, RegisterOperand8(cpu::Register8::A))        
    };
    opcode_table[0xF8] = |cpu| { // SET 7, B
		cpu.set(7, RegisterOperand8(cpu::Register8::B))        
    };
    opcode_table[0xF9] = |cpu| { // SET 7, C
		cpu.set(7, RegisterOperand8(cpu::Register8::C))        
    };
    opcode_table[0xFA] = |cpu| { // SET 7, D
		cpu.set(7, RegisterOperand8(cpu::Register8::D))        
    };
    opcode_table[0xFB] = |cpu| { // SET 7, E
		cpu.set(7, RegisterOperand8(cpu::Register8::E))        
    };
    opcode_table[0xFC] = |cpu| { // SET 7, H
		cpu.set(7, RegisterOperand8(cpu::Register8::H))        
    };
    opcode_table[0xFD] = |cpu| { // SET 7, L
		cpu.set(7, RegisterOperand8(cpu::Register8::L))        
    };
    opcode_table[0xFE] = |cpu| { // SET 7, (HL)
        cpu.set(7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
    };
    opcode_table[0xFF] = |cpu| { // SET 7, A
		cpu.set(7, RegisterOperand8(cpu::Register8::A))        
    };

    opcode_table
}