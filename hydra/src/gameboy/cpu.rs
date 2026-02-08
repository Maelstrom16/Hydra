mod ime;
mod opcode;

use std::{cell::RefCell, rc::Rc};

use futures::FutureExt;
use genawaiter::stack::Co;

use crate::{
    gameboy::{
        AGBRevision, CGBRevision, GBRevision, Model, SGBRevision,
        cpu::{ime::InterruptHandler, opcode::{CondOperand, ConstOperand16, IntOperand, LocalOpcodeFn}},
        memory::{
            MemoryMap,
            io::{self, IOMap, deserialized::{RegIe, RegIf}}, rom::Rom,
        },
    },
    gen_all,
};

/// A Game Boy CPU.
///
/// Note: Registers are stored in little-endian byte arrays, so the representation in code may be misleading.
/// The AF register, for example, is indexed as follows:
/// ```
/// let af: u16 = u16::from_le_bytes(cpu.af);
/// let a: u8 = cpu.af[1];
/// let f: u8 = cpu.af[0];
/// let z: bool = (cpu.af[0] & 0b10000000) != 0;
/// let n: bool = (cpu.af[0] & 0b01000000) != 0;
/// let h: bool = (cpu.af[0] & 0b00100000) != 0;
/// let c: bool = (cpu.af[0] & 0b00010000) != 0;
///
/// cpu.af[0] &= 0b01111111; // Reset zero flag
/// cpu.af[0] |= 0b00010000; // Set carry flag
/// cpu.af[0] = ((true as u8) << 5) | (cpu.af[0] & 0b11011111) // Set/reset half-carry flag based on bool
/// ```
pub struct CPU {
    af: [u8; 2],
    bc: [u8; 2],
    de: [u8; 2],
    hl: [u8; 2],
    sp: u16,
    pc: u16,
    ir: u8,
    r#if: RegIf,
    ie: RegIe,
    interrupt_handler: InterruptHandler,
}

pub enum Register8 {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

impl CPU {
    pub fn new(rom: &Rom, io: &IOMap, model: Model) -> Self {
        let af;
        let bc;
        let de;
        let hl;
        let sp: u16 = 0xFFFE;
        let pc: u16 = 0x0100;
        let ir: u8 = 0x00;
        let r#if = RegIf::wrap(io.clone_pointer(io::MMIO::IF));
        let ie = RegIe::wrap(io.clone_pointer(io::MMIO::IE));
        let interrupt_handler = InterruptHandler::default();
        match model {
            Model::GameBoy(Some(GBRevision::DMG0)) => {
                af = [0b0000 << 4, 0x01];
                bc = [0x13, 0xFF];
                de = [0xC1, 0x00];
                hl = [0x03, 0x84];
            }
            Model::GameBoy(Some(GBRevision::DMG)) => {
                af = [if rom.get_header_checksum() == 0 { 0b1000 << 4 } else { 0b1011 << 4 }, 0x01];
                bc = [0x13, 0x00];
                de = [0xD8, 0x00];
                hl = [0x4D, 0x01];
            }
            Model::GameBoy(Some(GBRevision::MGB)) => {
                af = [if rom.get_header_checksum() == 0 { 0b1000 << 4 } else { 0b1011 << 4 }, 0xFF];
                bc = [0x13, 0x00];
                de = [0xD8, 0x00];
                hl = [0x4D, 0x01];
            }
            Model::SuperGameBoy(Some(SGBRevision::SGB)) => {
                af = [0b0000 << 4, 0x01];
                bc = [0x14, 0x00];
                de = [0x00, 0x00];
                hl = [0x60, 0xC0];
            }
            Model::SuperGameBoy(Some(SGBRevision::SGB2)) => {
                af = [0b0000 << 4, 0xFF];
                bc = [0x14, 0x00];
                de = [0x00, 0x00];
                hl = [0x60, 0xC0];
            }
            Model::GameBoy(Some(GBRevision::CGBdmg)) => {
                let mut b = 0x00;
                let mut hl_bytes = [0x7C, 0x00];
                if rom.has_publisher_rnd1() {
                    // If either licensee code is 0x01, B = sum of all title bytes
                    b = rom.get_title().iter().sum();
                    if b == 0x43 || b == 0x58 {
                        // And, check special cases for HL
                        hl_bytes = [0x1A, 0x99];
                    }
                }
                af = [0b1000 << 4, 0x11];
                bc = [0x00, b];
                de = [0x08, 0x00];
                hl = hl_bytes;
            }
            Model::GameBoy(Some(GBRevision::AGBdmg)) => {
                let mut b = 0x01;
                let mut hl_bytes = [0x7C, 0x00];
                let mut f = 0b00000000;
                if rom.has_publisher_rnd1() {
                    // If either licensee code is 0x01, B = sum of all title bytes
                    b = rom.get_title().iter().sum();
                    if b & 0b1111 == 0 {
                        // Last op is an INC; set h flag...
                        f |= 0b0010 << 4;
                        if b == 0 {
                            // ...and z flag if necessary
                            f |= 0b1000 << 4
                        }
                    } else if b == 0x44 || b == 0x59 {
                        // Otherwise, still check special cases for HL
                        hl_bytes = [0x1A, 0x99];
                    }
                }
                af = [f, 0x11];
                bc = [0x00, b];
                de = [0x08, 0x00];
                hl = hl_bytes;
            }
            Model::GameBoyColor(Some(CGBRevision::CGB0 | CGBRevision::CGB)) => {
                af = [0b1000 << 4, 0x11];
                bc = [0x00, 0x00];
                de = [0x56, 0xFF];
                hl = [0x0D, 0x00];
            }
            Model::GameBoyAdvance(Some(AGBRevision::AGB0 | AGBRevision::AGB)) => {
                af = [0b0000 << 4, 0x11];
                bc = [0x00, 0x01];
                de = [0x56, 0xFF];
                hl = [0x0D, 0x00];
            }
            _ => panic!("Attempt to initialize Game Boy CPU without a proper revision"),
        }
        CPU {
            af,
            bc,
            de,
            hl,
            sp,
            pc,
            ir,
            r#if,
            ie,
            interrupt_handler,
        }
    }

    fn fetch_interrupt(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, jump_addr: u16) -> impl Future<Output = ()> {
        // TODO: Verify cycle accuracy
        self.call(&memory, co, CondOperand::Unconditional, ConstOperand16(jump_addr))
    }

    async fn fetch_opcode(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, debug: bool) {
        self.ir = gen_all!(&co, |co_inner| self.step_u8(memory, co_inner));
        if debug {println!(
            "{:#06X}: {:02X}  ---  A: {:#04X}   F: {:08b}   BC: {:#06X}   DE: {:#06X}   HL: {:#06X}   SP: {:#06X}",
            self.pc - 1,
            self.ir,
            self.af[1],
            self.af[0],
            u16::from_le_bytes(self.bc),
            u16::from_le_bytes(self.de),
            u16::from_le_bytes(self.hl),
            self.sp
        )}
        gen_all!(&co, |co_inner| opcode::OP_TABLE[self.ir as usize](self, memory, co_inner));
    }

    #[inline(always)]
    fn fetch<'a>(&mut self, debug: bool) -> LocalOpcodeFn<'a> {
        if self.interrupt_handler.ime && self.interrupt_pending() { 
            // Generate call instruction from handled interrupt
            let composite = self.ie.get_entire() & self.r#if.get_entire();
            for shift_width in 0..=4 {
                let bitmask = 1 << shift_width;
                if composite & bitmask == 0 {
                    // Check next priority interrupt
                    continue;
                } else {
                    // Handle interrupt and return call instruction
                    self.interrupt_handler.ime = false;
                    self.r#if.set_entire(self.r#if.get_entire() ^ bitmask);
                    let jump_addr = (shift_width * 0x8) + 0x40;
                    
                    return Box::new(move |cpu_inner: &'a mut CPU, memory_inner, co_inner| cpu_inner.fetch_interrupt(memory_inner, co_inner, jump_addr).boxed_local())
                        as LocalOpcodeFn<'a>;
                }
            }
            unreachable!() // Interrupt should've been handled
        } else {
            // Fetch instruction from memory
            return Box::new(move |cpu_inner: &'a mut CPU, memory_inner, co_inner| cpu_inner.fetch_opcode(memory_inner, co_inner, debug).boxed_local())
                as LocalOpcodeFn<'a>;
        }
    }

    #[inline(always)]
    async fn step_u8(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) -> u8 {
        let result = memory.borrow().read_u8(self.pc);
        self.pc += 1;
        co.yield_(()).await;
        result
    }

    #[inline(always)]
    async fn read_u8(&self, address: u16, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) -> u8 {
        co.yield_(()).await;
        memory.borrow().read_u8(address)
    }

    #[inline(always)]
    async fn write_u8(&self, address: u16, value: u8, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) -> () {
        co.yield_(()).await;
        memory.borrow_mut().write_u8(value, address);
    }

    #[inline(always)]
    fn interrupt_pending(&self) -> bool {
        self.ie.get_entire() & self.r#if.get_entire() != 0
    }

    pub async fn coro(&mut self, memory: Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, debug: bool) {
        loop {
            // Skip this iteration if halted and there are no interrupts pending
            if self.interrupt_handler.halted {
                if self.interrupt_pending() {
                    self.interrupt_handler.halted = false;
                } else {
                    co.yield_(()).await;
                    continue;  
                }
            }

            // Fetch cycle
            let pc = self.pc;
            let next = self.fetch(debug);

            // Execute cycle(s)
            gen_all!(&co, |co_inner| next(self, &memory, co_inner));
            self.interrupt_handler.refresh(&mut self.pc, pc);
        }
    }
}

// Opcode Helpers
macro_rules! _offset {
    (z) => {
        7
    };
    (n) => {
        6
    };
    (h) => {
        5
    };
    (c) => {
        4
    };
}
macro_rules! _inverse_mask {
    (z) => {
        0b01111111
    };
    (n) => {
        0b10111111
    };
    (h) => {
        0b11011111
    };
    (c) => {
        0b11101111
    };
}
macro_rules! set_flags {
    ($cpu:expr; $($key:ident=$val:expr),* $(,)?) => {
        $(
            $cpu.af[0] = ($cpu.af[0] & _inverse_mask!($key)) | (($val as u8) << _offset!($key));
        )*
    };
}
macro_rules! _mask {
    (z) => {
        0b10000000
    };
    (n) => {
        0b01000000
    };
    (h) => {
        0b00100000
    };
    (c) => {
        0b00010000
    };
}
macro_rules! get_flag {
    ($cpu:expr; $flag:ident) => {
        $cpu.af[0] & _mask!($flag) != 0
    };
}

impl CPU {
    #[inline(always)]
    async fn ld<T, O1: IntOperand<T>, O2: IntOperand<T>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, dest: O1, src: O2) {
        let value = gen_all!(co, |co_inner| src.get(self, memory, co_inner));
        gen_all!(co, |co_inner| dest.set(value, self, memory, co_inner));
    }
    #[inline(always)]
    async fn ld_hlspe(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        let e = gen_all!(&co, |co_inner| self.step_u8(memory, co_inner)) as i8;
        let result = self.sp.wrapping_add_signed(e.into());
        let lsb = (self.sp & 0xFF) as u8;
        let (_, carry) = lsb.overflowing_add_signed(e);
        let lsb_half = if e.signum() == 1 { lsb | 0xF0 } else { lsb & 0x0F };
        let (_, half_carry) = lsb_half.overflowing_add_signed(e);
        set_flags!(self;
            z=(false),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        co.yield_(()).await;
        self.hl = u16::to_le_bytes(result);
    }

    #[inline(always)]
    async fn inc<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        let (result, _) = o.overflowing_add(1);
        let (_, half_carry) = (o | 0xF0).overflowing_add(1);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry)
        );
        gen_all!(co, |co_inner| operand.set(result, self, memory, co_inner));
    }
    #[inline(always)]
    async fn inc16<O: IntOperand<u16>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        let result = o.wrapping_add(1);
        co.yield_(()).await;
        gen_all!(co, |co_inner| operand.set(result, self, memory, co_inner));
    }

    #[inline(always)]
    async fn dec<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        let (result, _) = o.overflowing_sub(1);
        let (_, half_carry) = (o & 0x0F).overflowing_sub(1);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry)
        );
        gen_all!(co, |co_inner| operand.set(result, self, memory, co_inner));
    }
    #[inline(always)]
    async fn dec16<O: IntOperand<u16>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        let result = o.wrapping_sub(1);
        co.yield_(()).await;
        gen_all!(co, |co_inner| operand.set(result, self, memory, co_inner));
    }

    #[inline(always)]
    async fn add<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let (a, operand) = (self.af[1], gen_all!(co, |co_inner| operand.get(self, memory, co_inner)));
        let (result, carry) = a.overflowing_add(operand);
        let (_, half_carry) = (a | 0xF0).overflowing_add(operand);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        self.af[1] = result;
    }
    #[inline(always)]
    async fn add_hl<O: IntOperand<u16>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let (hl, operand) = (u16::from_le_bytes(self.hl), gen_all!(co, |co_inner| operand.get(self, memory, co_inner)));
        let result = hl.wrapping_add(operand);
        let [_, h] = self.hl;
        let [_, oph] = u16::to_le_bytes(operand);
        let (_, carry) = h.overflowing_add(oph);
        let (_, half_carry) = (h | 0xF0).overflowing_add(oph);
        set_flags!(self;
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        self.hl = u16::to_le_bytes(result);
    }
    #[inline(always)]
    async fn add_spe(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        let e = gen_all!(&co, |co_inner| self.step_u8(memory, co_inner)) as i8;
        let result = self.sp.wrapping_add_signed(e.into());
        let lsb = (self.sp & 0xFF) as u8;
        let (_, carry) = lsb.overflowing_add_signed(e);
        let lsb_half = if e.signum() == 1 { lsb | 0xF0 } else { lsb & 0x0F };
        let (_, half_carry) = lsb_half.overflowing_add_signed(e);
        set_flags!(self;
            z=(false),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        co.yield_(()).await;
        co.yield_(()).await;
        self.sp = result;
    }

    #[inline(always)]
    async fn adc<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let (a, operand) = (self.af[1], gen_all!(co, |co_inner| operand.get(self, memory, co_inner)) + get_flag!(self; c) as u8);
        let (result, carry) = a.overflowing_add(operand);
        let (_, half_carry) = (a | 0xF0).overflowing_add(operand);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        self.af[1] = result;
    }

    #[inline(always)]
    async fn sub<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let (a, operand) = (self.af[1], gen_all!(co, |co_inner| operand.get(self, memory, co_inner)));
        let (result, carry) = a.overflowing_sub(operand);
        let (_, half_carry) = (a & 0x0F).overflowing_sub(operand);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        self.af[1] = result;
    }

    #[inline(always)]
    async fn sbc<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let (a, operand) = (self.af[1], gen_all!(co, |co_inner| operand.get(self, memory, co_inner)) + get_flag!(self; c) as u8);
        let (result, carry) = a.overflowing_sub(operand);
        let (_, half_carry) = (a & 0x0F).overflowing_sub(operand);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        self.af[1] = result;
    }

    #[inline(always)]
    async fn and<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let result = self.af[1] & gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(true),
            c=(false)
        );
        self.af[1] = result;
    }

    #[inline(always)]
    async fn or<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let result = self.af[1] | gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(false)
        );
        self.af[1] = result;
    }

    #[inline(always)]
    async fn xor<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let result = self.af[1] ^ gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(false)
        );
        self.af[1] = result;
    }

    #[inline(always)]
    async fn cp<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let (a, operand) = (self.af[1], gen_all!(co, |co_inner| operand.get(self, memory, co_inner)));
        let (result, carry) = a.overflowing_sub(operand);
        let (_, half_carry) = (a & 0x0F).overflowing_sub(operand);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
    }

    #[inline(always)]
    async fn push<O: IntOperand<u16>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let bytes = u16::to_le_bytes(gen_all!(co, |co_inner| operand.get(self, memory, co_inner)));
        co.yield_(()).await;
        self.sp -= 1;
        gen_all!(&co, |co_inner| self.write_u8(self.sp, bytes[1], memory, co_inner));
        self.sp -= 1;
        gen_all!(&co, |co_inner| self.write_u8(self.sp, bytes[0], memory, co_inner));
    }

    #[inline(always)]
    async fn pop<O: IntOperand<u16>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let mut bytes = [0; 2];
        bytes[0] = gen_all!(&co, |co_inner| self.read_u8(self.sp, memory, co_inner));
        self.sp += 1;
        bytes[1] = gen_all!(&co, |co_inner| self.read_u8(self.sp, memory, co_inner));
        self.sp += 1;
        gen_all!(co, |co_inner| operand.set(u16::from_le_bytes(bytes), self, memory, co_inner));
    }

    #[inline(always)]
    async fn ccf(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        set_flags!(self;
            n=(false),
            h=(false)
        );
        self.af[0] ^= _mask!(c);
    }

    #[inline(always)]
    async fn scf(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        set_flags!(self;
            n=(false),
            h=(false),
            c=(true)
        );
    }

    #[inline(always)]
    async fn daa(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        todo!() //TODO
    }

    #[inline(always)]
    async fn cpl(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        self.af[1] ^= 0xFF;
        set_flags!(self;
            n=(true),
            h=(true)
        );
    }

    #[inline(always)]
    async fn rlc<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        let result = o.rotate_left(1);
        set_flags!(self;
            z=(false),
            n=(false),
            h=(false),
            c=(result & 0b00000001 != 0)
        );
        gen_all!(co, |co_inner| operand.set(result, self, memory, co_inner));
    }

    #[inline(always)]
    async fn rrc<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        let result = o.rotate_right(1);
        set_flags!(self;
            z=(false),
            n=(false),
            h=(false),
            c=(result & 0b10000000 != 0)
        );
        gen_all!(co, |co_inner| operand.set(result, self, memory, co_inner));
    }

    #[inline(always)]
    async fn rl<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        let (result, carry) = o.overflowing_shl(1);
        set_flags!(self;
            z=(false),
            n=(false),
            h=(false),
            c=(carry)
        );
        gen_all!(co, |co_inner| operand.set(result | get_flag!(self; c) as u8, self, memory, co_inner));
    }

    #[inline(always)]
    async fn rr<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        let (result, carry) = o.overflowing_shr(1);
        set_flags!(self;
            z=(false),
            n=(false),
            h=(false),
            c=(carry)
        );
        gen_all!(co, |co_inner| operand.set(result | (get_flag!(self; c) as u8) << 7, self, memory, co_inner));
    }

    #[inline(always)]
    async fn sla<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        let (result, carry) = o.overflowing_shl(1);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(carry)
        );
        gen_all!(co, |co_inner| operand.set(result, self, memory, co_inner));
    }

    #[inline(always)]
    async fn sra<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner)) as i8;
        let (result, carry) = o.overflowing_shr(1);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(carry)
        );
        gen_all!(co, |co_inner| operand.set(result as u8, self, memory, co_inner));
    }

    #[inline(always)]
    async fn swap<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        let result = (o & 0x0F) << 4 | (o & 0xF0) >> 4;
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(false)
        );
        gen_all!(co, |co_inner| operand.set(result, self, memory, co_inner));
    }

    #[inline(always)]
    async fn srl<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        let (result, carry) = o.overflowing_shr(1);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(carry)
        );
        gen_all!(co, |co_inner| operand.set(result, self, memory, co_inner));
    }

    #[inline(always)]
    async fn bit<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, index: u8, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        set_flags!(self;
            z=(o & (1 << index) != 0),
            n=(false),
            h=(true),
        );
    }

    #[inline(always)]
    async fn res<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, index: u8, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        gen_all!(co, |co_inner| operand.set(o & ((1 << index) ^ 0b11111111), self, memory, co_inner));
    }

    #[inline(always)]
    async fn set<O: IntOperand<u8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, index: u8, operand: O) {
        let o = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        gen_all!(co, |co_inner| operand.set(o | (1 << index), self, memory, co_inner));
    }

    #[inline(always)]
    async fn jp<O: IntOperand<u16>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, condition: CondOperand, operand: O) {
        let addr = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        if condition.evaluate(self) {
            co.yield_(()).await;
            self.pc = addr;
        }
    }

    #[inline(always)]
    async fn jr<O: IntOperand<i8>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, condition: CondOperand, operand: O) {
        let e = gen_all!(co, |co_inner| operand.get(self, memory, co_inner)) as i8;
        let addr = self.pc.wrapping_add_signed(e.into());
        if condition.evaluate(self) {
            co.yield_(()).await;
            self.pc = addr;
        }
    }

    #[inline(always)]
    async fn call<O: IntOperand<u16>>(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, condition: CondOperand, operand: O) {
        let addr = gen_all!(co, |co_inner| operand.get(self, memory, co_inner));
        if condition.evaluate(self) {
            gen_all!(&co, |co_inner| self.push(memory, co_inner, opcode::RegisterOperand16(Register16::PC)));
            self.pc = addr;
        }
    }

    #[inline(always)]
    async fn ret(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>, condition: CondOperand) {
        co.yield_(()).await;
        if condition.evaluate(self) {
            gen_all!(&co, |co_inner| self.pop(memory, co_inner, opcode::RegisterOperand16(Register16::PC)));
            co.yield_(()).await;
        }
    }

    #[inline(always)]
    async fn reti(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        gen_all!(&co, |co_inner| self.ret(memory, co_inner, CondOperand::Unconditional));
        self.interrupt_handler.ime = true;
    }

    #[inline(always)]
    async fn ei(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        self.interrupt_handler.queue_ime();
    }

    #[inline(always)]
    async fn di(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        self.interrupt_handler.ime = false;
        self.interrupt_handler.cancel_ime();
    }

    #[inline(always)]
    async fn halt(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        self.interrupt_handler.halted = true;

        // If IME is off but an interrupt is already pending, trigger halt bug
        if !self.interrupt_handler.ime && self.interrupt_pending() {
            self.interrupt_handler.queue_halt_bug();
        }
    }

    #[inline(always)]
    async fn stop(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        todo!() //TODO
    }

    #[inline(always)]
    async fn inavlidop(&mut self, memory: &Rc<RefCell<MemoryMap>>, co: Co<'_, ()>) {
        panic!("Unknown opcode")
    }
}
