mod cpu;
mod memory;
mod ppu;

use genawaiter::stack::let_gen_using;
use winit::{event_loop::EventLoopProxy, window::Window};

use crate::{
    common::{
        emulator::{EmuMessage, Emulator},
        errors::HydraIOError,
    },
    config::Config,
    gameboy::memory::io::IO,
    graphics::Graphics, window::{HydraApp, UserEvent},
};
use std::{
    cell::RefCell, ffi::OsStr, fs, path::Path, rc::Rc, sync::{
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
    clock: u32,

    channel: Receiver<EmuMessage>,
}

impl GameBoy {
    fn from_revision(path: &Path, model: Model, app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> {
        let rom = fs::read(path)?.into_boxed_slice();
        let (send, recv) = channel();
        let window = app.get_window();
        let graphics = app.get_graphics();
        let proxy = app.get_proxy();

        // Build Game Boy on a new thread
        thread::spawn(move || {
            let io = IO::new(model);
            let cpu = cpu::CPU::new(&rom, &io, model);
            let ppu = ppu::PPU::new(&io, window, graphics, proxy);
            let memory = Rc::new(RefCell::new(memory::Memory::from_rom_and_model(rom, model, io).unwrap())); // TODO: Error should be handled rather than unwrapped
            GameBoy {
                cpu,
                memory,
                ppu,
                clock: 0,
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

            // Main loop
            // let mut durs = [0.0f64, 0.0f64, 0.0f64];
            'main: loop {
                // let start = Instant::now();
                self.clock = (self.clock + 1) % CYCLES_PER_FRAME;
                // let clktime = Instant::now();
                cpu_coro.resume();
                // let cputime = Instant::now();
                self.ppu.step(&self.clock);
                // let pputime = Instant::now();
                // durs[0] = ((clktime - start).as_secs_f64() + durs[0]) / 2.0f64;
                // durs[1] = ((cputime - clktime).as_secs_f64() + durs[1]) / 2.0f64;
                // durs[2] = ((pputime - cputime).as_secs_f64() + durs[2]) / 2.0f64;

                // Every frame
                if self.clock == 0 {
                    // Display diagnostic information
                    // println!("CLK: {}; CPU: {}; PPU: {}", durs[0], durs[1], durs[2]);

                    // Process any new messages
                    for msg in self.channel.try_iter() {
                        match msg {
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
