use std::{matches, unreachable};

use funty::Unsigned;

use crate::common::bit::BitVec;

pub enum ArmVersion {
    V4,
    V5,
    V6,
}

pub enum ArmFeatures {
    Thumb,    // Included as of ARMv6
    Dsp,      // Included as of ARMv6
    Jazelle,  // Included as of ARMv6
    KernelExt // Included as of ARMv7
}

pub struct ArmArchitecture {
    version: ArmVersion,
    features: ArmFeatures
}

pub enum ArmAddress {
    Inst32(u32),
    Thumb16(u16)
}

pub trait ArmCpu {
    fn fetch_instruction(&self) -> ArmAddress;
    fn get_architecture(&self) -> &ArmArchitecture;
}


pub fn decode_instruction(cpu: &dyn ArmCpu) -> ArmInstruction {
    match cpu.fetch_instruction() {
        ArmAddress::Inst32(inst) => decode_fullsize(cpu, inst),
        ArmAddress::Thumb16(thumb) => decode_thumb(cpu, thumb),
    }
}

#[inline]
fn decode_fullsize(cpu: &dyn ArmCpu, inst32: u32) -> ArmInstruction {
    let major_op = inst32.range_bits(27, 25);
    let cond = ArmCondition::from_inst32(inst32);

    if cond.is_special() {
        decode_fullsize_unconditional(cpu, major_op, inst32)
    } else {
        match major_op {
            0b000 => decode_fullsize_misc000(cpu, inst32),
            0b001 => decode_fullsize_dataprocessing(cpu, inst32, true),
            0b010 => decode_fullsize_load(cpu, inst32, false),
            0b011 => decode_fullsize_misc011(cpu, inst32),
            0b100 => decode_fullsize_loadmultiple(cpu, inst32),
            0b101 => decode_fullsize_branch(cpu, inst32),
            0b110 => decode_fullsize_loadcoprocessor(cpu, inst32),
            0b111 => decode_fullsize_coprocessor(cpu, inst32),
            _ => unreachable!()
        }
    }
}

#[inline]
fn decode_fullsize_unconditional(cpu: &dyn ArmCpu, major_op: u32, inst32: u32) -> ArmInstruction {
    match major_op {
        0b000 => match (inst32.range_bits(24, 20), inst32.test_bit(16)) {
            (0b10000, false) if inst32 & 0b100000 == 0 => ArmInstruction::CPS,
            (0b10000, true) if inst32 & 0b11100000000011110000 == 0 => ArmInstruction::SETEND,
            _ => ArmInstruction::Undefined,
        }
        0b001 => ArmInstruction::Undefined,
        0b010 | 0b011 if inst32 & 0b1011100001111000000000000 == 0b1010100001111000000000000 => ArmInstruction::PLD,
        0b100 => match (inst32.test_bit(22), inst32.test_bit(20)) {
            (true, false) => ArmInstruction::SRS,
            (false, true) => ArmInstruction::RFE,
            _ => ArmInstruction::Undefined,
        }
        0b101 => {
            let mut offset = extract_imm_branch_offset(inst32);
            offset.map_bit(1, inst32.test_bit(24));
            ArmInstruction::BLX{offset: BlxVariant::Immediate(offset)}
        }
        0b110 => match (inst32.range_bits(24, 21), inst32.test_bit(20)) {
            (0b0010, true) => ArmInstruction::MRRC2,
            (_, true) => ArmInstruction::LDC2,
            (0b0010, false) => ArmInstruction::MCRR2,
            (_, false) => ArmInstruction::STC2,
        }
        0b111 => match (inst32.test_bit(24), inst32.test_bit(20), inst32.test_bit(4)) {
            (false, _, false) => ArmInstruction::CDP2,
            (false, false, true) => ArmInstruction::MCR2,
            (false, true, true) => ArmInstruction::MRC2,
            _ => ArmInstruction::Undefined,
        }
        _ => unreachable!()
    }
}

#[inline]
fn decode_fullsize_misc000(cpu: &dyn ArmCpu, inst32: u32) -> ArmInstruction {
    let in_extension_space = inst32 & 0b10010000 == 0b10010000;
    let is_mult_extension = in_extension_space && (inst32 & 0b1_00000000_00000000_01100000 == 0);
    
    if is_mult_extension {
        let opcode = inst32.range_bits(23, 21);
        let s = inst32.test_bit(20);
        let cond = ArmCondition::from_inst32(inst32);
        match (opcode, s) {
            (0b000, _) => ArmInstruction::MUL,
            (0b001, _) => ArmInstruction::MLA,

            (0b100, _) => ArmInstruction::UMULL,
            (0b101, _) => ArmInstruction::UMLAL,
            (0b110, _) => ArmInstruction::SMULL,
            (0b111, _) => ArmInstruction::SMLAL(None),

            (0b010, false) => ArmInstruction::UMAAL,

            _ => ArmInstruction::Undefined
        }
    } else if in_extension_space { // Load/Store Extension
        let opcode = inst32.range_bits(24, 21);
        let s = inst32.test_bit(20);
        let shifter_opcode = inst32.range_bits(7, 4);
        let cond = ArmCondition::from_inst32(inst32);
        match (opcode, s, shifter_opcode) {
            (0b1000, false, 0b1001) => ArmInstruction::SWP,
            (0b1010, false, 0b1001) => ArmInstruction::SWPB,
            (0b1100, true, 0b1001) => ArmInstruction::LDREX,
            (0b1100, false, 0b1001) => ArmInstruction::STREX,
            (_, true, 0b1011) => ArmInstruction::LDRH,
            (_, false, 0b1011) => ArmInstruction::STRH,
            (_, true, 0b1101) => ArmInstruction::LDRSB,
            (_, false, 0b1101) => ArmInstruction::LDRD,
            (_, true, 0b1111) => ArmInstruction::LDRSH,
            (_, false, 0b1111) => ArmInstruction::STRD,

            _ => ArmInstruction::Undefined
        }
    } else {
        decode_fullsize_dataprocessing(cpu, inst32, false)
    } 
}

#[inline]
fn decode_fullsize_dataprocessing(cpu: &dyn ArmCpu, inst32: u32, i: bool) -> ArmInstruction {
    let opcode = inst32.range_bits(24, 21);
    let s = inst32.test_bit(20);
    let rn = inst32.range_bits(19, 16);
    let rd = inst32.range_bits(15, 12);
    let shifter = inst32.range_bits(11, 0);
    let shifter_opcode = inst32.range_bits(7, 4);
    let is_dataprocessing = i || !inst32.test_bit(7) || !inst32.test_bit(4);
    let cond = ArmCondition::from_inst32(inst32);
    
    match (opcode, s) {
        (0b0000, _) => ArmInstruction::AND,
        (0b0001, _) => ArmInstruction::EOR,
        (0b0010, _) => ArmInstruction::SUB,
        (0b0011, _) => ArmInstruction::RSB,
        (0b0100, _) => ArmInstruction::ADD,
        (0b0101, _) => ArmInstruction::ADC,
        (0b0110, _) => ArmInstruction::SBC,
        (0b0111, _) => ArmInstruction::RSC,
        (0b1000, true) => ArmInstruction::TST,
        (0b1000, false) => match (i, shifter_opcode) {
            (false, 0b0000) => ArmInstruction::MRS,
            (false, 0b0101) => ArmInstruction::QADD,
            (false, 0b1000) => ArmInstruction::SMLA(BitHalf::Bottom, BitHalf::Bottom),
            (false, 0b1010) => ArmInstruction::SMLA(BitHalf::Top, BitHalf::Bottom),
            (false, 0b1100) => ArmInstruction::SMLA(BitHalf::Bottom, BitHalf::Top),
            (false, 0b1110) => ArmInstruction::SMLA(BitHalf::Top, BitHalf::Top),

            _ => ArmInstruction::Undefined
        },
        (0b1001, true) => ArmInstruction::TEQ,
        (0b1001, false) => match (i, shifter_opcode) {
            (true, _) => ArmInstruction::MSR,
            (false, 0b0000) => ArmInstruction::MSR,
            (false, 0b0001) => ArmInstruction::BX,
            (false, 0b0010) => ArmInstruction::BXJ,
            (false, 0b0011) => ArmInstruction::BLX { offset: BlxVariant::Register },
            (false, 0b0101) => ArmInstruction::QSUB,
            (false, 0b0111) if matches!(cond, ArmCondition::Always) => ArmInstruction::BKPT,
            (false, 0b1000) => ArmInstruction::SMLAW(BitHalf::Bottom),
            (false, 0b1010) => ArmInstruction::SMULW(BitHalf::Bottom),
            (false, 0b1100) => ArmInstruction::SMLAW(BitHalf::Top),
            (false, 0b1110) => ArmInstruction::SMULW(BitHalf::Top),

            _ => ArmInstruction::Undefined
        },
        (0b1010, true) => ArmInstruction::CMP,
        (0b1010, false) => match (i, shifter_opcode) {
            (false, 0b0000) => ArmInstruction::MRS,
            (false, 0b0101) => ArmInstruction::QDADD,
            (false, 0b1000) => ArmInstruction::SMLAL(Some((BitHalf::Bottom, BitHalf::Bottom))),
            (false, 0b1010) => ArmInstruction::SMLAL(Some((BitHalf::Top, BitHalf::Bottom))),
            (false, 0b1100) => ArmInstruction::SMLAL(Some((BitHalf::Bottom, BitHalf::Top))),
            (false, 0b1110) => ArmInstruction::SMLAL(Some((BitHalf::Top, BitHalf::Top))),

            _ => ArmInstruction::Undefined
        },
        (0b1011, true) => ArmInstruction::CMN,
        (0b1011, false) => match (i, shifter_opcode) {
            (true, _) => ArmInstruction::MSR,
            (false, 0b0000) => ArmInstruction::MSR,
            (false, 0b0001) => ArmInstruction::CLZ,
            (false, 0b0101) => ArmInstruction::QDSUB,
            (false, 0b1000) => ArmInstruction::SMUL(BitHalf::Bottom, BitHalf::Bottom),
            (false, 0b1010) => ArmInstruction::SMUL(BitHalf::Top, BitHalf::Bottom),
            (false, 0b1100) => ArmInstruction::SMUL(BitHalf::Bottom, BitHalf::Top),
            (false, 0b1110) => ArmInstruction::SMUL(BitHalf::Top, BitHalf::Top),

            _ => ArmInstruction::Undefined
        },
        (0b1100, _) => ArmInstruction::ORR,
        (0b1101, _) => if inst32.range_bits(11, 4) == 0 {
            ArmInstruction::CPY
        } else {
            ArmInstruction::MOV
        }
        (0b1110, _) => ArmInstruction::BIC,
        (0b1111, _) => ArmInstruction::MVN,

        _ => ArmInstruction::Undefined
    }
}

#[inline]
fn decode_fullsize_misc011(cpu: &dyn ArmCpu, inst32: u32) -> ArmInstruction {
    let in_extension_space = inst32.test_bit(4);

    if in_extension_space {
        let p = inst32.test_bit(24);
        let u = inst32.test_bit(23);
        let b = inst32.test_bit(22);
        let w = inst32.test_bit(21);
        let l = inst32.test_bit(20);
        let shifter_opcode = inst32.range_bits(7, 4);
        match (p, u, b, w, l, shifter_opcode) {
            (true, true, true, true, true, 0b1111) => ArmInstruction::Undefined, // Architecturally Undefined

            (false, true, false, false, false, 0b0001 | 0b1001) => ArmInstruction::PKH(PkhForm::BT),
            (false, true, false, false, false, 0b0101 | 0b1101) => ArmInstruction::PKH(PkhForm::TB),

            (false, false, false, true, false, 0b0001) => ArmInstruction::QADD16,
            (false, false, false, true, false, 0b1001) => ArmInstruction::QADD8,
            (false, false, false, true, false, 0b0011) => ArmInstruction::QADDSUBX,
            (false, false, false, true, false, 0b0111) => ArmInstruction::QSUB16,
            (false, false, false, true, false, 0b1111) => ArmInstruction::QSUB8,
            (false, false, false, true, false, 0b0101) => ArmInstruction::QSUBADDX,
            (false, false, true, true, false, 0b0001) => ArmInstruction::UQADD16,
            (false, false, true, true, false, 0b1001) => ArmInstruction::UQADD8,
            (false, false, true, true, false, 0b0011) => ArmInstruction::UQADDSUBX,
            (false, false, true, true, false, 0b0111) => ArmInstruction::UQSUB16,
            (false, false, true, true, false, 0b1111) => ArmInstruction::UQSUB8,
            (false, false, true, true, false, 0b0101) => ArmInstruction::UQSUBADDX,

            (false, true, false, true, true, 0b0011) => ArmInstruction::REV,
            (false, true, false, true, true, 0b1011) => ArmInstruction::REV16,
            (false, true, true, true, true, 0b1011) => ArmInstruction::REVSH,

            (false, false, false, false, true, 0b0001) => ArmInstruction::SADD16,
            (false, false, false, false, true, 0b1001) => ArmInstruction::SADD8,
            (false, false, false, false, true, 0b0011) => ArmInstruction::SADDSUBX,
            (false, false, false, false, true, 0b0111) => ArmInstruction::SSUB16,
            (false, false, false, false, true, 0b1111) => ArmInstruction::SSUB8,
            (false, false, false, false, true, 0b0101) => ArmInstruction::SSUBADDX,
            (false, false, true, false, true, 0b0001) => ArmInstruction::UADD16,
            (false, false, true, false, true, 0b1001) => ArmInstruction::UADD8,
            (false, false, true, false, true, 0b0011) => ArmInstruction::UADDSUBX,
            (false, false, true, false, true, 0b0111) => ArmInstruction::USUB16,
            (false, false, true, false, true, 0b1111) => ArmInstruction::USUB8,
            (false, false, true, false, true, 0b0101) => ArmInstruction::USUBADDX,

            (false, false, false, true, true, 0b0001) => ArmInstruction::SHADD16,
            (false, false, false, true, true, 0b1001) => ArmInstruction::SHADD8,
            (false, false, false, true, true, 0b0011) => ArmInstruction::SHADDSUBX,
            (false, false, false, true, true, 0b0111) => ArmInstruction::SHSUB16,
            (false, false, false, true, true, 0b1111) => ArmInstruction::SHSUB8,
            (false, false, false, true, true, 0b0101) => ArmInstruction::SHSUBADDX,
            (false, false, true, true, true, 0b0001) => ArmInstruction::UHADD16,
            (false, false, true, true, true, 0b1001) => ArmInstruction::UHADD8,
            (false, false, true, true, true, 0b0011) => ArmInstruction::UHADDSUBX,
            (false, false, true, true, true, 0b0111) => ArmInstruction::UHSUB16,
            (false, false, true, true, true, 0b1111) => ArmInstruction::UHSUB8,
            (false, false, true, true, true, 0b0101) => ArmInstruction::UHSUBADDX,

            (false, true, false, false, false, 0b1011) => ArmInstruction::SEL,

            (true, false, false, false, false, 0b0001 | 0b0011) if inst32.range_bits(15, 12) == 0b1111 => ArmInstruction::SMUAD, // Overlaps SMLAD
            (true, false, false, false, false, 0b0001 | 0b0011) => ArmInstruction::SMLAD,
            (true, false, true, false, false, 0b0001 | 0b0011) => ArmInstruction::SMLALD,
            (true, false, false, false, false, 0b0101 | 0b0111) if inst32.range_bits(15, 12) == 0b1111 => ArmInstruction::SMUSD, // Overlaps SMLSD
            (true, false, false, false, false, 0b0101 | 0b0111) => ArmInstruction::SMLSD,
            (true, false, true, false, false, 0b0101 | 0b0111) => ArmInstruction::SMLSLD,
            (true, false, true, false, true, 0b0001 | 0b0011) if inst32.range_bits(15, 12) == 0b1111 => ArmInstruction::SMMUL, // Overlaps SMMLA
            (true, false, true, false, true, 0b0001 | 0b0011) => ArmInstruction::SMMLA,
            (true, false, true, false, true, 0b1101 | 0b1111) => ArmInstruction::SMMLS,

            (false, true, false, true, _, 0b0001 | 0b0101 | 0b1001 | 0b1101) => ArmInstruction::SSAT,
            (false, true, false, true, false, 0b0011) => ArmInstruction::SSAT16,
            (false, true, true, true, _, 0b0001 | 0b0101 | 0b1001 | 0b1101) => ArmInstruction::USAT,
            (false, true, true, true, false, 0b0011) => ArmInstruction::USAT16,

            (false, true, false, true, false, 0b0111) if inst32.range_bits(19, 16) == 0b1111 => ArmInstruction::SXTB, // Overlaps SXTAB
            (false, true, false, true, false, 0b0111) => ArmInstruction::SXTAB,
            (false, true, false, false, false, 0b0111) if inst32.range_bits(19, 16) == 0b1111 => ArmInstruction::SXTB16, // Overlaps SXTAB16
            (false, true, false, false, false, 0b0111) => ArmInstruction::SXTAB16,
            (false, true, false, true, true, 0b0111) if inst32.range_bits(19, 16) == 0b1111 => ArmInstruction::SXTH, // Overlaps SXTAH
            (false, true, false, true, true, 0b0111) => ArmInstruction::SXTAH,
            (false, true, true, true, false, 0b0111) if inst32.range_bits(19, 16) == 0b1111 => ArmInstruction::UXTB, // Overlaps UXTAB
            (false, true, true, true, false, 0b0111) => ArmInstruction::UXTAB,
            (false, true, true, false, false, 0b0111) if inst32.range_bits(19, 16) == 0b1111 => ArmInstruction::UXTB16, // Overlaps UXTAB16
            (false, true, true, false, false, 0b0111) => ArmInstruction::UXTAB16,
            (false, true, true, true, true, 0b0111) if inst32.range_bits(19, 16) == 0b1111 => ArmInstruction::UXTH, // Overlaps UXTAH
            (false, true, true, true, true, 0b0111) => ArmInstruction::UXTAH,

            (true, true, false, false, false, 0b0001) if inst32.range_bits(15, 12) == 0b1111 => ArmInstruction::USAD8, // Overlaps USADA8
            (true, true, false, false, false, 0b0001) => ArmInstruction::USADA8,

            _ => ArmInstruction::Undefined
        }
    } else {
        decode_fullsize_load(cpu, inst32, true)
    }
}

#[inline]
fn decode_fullsize_load(cpu: &dyn ArmCpu, inst32: u32, i: bool) -> ArmInstruction {
    let p = inst32.test_bit(24);
    let u = inst32.test_bit(23);
    let b = inst32.test_bit(22);
    let w = inst32.test_bit(21);
    let l = inst32.test_bit(20);
    let cond = ArmCondition::from_inst32(inst32);
    
    match (p, b, w, l) {
        (false, false, true, true) => ArmInstruction::LDRT, // overlaps LDR
        (_, false, _, true) => ArmInstruction::LDR,
        (false, true, true, true) => ArmInstruction::LDRBT, // overlaps LDRB
        (_, true, _, true) => ArmInstruction::LDRB,
        (false, false, true, false) => ArmInstruction::STRT, // Overlaps STR
        (_, false, _, false) => ArmInstruction::STR,
        (false, true, true, false) => ArmInstruction::STRBT, // Overlaps STRB
        (_, true, _, false) => ArmInstruction::STRB,
    }
}

#[inline]
fn decode_fullsize_loadmultiple(cpu: &dyn ArmCpu, inst32: u32) -> ArmInstruction {
    let p = inst32.test_bit(24);
    let u = inst32.test_bit(23);
    let s = inst32.test_bit(22);
    let w = inst32.test_bit(21);
    let l = inst32.test_bit(20);
    let decode_lo = inst32.test_bit(15);
    let cond = ArmCondition::from_inst32(inst32);
    
    match (s, w, l, decode_lo) {
        (false, _, true, _) => ArmInstruction::LDM(LdmForm::Standard),
        (true, false, true, false) => ArmInstruction::LDM(LdmForm::UserRegisters),
        (true, _, true, true) => ArmInstruction::LDM(LdmForm::Restore),
        (false, _, false, _) => ArmInstruction::STM(StmForm::Standard),
        (true, false, false, _) => ArmInstruction::STM(StmForm::Standard),

        _ => ArmInstruction::Undefined
    }
}

#[inline]
fn decode_fullsize_branch(cpu: &dyn ArmCpu, inst32: u32) -> ArmInstruction {
    let link = inst32.test_bit(24);
    let offset = extract_imm_branch_offset(inst32);
    let cond = ArmCondition::from_inst32(inst32);
    
    match link {
        false => ArmInstruction::B{offset},
        true => ArmInstruction::BL{offset},
    }
}

#[inline]
fn decode_fullsize_loadcoprocessor(cpu: &dyn ArmCpu, inst32: u32) -> ArmInstruction {
    let p = inst32.test_bit(24);
    let u = inst32.test_bit(23);
    let n = inst32.test_bit(22);
    let w = inst32.test_bit(21);
    let l = inst32.test_bit(20);
    let in_extension_space = !(p || u || w);

    if in_extension_space {
        match (n, l) {
            (true, false) => ArmInstruction::MCRR,
            (true, true) => ArmInstruction::MRRC,
            
            _ => ArmInstruction::Undefined,
        }
    } else {
        match l {
            true => ArmInstruction::LDC,
            false => ArmInstruction::STC,
        }
    }
}

#[inline]
fn decode_fullsize_coprocessor(cpu: &dyn ArmCpu, inst32: u32) -> ArmInstruction {
    let decode_hi = inst32.test_bit(20);
    let decode_lo = inst32.test_bit(4);

    if inst32.test_bit(24) {
        ArmInstruction::Undefined
    } else {
        match (decode_hi, decode_lo) {
            (_, false) => ArmInstruction::CDP,
            (false, true) => ArmInstruction::MCR,
            (true, true) => ArmInstruction::MRC,
        }
    }
}

fn extract_imm_branch_offset(inst32: u32) -> u32 {
    inst32.range_bits(23, 0)
}



fn decode_thumb(cpu: &dyn ArmCpu, thumb16: u16) -> ArmInstruction {
    // TODO: Stub
    ArmInstruction::Undefined
}

enum ArmInstruction {
    Undefined,

    ADC,
    ADD,
    AND,
    B{offset: u32},
    BIC,
    BKPT,
    BL{offset: u32},
    BLX{offset: BlxVariant}, // Multiple encodings
    BX,
    BXJ,
    CDP,
    CDP2,
    CLZ,
    CMN,
    CMP,
    CPS,
    CPY,
    EOR,
    LDC,
    LDC2,
    LDM(LdmForm),
    LDR,
    LDRB,
    LDRD,
    LDRBT,
    LDREX,
    LDRH,
    LDRSB,
    LDRSH,
    LDRT,
    MCR,
    MCR2,
    MCRR,
    MCRR2,
    MLA,
    MOV,
    MRC,
    MRC2,
    MRRC,
    MRRC2,
    MRS,
    MSR,
    MUL,
    MVN,
    ORR,
    PKH(PkhForm),
    PLD,
    QADD,
    QADD16,
    QADD8,
    QADDSUBX,
    QDADD,
    QDSUB,
    QSUB,
    QSUB16,
    QSUB8,
    QSUBADDX,
    REV,
    REV16,
    REVSH,
    RFE,
    RSB,
    RSC,
    SADD16,
    SADD8,
    SADDSUBX,
    SBC,
    SEL,
    SETEND,
    SHADD16,
    SHADD8,
    SHADDSUBX,
    SHSUB16,
    SHSUB8,
    SHSUBADDX,
    SMLA(BitHalf, BitHalf),
    SMLAD,
    SMLAL(Option<(BitHalf, BitHalf)>),
    SMLALD,
    SMLAW(BitHalf),
    SMLSD,
    SMLSLD,
    SMMLA,
    SMMLS,
    SMMUL,
    SMUAD,
    SMULL,
    SMUL(BitHalf, BitHalf),
    SMULW(BitHalf),
    SMUSD,
    SRS,
    SSAT,
    SSAT16,
    SSUB16,
    SSUB8,
    SSUBADDX,
    STC,
    STC2,
    STM(StmForm),
    STR,
    STRB,
    STRBT,
    STRD,
    STREX,
    STRH,
    STRT,
    SUB,
    SWI,
    SWP,
    SWPB,
    SXTAB,
    SXTAB16,
    SXTAH,
    SXTB,
    SXTB16,
    SXTH,
    TEQ,
    TST,
    UADD16,
    UADD8,
    UADDSUBX,
    UHADD16,
    UHADD8,
    UHADDSUBX,
    UHSUB16,
    UHSUB8,
    UHSUBADDX,
    UMAAL,
    UMLAL,
    UMULL,
    UQADD16,
    UQADD8,
    UQADDSUBX,
    UQSUB16,
    UQSUB8,
    UQSUBADDX,
    USAD8,
    USADA8,
    USAT,
    USAT16,
    USUB16,
    USUB8,
    USUBADDX,
    UXTAB,
    UXTAB16,
    UXTAH,
    UXTB,
    UXTB16,
    UXTH,
}



enum BitHalf {
    Bottom,
    Top,
}

enum BlxVariant {
    Immediate(u32),
    Register,
}

enum PkhForm {
    BT,
    TB
}

enum LdmForm {
    Standard,
    UserRegisters,
    Restore
}

enum StmForm {
    Standard,
    UserRegisters,
}

enum ArmCondition {
    Equal,
    NotEqual,
    CarrySet,
    CarryClear,
    Minus,
    Plus,
    Overflow,
    NoOverflow,
    UnsignedHigher,
    UnsignedLowerOrSame,
    SignedGreaterOrEqual,
    SignedLessThan,
    SignedGreaterThan,
    SignedLessOrEqual,
    Always,
    SpecialUnconditional,
}

impl ArmCondition {
    fn from_inst32(inst32: u32) -> ArmCondition {
        match inst32.range_bits(31, 28) {
            0b0000 => ArmCondition::Equal,
            0b0001 => ArmCondition::NotEqual,
            0b0010 => ArmCondition::CarrySet,
            0b0011 => ArmCondition::CarryClear,
            0b0100 => ArmCondition::Minus,
            0b0101 => ArmCondition::Plus,
            0b0110 => ArmCondition::Overflow,
            0b0111 => ArmCondition::NoOverflow,
            0b1000 => ArmCondition::UnsignedHigher,
            0b1001 => ArmCondition::UnsignedLowerOrSame,
            0b1010 => ArmCondition::SignedGreaterOrEqual,
            0b1011 => ArmCondition::SignedLessThan,
            0b1100 => ArmCondition::SignedGreaterThan,
            0b1101 => ArmCondition::SignedLessOrEqual,
            0b1110 => ArmCondition::Always,
            0b1111 => ArmCondition::SpecialUnconditional,
            _ => unreachable!()
        }
    }

    fn is_special(&self) -> bool {
        matches!(self, ArmCondition::SpecialUnconditional)
    }
}

struct ProgramStatusRegister(u32);

impl ProgramStatusRegister {
    
}


    // ADC Yes Yes Yes Yes Yes
    // ADD Yes Yes Yes Yes Yes
    // AND Yes Yes Yes Yes Yes
    // B Yes Yes Yes Yes Yes
    // BIC Yes Yes Yes Yes Yes
    // BKPT No No Yes Yes Yes
    // BL Yes Yes Yes Yes Yes
    // BLX (both forms) No No Yes Yes Yes
    // BX No Yes Yes Yes Yes
    // BXJ No No No Only v5TEJ Yes
    // CDP Yes Yes Yes Yes Yes
    // CDP2 No No Yes Yes Yes
    // CLZ No No Yes Yes Yes
    // CMN Yes Yes Yes Yes Yes
    // CMP Yes Yes Yes Yes Yes
    // CPS No No No No Yes
    // CPY No No No No Yes
    // EOR Yes Yes Yes Yes Yes
    // LDC Yes Yes Yes Yes Yes
    // LDC2 No No Yes Yes Yes
    // LDM (all forms) Yes Yes Yes Yes Yes
    // LDR Yes Yes Yes Yes Yes
    // LDRB Yes Yes Yes Yes Yes
    // LDRD No No No Only v5TE, v5TEJ Yes
    // LDRBT Yes Yes Yes Yes Yes
    // LDREX No No No No Yes
    // LDRH Yes Yes Yes Yes Yes
    // LDRSB Yes Yes Yes Yes Yes
    // LDRSH Yes Yes Yes Yes Yes
    // LDRT Yes Yes Yes Yes Yes
    // MCR Yes Yes Yes Yes Yes
    // MCR2 No No Yes Yes Yes
    // MCRR No No No Only v5TE, v5TEJ Yes
    // MCRR2 No No No No Yes
    // MLA Yes Yes Yes Yes Yes
    // MOV Yes Yes Yes Yes Yes
    // MRC Yes Yes Yes Yes Yes
    // MRC2 No No Yes Yes Yes
    // MRRC No No No Only v5TE, v5TEJ Yes
    // MRRC2 No No No No Yes
    // MRS Yes Yes Yes Yes Yes
    // MSR Yes Yes Yes Yes Yes
    // MUL Yes Yes Yes Yes Yes
    // MVN Yes Yes Yes Yes Yes
    // ORR Yes Yes Yes Yes Yes
    // PKH (both forms) No No No No Yes
    // PLD No No No Only v5TE, v5TEJ Yes
    // QADD No No No Yes Yes
    // QADD16 No No No No Yes
    // QADD8 No No No No Yes
    // QADDSUBX No No No No Yes
    // QDADD No No No Yes Yes
    // QDSUB No No No Yes Yes
    // QSUB No No No Yes Yes
    // QSUB16 No No No No Yes
    // QSUB8 No No No No Yes
    // QSUBADDX No No No No Yes
    // REV (all forms) No No No No Yes
    // RFE No No No No Yes
    // RSB Yes Yes Yes Yes Yes
    // RSC Yes Yes Yes Yes Yes
    // SADD (all forms) No No No No Yes
    // SBC Yes Yes Yes Yes Yes
    // SEL No No No No Yes
    // SETEND No No No No Yes
    // SHADD (all forms) No No No No Yes
    // SHSUB (all forms) No No No No Yes
    // SMLAD No No No No Yes
    // SMLAL Yes Yes Yes Yes Yes
    // SMLALD No No No No Yes
    // SMLA<x, y> No No No Yes Yes
    // SMLAL<x, y> No No No Yes Yes
    // SMLAW<y> No No No Yes Yes
    // SMLSD No No No No Yes
    // SMLSLD No No No No Yes
    // SMMLA No No No No Yes
    // SMMLS No No No No Yes
    // SMMUL No No No No Yes
    // SMUAD No No No No Yes
    // SMULL Yes Yes Yes Yes Yes
    // SMUL<x, y> No No No Yes Yes
    // SMULW<y> No No No Yes Yes
    // SMUSD No No No No Yes
    // SRS No No No No Yes
    // SSAT (both forms) No No No No Yes
    // SSUB (all forms) No No No No Yes
    // STC Yes Yes Yes Yes Yes
    // STC2 No No Yes Yes Yes
    // STM (both forms) Yes Yes Yes Yes Yes
    // STR Yes Yes Yes Yes Yes
    // STRB Yes Yes Yes Yes Yes
    // STRBT Yes Yes Yes Yes Yes
    // STRD No No No Only v5TE, v5TEJ Yes
    // STREX No No No No Yes
    // STRH Yes Yes Yes Yes Yes
    // STRT Yes Yes Yes Yes Yes
    // SUB Yes Yes Yes Yes Yes
    // SWI Yes Yes Yes Yes Yes
    // SWP Yes Yes Yes Yes Deprecated
    // SWPB Yes Yes Yes Yes Deprecated
    // SXT (all forms) No No No No Yes
    // TEQ Yes Yes Yes Yes Yes
    // TST Yes Yes Yes Yes Yes
    // UADD (all forms) No No No No Yes
    // UHADD (all forms) No No No No Yes
    // UMAAL No No No No Yes
    // UMLAL Yes Yes Yes Yes Yes
    // UMULL Yes Yes Yes Yes Yes
    // UQADD (all forms) No No No No Yes
    // UQSUB (all forms) No No No No Yes
    // USAD (both forms) No No No No Yes
    // USAT (both forms) No No No No Yes
    // USUB (all forms) No No No No Yes
    // UXT (all forms) No No No No Yes



// ADC Yes Yes Yes
// ADD (all forms) Yes Yes Yes
// AND Yes Yes Yes
// ASR (both forms) Yes Yes Yes
// B (both forms) Yes Yes Yes
// BIC Yes Yes Yes
// BKPT No Yes Yes
// BL Yes Yes Yes
// BLX (both forms) No Yes Yes
// BX Yes Yes Yes
// CMN Yes Yes Yes
// CMP (all forms) Yes Yes Yes
// CPS No No Yes
// CPY No No Yes
// EOR Yes Yes Yes
// LDMIA Yes Yes Yes
// LDR (all forms) Yes Yes Yes
// LDRB (both forms) Yes Yes Yes
// LDRH (both forms) Yes Yes Yes
// LDRSB Yes Yes Yes
// LDRSH Yes Yes Yes
// LSL (both forms) Yes Yes Yes
// LSR (both forms) Yes Yes Yes
// MOV (all forms) Yes Yes Yes
// MUL Yes Yes Yes
// MVN Yes Yes Yes
// NEG Yes Yes Yes
// ORR Yes Yes Yes
// POP Yes Yes Yes
// PUSH Yes Yes Yes
// REV (all forms) No No Yes
// ROR Yes Yes Yes
// SBC Yes Yes Yes
// SETEND No No Yes
// STMIA Yes Yes Yes
// STR (all forms) Yes Yes Yes
// STRB (both forms) Yes Yes Yes
// STRH (both forms) Yes Yes Yes
// SUB (all forms) Yes Yes Yes
// SWI Yes Yes Yes
// SXTB/H No No Yes
// TST Yes Yes Yes
// UXTB/H No No Yes