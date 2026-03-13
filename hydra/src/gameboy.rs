mod apu;
mod cpu;
mod interrupt;
mod joypad;
mod memory;
mod ppu;
mod serial;
mod timer;

use winit::{event::KeyEvent, keyboard::{KeyCode, PhysicalKey}};

use crate::{
    common::{
        bit::{BitVec, MaskedBitVec}, emulator::{EmuMessage, Emulator}, errors::HydraIOError
    },
    gameboy::{apu::Apu, cpu::Cpu, interrupt::{InterruptEnable, InterruptFlags}, joypad::{Button, Dpad, Joypad}, memory::{MemoryMap, MemoryMapped, oam::Oam, rom::Rom, vram::Vram, wram::Wram}, ppu::{Ppu, PpuMode, colormap::{self, CgbColorMap, ColorMap, DmgColorMap}, state::PpuState}, timer::MasterTimer},
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
    apu: Apu,
    cpu: Option<Cpu>,
    memory: MemoryMap,
    ppu: Ppu,

    channel: Receiver<EmuMessage>,

    running: bool,
    turbo: bool,
    dump_cpu: bool,
    next_frame_instant: Instant,
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

            let ppu = Ppu::new(model.clone());
            let apu = Apu::new(audio);
            let cpu = Some(Cpu::new(&rom, &model, &mode));
            let mut memory = MemoryMap::new(model.clone(), mode.clone(), graphics, proxy).unwrap(); // TODO: Error should be handled rather than unwrapped
            memory.hot_swap_rom(rom).unwrap();

            GameBoy {
                apu,
                cpu,
                memory,
                ppu,

                channel: recv,

                running: true,
                turbo: false,
                dump_cpu: false,
                next_frame_instant: Instant::now()
            }.main_thread();
        });
        Ok(send)
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    fn dump_mem(&self) {
        for y in 0..=0xFFF {
            print!("{:#06X}:   ", y << 4);
            for x in 0..=0xF {
                print!("{:02X} ", self.memory.read_u8(x | (y << 4), true));
            }
            println!("");
        }
    }

    fn cycle_components(&mut self) {
        let memory = &mut self.memory;
        // Finish current M-cycle
        memory.timer.refresh_tima_if_overflowing();

        // Loop until next M-cycle
        loop { 
            // Finish current T-cycle
            self.ppu.coro(memory);
            
            // Every frame
            if memory.timer.is_new_frame() {
                // Sleep until next frame (unless turbo is active)
                if self.turbo {
                    // If turbo is on, instantly render next frame without delays
                    self.next_frame_instant = Instant::now();
                } else {
                    // Delay thread until next frame if turbo is off
                    let duration_until_next = self.next_frame_instant.saturating_duration_since(Instant::now());
                    thread::sleep(duration_until_next);
                }
                // Set expected timing for next frame
                const SECS_PER_FRAME: f64 = 1f64 / 60f64;
                self.next_frame_instant += Duration::from_secs_f64(SECS_PER_FRAME);

                // Send audio for playback
                self.apu.frame();

                // Process any new messages
                for msg in self.channel.try_iter() {
                    match msg {
                        // TODO: Allow remapping controls in the future
                        EmuMessage::KeyboardInput(KeyEvent {state, physical_key: PhysicalKey::Code(keycode), .. }) => match keycode {
                            KeyCode::KeyW => memory.joypad.press_dpad(Dpad::Up, state.is_pressed(), &mut memory.interrupt_flags),
                            KeyCode::KeyS => memory.joypad.press_dpad(Dpad::Down, state.is_pressed(), &mut memory.interrupt_flags),
                            KeyCode::KeyA => memory.joypad.press_dpad(Dpad::Left, state.is_pressed(), &mut memory.interrupt_flags),
                            KeyCode::KeyD => memory.joypad.press_dpad(Dpad::Right, state.is_pressed(), &mut memory.interrupt_flags),
                            KeyCode::KeyK => memory.joypad.press_button(Button::A, state.is_pressed(), &mut memory.interrupt_flags),
                            KeyCode::KeyJ => memory.joypad.press_button(Button::B, state.is_pressed(), &mut memory.interrupt_flags),
                            KeyCode::Enter => memory.joypad.press_button(Button::Start, state.is_pressed(), &mut memory.interrupt_flags),
                            KeyCode::ShiftRight => memory.joypad.press_button(Button::Select, state.is_pressed(), &mut memory.interrupt_flags),
                            KeyCode::Space => self.turbo = state.is_pressed(),
                            KeyCode::AltLeft => self.dump_cpu = state.is_pressed(),
                            _ => {}
                        }
                        EmuMessage::HotSwap(path) => {
                            if let Err(e) = read_rom(path).and_then(|rom| memory.hot_swap_rom(rom)) {
                                println!("{}", e);
                            }
                        },
                        EmuMessage::Stop => self.running = false,
                        _ => {} // Do nothing
                    }
                }
            }

            // Next T-cycle
            memory.timer.tick(&mut memory.interrupt_flags, &mut memory.ppu_state, &mut memory.apu_state);
            self.apu.dot_tick(&mut memory.apu_state);

            // Break for next M-cycle when applicable
            if memory.timer.is_system_cycle() {break;}
        }

        // Start next M-cycle
        memory.tick_dma();
        memory.serial.tick(&mut memory.interrupt_flags);
    }
}

impl Emulator for GameBoy {
    fn main_thread(mut self) {
        println!("Launching Wyrm");

        // Start main loop
        let mut cpu = self.cpu.take().unwrap();
        cpu.coro(&mut self, true);

        println!("Exiting Wyrm");

        // Dump memory (for debugging)
        self.dump_mem();
    }
}