mod cpu;
mod memory;
mod ppu;

use genawaiter::stack::let_gen_using;
use winit::{event::ElementState, event_loop::EventLoopProxy, keyboard::{KeyCode, PhysicalKey}, window::Window};

use crate::{
    common::{
        emulator::{EmuMessage, Emulator},
        errors::HydraIOError,
    },
    config::Config,
    gameboy::memory::{io::{IOMap, MMIO, deserialized::{RegIf, RegP1}}, rom::Rom},
    graphics::Graphics, window::{HydraApp, UserEvent},
};
use std::{
    cell::{Cell, RefCell}, ffi::OsStr, fs, path::Path, rc::Rc, sync::{
        Arc, RwLock,
        mpsc::{Receiver, Sender, channel},
    }, thread, time::Instant
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
    //apu: apu::APU,
    cpu: cpu::CPU,
    memory: Rc<RefCell<memory::Memory>>,
    ppu: ppu::PPU,
    clock: Rc<Cell<u32>>,

    buttons: PressedButtons,
    joyp: RegP1,
    r#if: RegIf,

    channel: Receiver<EmuMessage>,
}

impl GameBoy {
    fn from_revision(path: &Path, model: Model, app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> {
        let rom = Rom::from_vec(fs::read(path)?)?;
        let (send, recv) = channel();
        let graphics = app.get_graphics();
        let proxy = app.get_proxy();

        // Build Game Boy on a new thread
        thread::spawn(move || {
            let io = IOMap::new(model);
            let joyp = RegP1::wrap(io.clone_pointer(MMIO::P1));
            let r#if = RegIf::wrap(io.clone_pointer(MMIO::IF));
            let cpu = cpu::CPU::new(&rom, &io, model);
            let memory = Rc::new(RefCell::new(memory::Memory::from_rom_and_model(rom, model, io).unwrap())); // TODO: Error should be handled rather than unwrapped
            let ppu = ppu::PPU::new(memory.clone(), graphics, proxy);
            let clock = Rc::new(Cell::new(0));
            let buttons = PressedButtons::default();
            GameBoy {
                cpu,
                memory,
                ppu,
                clock,
                buttons,
                joyp,
                r#if,
                channel: recv,
            }
            .main_thread();
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
                print!("{:02X} ", self.memory.borrow().read_u8(x | (y << 4)));
            }
            println!("");
        }
    }
}

const CYCLES_PER_FRAME: u32 = 70224;

impl Emulator for GameBoy {
    fn main_thread(mut self) {
        println!("Launching Wyrm");

        // Enter coroutine scope
        {
            // Generate Coroutines
            let_gen_using!(cpu_coro, |co| self.cpu.coro(self.memory.clone(), co));
            let_gen_using!(ppu_coro, |co| self.ppu.coro(self.clock.clone(), co));

            // Main loop
            // let mut durs = [0.0f64, 0.0f64, 0.0f64];
            'main: loop {
                // let start = Instant::now();
                self.clock.set((self.clock.get() + 1) % CYCLES_PER_FRAME);
                // let clktime = Instant::now();
                cpu_coro.resume();
                // let cputime = Instant::now();
                ppu_coro.resume();
                // let pputime = Instant::now();
                // durs[0] = ((clktime - start).as_secs_f64() + durs[0]) / 2.0f64;
                // durs[1] = ((cputime - clktime).as_secs_f64() + durs[1]) / 2.0f64;
                // durs[2] = ((pputime - cputime).as_secs_f64() + durs[2]) / 2.0f64;

                // When inputs are being polled
                let polling_buttons = !self.joyp.get_polling_buttons();
                let polling_dpad = !self.joyp.get_polling_dpad();
                if polling_buttons || polling_dpad {
                    let mut joypad_interrupt = false;
                    let a_or_right = (self.buttons.a && polling_buttons) || (self.buttons.right && polling_dpad);
                    joypad_interrupt |= self.joyp.get_a_or_right() && a_or_right;
                    self.joyp.set_a_or_right(!a_or_right);
                    let b_or_left = (self.buttons.b && polling_buttons) || (self.buttons.left && polling_dpad);
                    joypad_interrupt |= self.joyp.get_b_or_left() && b_or_left;
                    self.joyp.set_b_or_left(!b_or_left);
                    let start_or_down = (self.buttons.start && polling_buttons) || (self.buttons.down && polling_dpad);
                    joypad_interrupt |= self.joyp.get_start_or_down() && start_or_down;
                    self.joyp.set_start_or_down(!start_or_down);
                    let select_or_up = (self.buttons.select && polling_buttons) || (self.buttons.up && polling_dpad);
                    joypad_interrupt |= self.joyp.get_select_or_up() && select_or_up;
                    self.joyp.set_select_or_up(!select_or_up);

                    if joypad_interrupt {
                        self.r#if.set_joypad(true);
                    }
                } else {
                    // Clear joypad if not polled
                    self.joyp.set_a_or_right(true);
                    self.joyp.set_b_or_left(true);
                    self.joyp.set_start_or_down(true);
                    self.joyp.set_select_or_up(true);
                }

                // Every frame
                if self.clock.get() == 0 {
                    // Display diagnostic information
                    // println!("CLK: {}; CPU: {}; PPU: {}", durs[0], durs[1], durs[2]);

                    // Process any new messages
                    for msg in self.channel.try_iter() {
                        match msg {
                            EmuMessage::KeyboardInput(event) => 'keyevent: {
                                // TODO: Allow remapping controls in the future
                                match event.physical_key {
                                    PhysicalKey::Code(KeyCode::KeyW) => self.buttons.up = event.state.is_pressed(),
                                    PhysicalKey::Code(KeyCode::KeyS) => self.buttons.down = event.state.is_pressed(),
                                    PhysicalKey::Code(KeyCode::KeyA) => self.buttons.left = event.state.is_pressed(),
                                    PhysicalKey::Code(KeyCode::KeyD) => self.buttons.right = event.state.is_pressed(),
                                    PhysicalKey::Code(KeyCode::KeyK) => self.buttons.a = event.state.is_pressed(),
                                    PhysicalKey::Code(KeyCode::KeyJ) => self.buttons.b = event.state.is_pressed(),
                                    PhysicalKey::Code(KeyCode::Enter) => self.buttons.start = event.state.is_pressed(),
                                    PhysicalKey::Code(KeyCode::ShiftRight) => self.buttons.select = event.state.is_pressed(),
                                    _ => break 'keyevent
                                };
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
        } // Coroutines now out of scope

        println!("Exiting Wyrm");

        // Dump memory (for debugging)
        self.dump_mem();
    }
}

#[derive(Debug, Default)]
pub struct PressedButtons {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool
}