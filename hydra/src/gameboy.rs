mod apu;
mod cpu;
mod interrupt;
mod joypad;
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
    gameboy::{apu::Apu, cpu::Cpu, interrupt::{InterruptEnable, InterruptFlags}, joypad::{Button, Dpad, Joypad}, memory::{MemoryMap, MemoryMapped, oam::Oam, rom::Rom, vram::Vram, wram::Wram}, ppu::{Ppu, PpuMode, colormap::{self, CgbColorMap, ColorMap, DmgColorMap}, lcdc::LcdController, state::PpuState}, timer::MasterTimer},
    window::HydraApp
};
use std::{
    cell::{Cell, RefCell}, ffi::OsStr, fs, path::Path, rc::Rc, sync::mpsc::{Receiver, Sender, channel}, thread, time::{Duration, Instant}
};

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Model {
    GameBoy(GBRevision),
    SuperGameBoy(SGBRevision),
    GameBoyColor(CGBRevision),
    GameBoyAdvance(AGBRevision),
}
impl Model {
    const fn as_str(&self) -> &'static str {
        match self {
            Model::GameBoy(GBRevision::DMG0) => "Game Boy (DMG0)",
            Model::GameBoy(GBRevision::DMG) => "Game Boy (DMG)",
            Model::GameBoy(GBRevision::MGB) => "Game Boy Pocket",
            Model::SuperGameBoy(SGBRevision::SGB) => "Super Game Boy",
            Model::SuperGameBoy(SGBRevision::SGB2) => "Super Game Boy 2",
            Model::GameBoyColor(CGBRevision::CGB0) => "Game Boy Color (CGB0)",
            Model::GameBoyColor(CGBRevision::CGB) => "Game Boy Color (CGB)",
            Model::GameBoyAdvance(AGBRevision::AGB0) => "Game Boy Advance (AGB0)",
            Model::GameBoyAdvance(AGBRevision::AGB) => "Game Boy Advance (AGB)",
        }
    }

    fn is_extension_valid(&self, ext: Option<&str>) -> bool {
        match self {
            Model::GameBoy(_) => matches!(ext, Some("gb" | "gbc")),
            Model::SuperGameBoy(_) => matches!(ext, Some("gb" | "gbc")),
            Model::GameBoyColor(_) => matches!(ext, Some("gb" | "gbc")),
            Model::GameBoyAdvance(_) => matches!(ext, Some("gb" | "gbc" | "gba")),
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

pub enum GbMode {
    DMG,
    CGB
}


pub struct GameBoy {
    apu: Rc<RefCell<Apu>>,
    cpu: Option<Cpu>,
    memory: Rc<RefCell<MemoryMap>>,
    ppu: Option<Ppu>,

    channel: Receiver<EmuMessage>,
}

fn read_rom(path: &Path) -> Result<Rom, HydraIOError> {
    Ok(Rom::from_vec(fs::read(path)?)?)
}

impl GameBoy {
    pub fn new(path: &Path, model: Model, app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> {
        let ext = path.extension().and_then(OsStr::to_str);
        if !model.is_extension_valid(ext) {
            return Err(HydraIOError::InvalidEmulator(model.as_str(), ext.map(str::to_string)));
        }

        let rom = read_rom(path)?;
        let mode = match model.is_color() && rom.supports_cgb_mode() {
            true => GbMode::CGB,
            false => GbMode::DMG
        };

        GameBoy::with_mode(rom, model, mode, app)
    }

    fn with_mode(rom: Rom, model: Model, mode: GbMode, app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> {
        let (send, recv) = channel();
        let graphics = app.clone_graphics();
        let audio = app.clone_audio();
        let proxy = app.clone_proxy();

        // Build Game Boy on a new thread
        thread::spawn(move || {
            let model = Rc::new(model);
            let mode = Rc::new(mode);

            let memory = Rc::new(RefCell::new(memory::MemoryMap::new(&model, mode.clone()).unwrap())); // TODO: Error should be handled rather than unwrapped
            let ppu = Some(Ppu::new(model.clone(), memory.clone(), graphics, proxy));
            let apu = Rc::new(RefCell::new(Apu::new(memory.clone(), audio)));
            let cpu = Some(Cpu::new(&rom, &model, &mode, memory.clone()));
            memory.borrow_mut().hot_swap_rom(rom);
            GameBoy {
                apu,
                cpu,
                memory,
                ppu,
                channel: recv,
            }.main_thread();
        });
        Ok(send)
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
        let_gen_using!(cpu_coro, |co| cpu.coro(co, false));
        let_gen_using!(ppu_coro, |co| ppu.coro(co));

        // Main loop
        'main: loop {
            {
                let memory = &mut *self.memory.borrow_mut();
                memory.tick_dma();

                let interrupt_flags = &mut memory.interrupt_flags;
                let ppu_state = &mut memory.ppu_state;
                let apu_state = &mut memory.apu_state;

                memory.timer.tick(interrupt_flags, ppu_state, apu_state);
                self.apu.borrow_mut().dot_tick(apu_state);
            }
            if self.memory.borrow_mut().timer.is_system_cycle() {
                cpu_coro.resume();
            }
            self.memory.borrow_mut().timer.refresh_tima_if_overflowing();
            ppu_coro.resume();

            let memory = &mut *self.memory.borrow_mut();
            let interrupt_flags = &mut memory.interrupt_flags;
            let ppu_state = &mut memory.ppu_state;
            let apu_state = &mut memory.apu_state;
            
            // Every frame
            if memory.timer.get_ppu_dots(ppu_state) == 0 {

                // Send audio for playback
                self.apu.borrow_mut().frame();

                // Process any new messages
                for msg in self.channel.try_iter() {
                    match msg {
                        // TODO: Allow remapping controls in the future
                        EmuMessage::KeyboardInput(KeyEvent {state, physical_key: PhysicalKey::Code(keycode), .. }) => match keycode {
                            KeyCode::KeyW => memory.joypad.press_dpad(Dpad::Up, state.is_pressed(), interrupt_flags),
                            KeyCode::KeyS => memory.joypad.press_dpad(Dpad::Down, state.is_pressed(), interrupt_flags),
                            KeyCode::KeyA => memory.joypad.press_dpad(Dpad::Left, state.is_pressed(), interrupt_flags),
                            KeyCode::KeyD => memory.joypad.press_dpad(Dpad::Right, state.is_pressed(), interrupt_flags),
                            KeyCode::KeyK => memory.joypad.press_button(Button::A, state.is_pressed(), interrupt_flags),
                            KeyCode::KeyJ => memory.joypad.press_button(Button::B, state.is_pressed(), interrupt_flags),
                            KeyCode::Enter => memory.joypad.press_button(Button::Start, state.is_pressed(), interrupt_flags),
                            KeyCode::ShiftRight => memory.joypad.press_button(Button::Select, state.is_pressed(), interrupt_flags),
                            _ => {}
                        }
                        EmuMessage::HotSwap(path) => {
                            // if let Err(e) = read_rom(path).and_then(|rom| memory.hot_swap_rom(rom)) {
                            //     println!("{}", e);
                            // }
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