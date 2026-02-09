mod apu;
mod cpu;
mod memory;
mod ppu;

use genawaiter::stack::let_gen_using;
use winit::{event::KeyEvent, keyboard::{KeyCode, PhysicalKey}};

use crate::{
    common::{
        emulator::{EmuMessage, Emulator},
        errors::HydraIOError,
    },
    gameboy::memory::{io::{IoMap, MMIO, deserialized::{RegIf, RegJoyp}}, rom::Rom, vram::Vram},
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
    //apu: apu::APU,
    cpu: Option<cpu::Cpu>,
    memory: Rc<RefCell<memory::MemoryMap>>,
    ppu: Option<ppu::Ppu>,
    clock: Rc<Cell<u32>>,

    buttons: PressedButtons,
    joyp: RegJoyp,
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
            let io = IoMap::new(model);
            let joyp = RegJoyp::wrap(io.clone_pointer(MMIO::JOYP));
            let r#if = RegIf::wrap(io.clone_pointer(MMIO::IF));
            let cpu = Some(cpu::Cpu::new(&rom, &io, model));
            let vram = Rc::new(RefCell::new(Vram::new(model, &io)));
            let ppu = Some(ppu::Ppu::new(vram.clone(), &io, graphics, proxy));
            let memory = Rc::new(RefCell::new(memory::MemoryMap::from_rom_and_model(rom, model, vram, io).unwrap())); // TODO: Error should be handled rather than unwrapped
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

    fn update_joyp(&mut self) {
        // Save pressed buttons and set bottom nybble (i.e. unpress them)
        let joyp_original = self.joyp.get_entire();
        self.joyp.set_entire(joyp_original | 0x0F);

        let polling_buttons = !self.joyp.get_polling_buttons();
        let polling_dpad = !self.joyp.get_polling_dpad();
        if polling_buttons || polling_dpad {
            let mut joypad_interrupt = false;

            let mut update_pair = |button: bool, dpad: bool, get_fn: fn(&RegJoyp) -> bool, set_fn: fn(&mut RegJoyp, bool)| {
                let either_pressed = (button && polling_buttons) || (dpad && polling_dpad);
                joypad_interrupt |= get_fn(&self.joyp) && either_pressed;
                set_fn(&mut self.joyp, !either_pressed);
            };
            
            update_pair(self.buttons.a, self.buttons.right, RegJoyp::get_a_or_right, RegJoyp::set_a_or_right);
            update_pair(self.buttons.b, self.buttons.left, RegJoyp::get_b_or_left, RegJoyp::set_b_or_left);
            update_pair(self.buttons.start, self.buttons.down, RegJoyp::get_start_or_down, RegJoyp::set_start_or_down);
            update_pair(self.buttons.select, self.buttons.up, RegJoyp::get_select_or_up, RegJoyp::set_select_or_up);

            if joypad_interrupt {
                self.r#if.set_joypad(true);
            }
        }
    }
}

const CYCLES_PER_FRAME: u32 = 70224;

impl Emulator for GameBoy {
    fn main_thread(mut self) {
        println!("Launching Wyrm");

        // Generate Coroutines
        let mut cpu = self.cpu.take().unwrap();
        let mut ppu = self.ppu.take().unwrap();
        let_gen_using!(cpu_coro, |co| cpu.coro(self.memory.clone(), co, false));
        let_gen_using!(ppu_coro, |co| ppu.coro(self.clock.clone(), co));
        apu::test();

        // Main loop
        'main: loop {
            self.clock.set((self.clock.get() + 1) % CYCLES_PER_FRAME);
            cpu_coro.resume();
            ppu_coro.resume();

            self.update_joyp();

            // Every frame
            if self.clock.get() == 0 {
                // Process any new messages
                for msg in self.channel.try_iter() {
                    match msg {
                        // TODO: Allow remapping controls in the future
                        EmuMessage::KeyboardInput(KeyEvent {state, physical_key: PhysicalKey::Code(keycode), .. }) => match keycode {
                            KeyCode::KeyW => self.buttons.up = state.is_pressed(),
                            KeyCode::KeyS => self.buttons.down = state.is_pressed(),
                            KeyCode::KeyA => self.buttons.left = state.is_pressed(),
                            KeyCode::KeyD => self.buttons.right = state.is_pressed(),
                            KeyCode::KeyK => self.buttons.a = state.is_pressed(),
                            KeyCode::KeyJ => self.buttons.b = state.is_pressed(),
                            KeyCode::Enter => self.buttons.start = state.is_pressed(),
                            KeyCode::ShiftRight => self.buttons.select = state.is_pressed(),
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