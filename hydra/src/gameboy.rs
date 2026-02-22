mod apu;
mod cpu;
mod memory;
mod ppu;
mod serial;
mod timer;

use genawaiter::stack::let_gen_using;
use winit::{event::KeyEvent, keyboard::{KeyCode, PhysicalKey}};

use crate::{
    common::{
        bit::{BitVec, MaskedBitVec}, emulator::{EmuMessage, Emulator}, errors::HydraIOError
    },
    gameboy::{apu::Apu, cpu::Cpu, memory::{MMIO, MemoryMap, MemoryMappedIo, oam::Oam, rom::Rom, vram::Vram, wram::Wram}, ppu::{Ppu, PpuMode, colormap::ColorMap, lcdc::LcdController, state::PpuState}, timer::MasterTimer},
    window::HydraApp
};
use std::{
    cell::{Cell, RefCell}, ffi::OsStr, fs, path::Path, rc::Rc, sync::mpsc::{Receiver, Sender, channel}, thread, time::{Duration, Instant}
};

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Model {
    // A model with no revision specifies a target console (i.e. any revision).
    GameBoy(Option<GBRevision>),
    SuperGameBoy(Option<SGBRevision>),
    GameBoyColor(Option<CGBRevision>),
    GameBoyAdvance(Option<AGBRevision>),
}
impl Model {
    const fn as_str(&self) -> &'static str {
        match self {
            Model::GameBoy(Some(GBRevision::DMG0)) => "Game Boy (DMG0)",
            Model::GameBoy(Some(GBRevision::DMG)) => "Game Boy (DMG)",
            Model::GameBoy(Some(GBRevision::MGB)) => "Game Boy Pocket",
            Model::GameBoy(Some(GBRevision::CGBdmg)) => "Game Boy Color (DMG compat mode)",
            Model::GameBoy(Some(GBRevision::AGBdmg)) => "Game Boy Advance (DMG compat mode)",
            Model::GameBoy(None) => "Game Boy",
            Model::SuperGameBoy(Some(SGBRevision::SGB)) => "Super Game Boy",
            Model::SuperGameBoy(Some(SGBRevision::SGB2)) => "Super Game Boy 2",
            Model::SuperGameBoy(None) => "Super Game Boy",
            Model::GameBoyColor(Some(CGBRevision::CGB0)) => "Game Boy Color (CGB0)",
            Model::GameBoyColor(Some(CGBRevision::CGB)) => "Game Boy Color (CGB)",
            Model::GameBoyColor(None) => "Game Boy Color",
            Model::GameBoyAdvance(Some(AGBRevision::AGB0)) => "Game Boy Advance (AGB0)",
            Model::GameBoyAdvance(Some(AGBRevision::AGB)) => "Game Boy Advance (AGB)",
            Model::GameBoyAdvance(None) => "Game Boy Advance",
        }
    }

    const fn is_monochrome(&self) -> bool {
        matches!(self, Model::GameBoy(_) | Model::SuperGameBoy(_))
    }

    const fn is_color(&self) -> bool {
        matches!(self, Model::GameBoyColor(_) | Model::GameBoyAdvance(_))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GBRevision {
    DMG0,
    DMG,
    MGB,
    CGBdmg, // Special mode to specify Game Boy Color running original GB game in compatibility mode.
    AGBdmg, // Special mode to specify Game Boy Advance running original GB game in compatibility mode.
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SGBRevision {
    SGB,
    SGB2,
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CGBRevision {
    CGB0,
    CGB,
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AGBRevision {
    AGB0,
    AGB,
}

pub struct GameBoy {
    apu: Rc<RefCell<Apu>>,
    cpu: Option<Cpu>,
    memory: Rc<RefCell<MemoryMap>>,
    ppu: Option<Ppu>,
    clock: Rc<RefCell<MasterTimer>>,

    joypad: Rc<RefCell<Joypad>>,

    channel: Receiver<EmuMessage>,
}

impl GameBoy {
    fn from_revision(path: &Path, model: Model, app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> {
        let rom = Rom::from_vec(fs::read(path)?)?;
        let (send, recv) = channel();
        let graphics = app.clone_graphics();
        let audio = app.clone_audio();
        let proxy = app.clone_proxy();

        // Build Game Boy on a new thread
        thread::spawn(move || {
            let model = Rc::new(model);
            let interrupt_flags = Rc::new(RefCell::new(InterruptFlags::new()));
            let interrupt_enable = Rc::new(RefCell::new(InterruptEnable::new()));
            let joypad = Rc::new(RefCell::new(Joypad::new(interrupt_flags.clone())));
            let ppu_mode = Rc::new(RefCell::new(PpuMode::OAMScan));
            let lcd_controller = Rc::new(RefCell::new(LcdController::new()));
            let vram = Rc::new(RefCell::new(Vram::new(model.clone(), ppu_mode.clone(), lcd_controller.clone())));
            let wram = Rc::new(RefCell::new(Wram::new(model.clone())));
            let ppu_state = Rc::new(RefCell::new(PpuState::new(&model, ppu_mode.clone(), interrupt_flags.clone())));
            let apu = Rc::new(RefCell::new(Apu::new(audio)));
            let clock = Rc::new(RefCell::new(MasterTimer::new(model.clone(), apu.clone(), ppu_state.clone(), interrupt_flags.clone())));
            let cpu = Some(cpu::Cpu::new(&rom, &model, interrupt_flags.clone(), interrupt_enable.clone(), joypad.clone(), clock.clone()));
            let scy = Rc::new(Cell::new(0x00));
            let scx = Rc::new(Cell::new(0x00));
            let wy = Rc::new(Cell::new(0x00));
            let wx = Rc::new(Cell::new(0x00));
            let color_map = Rc::new(RefCell::new(ColorMap::new(&model)));
            let oam = Rc::new(RefCell::new(Oam::new(model.clone())));
            let ppu = Some(ppu::Ppu::new(model.clone(), vram.clone(), oam.clone(), lcd_controller.clone(), ppu_state.clone(), clock.clone(), interrupt_flags.clone(), scy.clone(), scx.clone(), wy.clone(), wx.clone(), color_map.clone(), graphics, proxy));
            let memory = Rc::new(RefCell::new(memory::MemoryMap::new(&model, rom, vram, wram, oam, joypad.clone(), clock.clone(), interrupt_flags.clone(), apu.clone(), lcd_controller.clone(), ppu_state.clone(), scy.clone(), scx.clone(), color_map.clone(), wy.clone(), wx.clone(), interrupt_enable.clone()).unwrap())); // TODO: Error should be handled rather than unwrapped
            GameBoy {
                apu,
                cpu,
                memory,
                ppu,
                clock,
                joypad,
                channel: recv,
            }.main_thread();
        });
        Ok(send)
    }
    pub fn from_model(path: &Path, model: Model, app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> {
        // If file extension is valid for the given model, initialize the emulator
        // Otherwise, return an InvalidExtension error
        let model = match (path.extension().and_then(OsStr::to_str), model) {
            (Some("gb") | Some("gbc"), Model::GameBoy(revision)) => Model::GameBoy(Some(revision.unwrap_or(app.get_config().gb.default_models.dmg))),
            (Some("gb") | Some("gbc"), Model::SuperGameBoy(revision)) => Model::SuperGameBoy(Some(revision.unwrap_or(app.get_config().gb.default_models.sgb))),
            (Some("gb") | Some("gbc"), Model::GameBoyColor(revision)) => Model::GameBoyColor(Some(revision.unwrap_or(app.get_config().gb.default_models.cgb))),
            (Some("gb") | Some("gbc") | Some("gba"), Model::GameBoyAdvance(revision)) => Model::GameBoyAdvance(Some(revision.unwrap_or(app.get_config().gb.default_models.agb))),
            (ext, model) => return Err(HydraIOError::InvalidEmulator(model.as_str(), ext.map(str::to_string))),
        };

        Self::from_revision(path, model, app)
    }
    fn dump_mem(&self) {
        for y in 0..=0xFFF {
            print!("{:#06X}:   ", y << 4);
            for x in 0..=0xF {
                print!("{:02X} ", self.memory.borrow().read_u8(x | (y << 4), true));
            }
            println!("");
        }
    }
}

impl Emulator for GameBoy {
    fn main_thread(mut self) {
        println!("Launching Wyrm");

        // Generate Coroutines
        let mut cpu = self.cpu.take().unwrap();
        let mut ppu = self.ppu.take().unwrap();
        let_gen_using!(cpu_coro, |co| cpu.coro(self.memory.clone(), co, false));
        let_gen_using!(ppu_coro, |co| ppu.coro(co));

        // Main loop
        'main: loop {
            self.clock.borrow_mut().tick();
            self.apu.borrow_mut().dot_tick();
            self.memory.borrow_mut().tick_dma();
            if self.clock.borrow().is_system_cycle() {
                cpu_coro.resume();
            }
            self.clock.borrow_mut().refresh_tima_if_overflowing();
            ppu_coro.resume();

            // Every frame
            if self.clock.borrow().get_ppu_dots() == 0 {

                // Send audio for playback
                self.apu.borrow_mut().frame();

                // Process any new messages
                for msg in self.channel.try_iter() {
                    match msg {
                        // TODO: Allow remapping controls in the future
                        EmuMessage::KeyboardInput(KeyEvent {state, physical_key: PhysicalKey::Code(keycode), .. }) => match keycode {
                            KeyCode::KeyW => self.joypad.borrow_mut().press_dpad(Dpad::Up, state.is_pressed()),
                            KeyCode::KeyS => self.joypad.borrow_mut().press_dpad(Dpad::Down, state.is_pressed()),
                            KeyCode::KeyA => self.joypad.borrow_mut().press_dpad(Dpad::Left, state.is_pressed()),
                            KeyCode::KeyD => self.joypad.borrow_mut().press_dpad(Dpad::Right, state.is_pressed()),
                            KeyCode::KeyK => self.joypad.borrow_mut().press_button(Button::A, state.is_pressed()),
                            KeyCode::KeyJ => self.joypad.borrow_mut().press_button(Button::B, state.is_pressed()),
                            KeyCode::Enter => self.joypad.borrow_mut().press_button(Button::Start, state.is_pressed()),
                            KeyCode::ShiftRight => self.joypad.borrow_mut().press_button(Button::Select, state.is_pressed()),
                            _ => {}
                        }
                        EmuMessage::HotSwap(path) => {
                            if let Err(e) = self.memory.borrow_mut().hot_swap_rom(path) {
                                println!("{}", e);
                            }
                        },
                        EmuMessage::Stop => break 'main,
                        _ => {} // Do nothing
                    }
                }
            }
        }

        println!("Exiting Wyrm");

        // Dump memory (for debugging)
        self.dump_mem();
    }
}

pub struct Joypad {
    button_vector: u8,
    dpad_vector: u8,
    joyp: MaskedBitVec<u8, true>,
    interrupt_flags: Rc<RefCell<InterruptFlags>>
}

impl Joypad {
    pub fn new(interrupt_flags: Rc<RefCell<InterruptFlags>>) -> Self {
        Joypad { 
            button_vector: 0b0000,
            dpad_vector: 0b0000,
            joyp: MaskedBitVec::new(0xCF, 0b00111111, 0b00110000),
            interrupt_flags 
        }
    }

    fn is_polling_buttons(&self) -> bool {
        !self.joyp.test_bit(5)
    }
    
    fn is_polling_dpad(&self) -> bool {
        !self.joyp.test_bit(4)
    }

    fn refresh(&mut self) {
        let mut after = 0b0000;
        if self.is_polling_buttons() {after |= self.button_vector}
        if self.is_polling_dpad() {after |= self.dpad_vector}
        
        if *self.joyp & after != 0 {
            self.interrupt_flags.borrow_mut().request(Interrupt::Joypad);
        }

        *self.joyp = (*self.joyp & 0b00110000) | (after ^ 0b1111);
    }

    pub(self) fn press_button(&mut self, button: Button, is_pressed: bool) {
        self.button_vector.map_bits(button as u8, is_pressed);
        self.refresh();
    }

    pub(self) fn press_dpad(&mut self, dpad: Dpad, is_pressed: bool) {
        self.dpad_vector.map_bits(dpad as u8, is_pressed);
        self.refresh();
    }

    pub fn is_input_active(&self) -> bool {
        !self.joyp.read() & 0xF != 0
    }
}

impl MemoryMappedIo<{MMIO::JOYP as u16}> for Joypad {   
    fn read(&self) -> u8 {
        self.joyp.read()
    }

    fn write(&mut self, val: u8) {
        self.joyp.write(val);
        self.refresh();
    }
}

#[repr(u8)]
enum Dpad {
    Right = 0b00000001,
    Left  = 0b00000010,
    Up    = 0b00000100,
    Down  = 0b00001000,
}

#[repr(u8)]
enum Button {
    A      = 0b00000001,
    B      = 0b00000010,
    Select = 0b00000100,
    Start  = 0b00001000,
}

pub struct InterruptFlags {
    interrupts: MaskedBitVec<u8, true>
}

impl InterruptFlags {
    pub fn new() -> Self {
        InterruptFlags {
            interrupts: MaskedBitVec::new(0b11100001, 0b00011111, 0b00011111)
        }
    }

    pub fn request(&mut self, interrupt: Interrupt) {
        *self.interrupts |= interrupt as u8
    }

    pub fn is_requested(&self, interrupt: Interrupt) -> bool {
        *self.interrupts & interrupt as u8 != 0
    }

    pub fn get_inner(&mut self) -> &mut MaskedBitVec<u8, true> {
        &mut self.interrupts
    }
}

impl MemoryMappedIo<{MMIO::IF as u16}> for InterruptFlags{
    fn read(&self) -> u8 {
        self.interrupts.read()
    }

    fn write(&mut self, val: u8) {
        self.interrupts.write(val);
    }
}

pub struct InterruptEnable {
    interrupts: MaskedBitVec<u8, false> // TODO: false assumed due to startup value -- verify this
}

impl InterruptEnable {
    pub fn new() -> Self {
        InterruptEnable {
            interrupts: MaskedBitVec::new(0b00000000, 0b00011111, 0b00011111)
        }
    }
}

impl MemoryMappedIo<{MMIO::IE as u16}> for InterruptEnable {
    fn read(&self) -> u8 {
        self.interrupts.read()
    }

    fn write(&mut self, val: u8) {
        self.interrupts.write(val)
    }
}

#[repr(u8)]
pub enum Interrupt {
    Vblank = 0b00000001,
    Stat   = 0b00000010,
    Timer  = 0b00000100,
    Serial = 0b00001000,
    Joypad = 0b00010000,
}