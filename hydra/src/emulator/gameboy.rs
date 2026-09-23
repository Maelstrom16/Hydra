mod apu;
mod cpu;
mod interrupt;
mod joypad;
mod memory;
mod ppu;
mod serial;
mod timer;

use ringbuf::{HeapProd, traits::Producer};
use serde::{Deserialize, Serialize};
use wgpu::{Device, Queue};
use winit::{event::KeyEvent, keyboard::{KeyCode, PhysicalKey}};

use crate::{
    common::{
        bit::{BitVec, MaskedBitVec}, errors::HydraIOError
    }, emulator::{EmuMessage, Emulator, SavePath, gameboy::{apu::Apu, cpu::Cpu, interrupt::{InterruptEnable, InterruptFlags}, joypad::{JoypButton, JoypDpad, Joypad}, memory::{MemoryMap, MemoryMapped, hdma::{CgbHdmAccessor, DmgHdmAccessor, HdmAccessor}, oam::Oam, rom::{Rom, RomHeader}, vram::Vram, wram::Wram}, ppu::{Ppu, PpuMode, colormap::{self, CgbColorMap, ColorMap, DmgColorMap}, state::PpuState}, timer::MasterTimer}}, graphics::Graphics, window::HydraApp
};
use std::{
    cell::{Cell, RefCell}, ffi::OsStr, fs, path::{Path, PathBuf}, rc::Rc, sync::{Arc, RwLock, mpsc::{Receiver, Sender, channel}}, thread, time::{Duration, Instant}
};

pub trait GbModel: Send + 'static {
    type Revision: GbRevision<Model = Self>;
    type ColorMap: ColorMap;
    type HdmAccessor: HdmAccessor;

    fn as_str(&self) -> &'static str;
    fn revision(&self) -> Self::Revision;
    fn is_extension_valid(ext: Option<&str>) -> bool;
    fn is_color() -> bool;
    fn is_monochrome() -> bool {!Self::is_color()}

    // TODO: Should be removable once boot ROMs are functional
    fn initial_cpu_registers(&self, header: &RomHeader, cgb_mode: bool) -> ([u8; 2], [u8; 2], [u8; 2], [u8; 2]);
    fn initial_div_full(&self) -> u16;
    fn initial_ly(&self) -> u8;
}

pub struct Dmg(pub DmgRevision);
impl GbModel for Dmg {
    type Revision = DmgRevision;
    type ColorMap = DmgColorMap;
    type HdmAccessor = DmgHdmAccessor;

    fn as_str(&self) -> &'static str {
        match self.0 {
            DmgRevision::DMG0 => "Game Boy (DMG0)",
            DmgRevision::DMG => "Game Boy (DMG)",
            DmgRevision::MGB => "Game Boy Pocket",
        }
    }

    fn revision(&self) -> Self::Revision { self.0 }
    fn is_extension_valid(ext: Option<&str>) -> bool { matches!(ext, Some("gb" | "gbc")) }
    fn is_color() -> bool { false }

    fn initial_cpu_registers(&self, header: &RomHeader, cgb_mode: bool) -> ([u8; 2], [u8; 2], [u8; 2], [u8; 2]) {
        let (af, b, e, hl);
        match self.0 {
            DmgRevision::DMG0 => {
                af = [0b0000 << 4, 0x01];
                b = 0xFF;
                e = 0xC1;
                hl = [0x03, 0x84];
            }
            DmgRevision::DMG => {
                af = [if header.get_header_checksum() == 0 { 0b1000 << 4 } else { 0b1011 << 4 }, 0x01];
                b = 0x00;
                e = 0xD8;
                hl = [0x4D, 0x01];
            }
            DmgRevision::MGB => {
                af = [if header.get_header_checksum() == 0 { 0b1000 << 4 } else { 0b1011 << 4 }, 0xFF];
                b = 0x00;
                e = 0xD8;
                hl = [0x4D, 0x01];
            }
        }
        (af, [0x13, b], [e, 0x00], hl)
    }

    fn initial_div_full(&self) -> u16 {
        (match self.0 { 
            DmgRevision::DMG0 => 0x18,
            _ => 0xAB,
        }) << 6
    }

    fn initial_ly(&self) -> u8 {
        match self.0 { 
            DmgRevision::DMG0 => 0x91,
            _ => 0x00,
        }
    }
}

pub struct Sgb(pub SgbRevision);
impl GbModel for Sgb {
    type Revision = SgbRevision;
    type ColorMap = DmgColorMap;
    type HdmAccessor = DmgHdmAccessor;

    fn as_str(&self) -> &'static str {
        match self.0 {
            SgbRevision::SGB => "Super Game Boy",
            SgbRevision::SGB2 => "Super Game Boy 2",
        }
    }

    fn revision(&self) -> Self::Revision { self.0 }
    fn is_extension_valid(ext: Option<&str>) -> bool { matches!(ext, Some("gb" | "gbc")) }
    fn is_color() -> bool { false }

    fn initial_cpu_registers(&self, header: &RomHeader, cgb_mode: bool) -> ([u8; 2], [u8; 2], [u8; 2], [u8; 2]) {
        let a = match self.0 {
            SgbRevision::SGB => 0x01,
            SgbRevision::SGB2 => 0xFF
        };

        ([0b0000 << 4, a], [0x14, 0x00], [0x00, 0x00], [0x60, 0xC0])
    }

    fn initial_div_full(&self) -> u16 {
        rand::random_range(0x00..=0xFF) << 6 // TODO: Number is supposed to be based on boot rom cycles
    }

    fn initial_ly(&self) -> u8 {
        rand::random_range(0x00..=0x99) // TODO: Number is supposed to be based on boot ROM cycles
    }
}

pub struct Cgb(pub CgbRevision);
impl GbModel for Cgb {
    type Revision = CgbRevision;
    type ColorMap = CgbColorMap;
    type HdmAccessor = CgbHdmAccessor;

    fn as_str(&self) -> &'static str {
        match self.0 {
            CgbRevision::CGB0 => "Game Boy Color (CGB0)",
            CgbRevision::CGB => "Game Boy Color (CGB)",
        }
    }

    fn revision(&self) -> Self::Revision { self.0 }
    fn is_extension_valid(ext: Option<&str>) -> bool { matches!(ext, Some("gb" | "gbc")) }
    fn is_color() -> bool { true }

    fn initial_cpu_registers(&self, header: &RomHeader, cgb_mode: bool) -> ([u8; 2], [u8; 2], [u8; 2], [u8; 2]) {
        let (b, de, hl);
        match cgb_mode {
            true => {
                b = 0x00;
                de = [0x56, 0xFF];
                hl = [0x0D, 0x00];
            }
            false => {
                let mut b_inner = 0x00;
                let mut hl_inner = [0x7C, 0x00];
                if header.has_publisher_rnd1() {
                    // If either licensee code is 0x01, B = sum of all title bytes
                    b_inner = header.get_title().iter().sum();
                    if b_inner == 0x43 || b_inner == 0x58 {
                        // And, check special cases for HL
                        hl_inner = [0x1A, 0x99];
                    }
                }
                b = b_inner;
                de = [0x08, 0x00];
                hl = hl_inner;
            }
        }
        ([0b1000 << 4, 0x11], [0x00, b], de, hl)
    }

    fn initial_div_full(&self) -> u16 {
        rand::random_range(0x00..=0xFF) << 6 // TODO: Number is supposed to be based on boot rom cycles
    }

    fn initial_ly(&self) -> u8 {
        rand::random_range(0x00..=0x99) // TODO: Number is supposed to be based on boot ROM cycles
    }
}

pub struct Agb(pub AgbRevision);
impl GbModel for Agb {
    type Revision = AgbRevision;
    type ColorMap = CgbColorMap;
    type HdmAccessor = CgbHdmAccessor;

    fn as_str(&self) -> &'static str {
        match self.0 {
            AgbRevision::AGB0 => "Game Boy Advance (AGB0)",
            AgbRevision::AGB => "Game Boy Advance (AGB)",
        }
    }

    fn revision(&self) -> Self::Revision { self.0 }
    fn is_extension_valid(ext: Option<&str>) -> bool { matches!(ext, Some("gb" | "gbc" | "gba")) }
    fn is_color() -> bool { true }

    fn initial_cpu_registers(&self, header: &RomHeader, cgb_mode: bool) -> ([u8; 2], [u8; 2], [u8; 2], [u8; 2]) {
        let (f, b, de, hl);
        match cgb_mode {
            true => {
                f = 0b0000 << 4;
                b = 0x01;
                de = [0x56, 0xFF];
                hl = [0x0D, 0x00];
            }
            false => {
                let mut b_inner = 0x01;
                let mut hl_inner = [0x7C, 0x00];
                let mut f_inner = 0b00000000;
                if header.has_publisher_rnd1() {
                    // If either licensee code is 0x01, B = sum of all title bytes
                    b_inner = header.get_title().iter().sum();
                    if b_inner & 0b1111 == 0 {
                        // Last op is an INC; set h flag...
                        f_inner |= 0b0010 << 4;
                        if b_inner == 0 {
                            // ...and z flag if necessary
                            f_inner |= 0b1000 << 4
                        }
                    } else if b_inner == 0x44 || b_inner == 0x59 {
                        // Otherwise, still check special cases for HL
                        hl_inner = [0x1A, 0x99];
                    }
                }
                f = f_inner;
                b = b_inner;
                de = [0x08, 0x00];
                hl = hl_inner;
            }
        }
        ([f, 0x11], [0x00, b], de, hl)
    }

    fn initial_div_full(&self) -> u16 {
        rand::random_range(0x00..=0xFF) << 6 // TODO: Number is supposed to be based on boot ROM cycles
    }

    fn initial_ly(&self) -> u8 {
        rand::random_range(0x00..=0x99) // TODO: Number is supposed to be based on boot ROM cycles
    }
}

trait GbRevision {
    type Model: GbModel<Revision = Self>;
    fn into_model(self) -> Self::Model;
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DmgRevision {
    DMG0,
    DMG,
    MGB,
}

impl GbRevision for DmgRevision {
    type Model = Dmg;
    fn into_model(self) -> Self::Model { Dmg(self) }
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SgbRevision {
    SGB,
    SGB2,
}

impl GbRevision for SgbRevision {
    type Model = Sgb;
    fn into_model(self) -> Self::Model { Sgb(self) }
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CgbRevision {
    CGB0,
    CGB,
}

impl GbRevision for CgbRevision {
    type Model = Cgb;
    fn into_model(self) -> Self::Model { Cgb(self) }
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AgbRevision {
    AGB0,
    AGB,
}

impl GbRevision for AgbRevision {
    type Model = Agb;
    fn into_model(self) -> Self::Model { Agb(self) }
}


pub struct GbState<M: GbModel> {
    apu: Apu,
    cpu: Option<Cpu<M>>,
    memory: MemoryMap<M>,
    ppu: Ppu<M>,
}

pub struct GameBoy<M: GbModel> {
    state: GbState<M>,

    channel: Receiver<EmuMessage>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    ring_buffer: HeapProd<f32>,

    powered_on: bool,
    running: bool,
    turbo: bool,
    dump_cpu: bool,
    next_frame_instant: Instant,

    rom_path: PathBuf,
}

fn read_as_rom(path: &Path) -> Result<RomHeader, HydraIOError> {
    Ok(RomHeader::load_from_file(path)?)
}

impl<M: GbModel> GameBoy<M> {
    pub fn new(rom_path: &Path, model: M, app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> {
        let ext = rom_path.extension().and_then(OsStr::to_str);
        if !M::is_extension_valid(ext) {
            return Err(HydraIOError::InvalidEmulator(model.as_str(), ext.map(str::to_string)));
        }

        let header = read_as_rom(rom_path)?;
        let cgb_mode = M::is_color() && header.supports_cgb_mode();

        let rom_path = rom_path.to_owned();

        let (send, recv) = channel();
        let controllers = app.clone_controllers();
        let graphics = app.clone_graphics();
        let (device, queue) = {
            let gtemp = graphics.read().unwrap();
            (gtemp.get_device(), gtemp.get_queue())
        };
        let audio = app.clone_audio();
        let ring_buffer = audio.write().unwrap().get_producer();
        let proxy = app.clone_proxy();

        Self::init_graphics(&graphics);

        // Build Game Boy on a new thread
        thread::spawn(move || {
            let ppu = Ppu::<M>::new();
            let apu = Apu::new(audio);
            let cpu = Some(Cpu::new(&header, &model, cgb_mode));
            let mut memory = MemoryMap::new(&model, cgb_mode, controllers, graphics, proxy).unwrap(); // TODO: Error should be handled rather than unwrapped
            memory.hot_swap_rom(header, device.clone(), queue.clone()).unwrap();

            GameBoy {
                state: GbState { apu, cpu, memory, ppu },

                channel: recv,
                device,
                queue,
                ring_buffer,

                powered_on: true,
                running: true,
                turbo: false,
                dump_cpu: false,
                next_frame_instant: Instant::now(),

                rom_path
            }.main_thread();
        });
        Ok(send)
    }

    fn init_graphics(graphics: &Arc<RwLock<Graphics>>) {
        graphics.write().unwrap().init_emulator(ppu::SCREEN_WIDTH as u32, ppu::SCREEN_HEIGHT as u32);
    }

    pub fn is_powered_on(&self) -> bool {
        self.powered_on
    }

    fn dump_mem(&self) {
        for y in 0..=0xFFF {
            print!("{:#06X}:   ", y << 4);
            for x in 0..=0xF {
                print!("{:02X} ", self.state.memory.read_u8(x | (y << 4), true));
            }
            println!("");
        }
    }

    fn cycle_components(&mut self) {
        let mut memory = &mut self.state.memory;
        // Finish current M-cycle
        memory.timer.refresh_tima_if_overflowing();

        // Loop until next M-cycle
        loop { 
            // Finish current T-cycle
            self.state.ppu.coro(memory);
            
            // Every frame
            if memory.timer.is_new_frame() {
                // Push SRAM to file
                memory.cartridge.as_ref().inspect(|rom| rom.save());

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

                // Process rumble (if applicable)
                if let Some(ref mut mbc) = memory.cartridge {mbc.frame();};

                // Send audio for playback
                self.ring_buffer.push_slice(self.state.apu.frame().as_slice());

                // Process any new messages
                'message: loop {
                    for msg in self.channel.try_iter() {
                        match msg {
                            // TODO: Allow remapping controls in the future
                            EmuMessage::Pause => {
                                self.running = !self.running;
                                self.next_frame_instant = Instant::now();
                            }
                            EmuMessage::SaveStateSlot(slot) => {
                                let path = self.rom_path.state_path(slot);
                                memory = &mut self.state.memory; // Restore memory reference after passing
                                match std::fs::write(&path, &[]) {
                                    Ok(_) => println!("SAVING STATE TO {}", path.to_str().unwrap()),
                                    Err(e) => {
                                        rfd::MessageDialog::new()
                                            .set_title("Unable to Save State ".to_owned() + &slot.to_string())
                                            .set_description(e.to_string())
                                            .set_buttons(rfd::MessageButtons::Ok)
                                            .set_level(rfd::MessageLevel::Warning)
                                            .show();
                                        self.next_frame_instant = Instant::now();
                                    }
                                }
                            }
                            EmuMessage::LoadStateSlot(slot) => {
                                let path = self.rom_path.state_path(slot);
                                memory = &mut self.state.memory; // Restore memory reference after passing
                                match std::fs::read(&path) {
                                    Ok(_) => println!("LOADING STATE FROM {}", path.to_str().unwrap()),
                                    Err(e) => {
                                        rfd::MessageDialog::new()
                                            .set_title("Unable to Load State ".to_owned() + &slot.to_string())
                                            .set_description(e.to_string())
                                            .set_buttons(rfd::MessageButtons::Ok)
                                            .set_level(rfd::MessageLevel::Warning)
                                            .show();
                                        self.next_frame_instant = Instant::now();
                                    }
                                }
                            }
                            EmuMessage::KeyboardInput(KeyEvent {state, physical_key: PhysicalKey::Code(keycode), .. }) if self.running => match keycode {
                                KeyCode::KeyW => memory.joypad.keyboard_vecs.press_dpad(JoypDpad::Up, state.is_pressed()),
                                KeyCode::KeyS => memory.joypad.keyboard_vecs.press_dpad(JoypDpad::Down, state.is_pressed()),
                                KeyCode::KeyA => memory.joypad.keyboard_vecs.press_dpad(JoypDpad::Left, state.is_pressed()),
                                KeyCode::KeyD => memory.joypad.keyboard_vecs.press_dpad(JoypDpad::Right, state.is_pressed()),
                                KeyCode::KeyK => memory.joypad.keyboard_vecs.press_button(JoypButton::A, state.is_pressed()),
                                KeyCode::KeyJ => memory.joypad.keyboard_vecs.press_button(JoypButton::B, state.is_pressed()),
                                KeyCode::Enter => memory.joypad.keyboard_vecs.press_button(JoypButton::Start, state.is_pressed()),
                                KeyCode::ShiftRight => memory.joypad.keyboard_vecs.press_button(JoypButton::Select, state.is_pressed()),
                                KeyCode::Space => self.turbo = state.is_pressed(),
                                KeyCode::AltLeft => self.dump_cpu = state.is_pressed(),
                                _ => {}
                            }
                            EmuMessage::HotSwap(path) => {
                                if let Err(e) = read_as_rom(path).and_then(|rom| memory.hot_swap_rom(rom, self.device.clone(), self.queue.clone())) {
                                    println!("{}", e);
                                }
                            },
                            EmuMessage::Stop => self.powered_on = false,
                            _ => {} // Do nothing
                        }
                    }
                    if self.running {break 'message}
                }

                memory.joypad.update_controller_vecs(&mut memory.interrupt_flags);
            }

            // Next T-cycle
            memory.timer.tick(&mut memory.interrupt_flags, &mut memory.ppu_state, &mut memory.apu_state);
            self.state.apu.dot_tick(&mut memory.apu_state);

            // Break for next M-cycle when applicable
            if memory.timer.is_system_cycle() {break;}
        }

        // Start next M-cycle
        memory.tick_dma();
        if let Some(ref mut mbc) = memory.cartridge {mbc.tick();};
        memory.serial.tick(&mut memory.interrupt_flags);
    }
}

impl<M: GbModel> Emulator for GameBoy<M> {
    const CONSOLE_NAME: &str = "Game Boy";
    const CORE_NAME: &str = "Wyrm";
    const FILE_FILTERS: &[(&str, &[&str])] = &[super::GB_FILE_FILTER];

    type Model = M::Revision;
    
    fn main_thread(mut self) {
        println!("Launching {}", Self::CORE_NAME);

        // Start main loop
        let mut cpu = self.state.cpu.take().unwrap();
        cpu.coro(&mut self, true);

        println!("Exiting {}", Self::CORE_NAME);

        // One last save to file
        self.state.memory.cartridge.as_ref().inspect(|rom| rom.save());

        // Dump memory (for debugging)
        self.dump_mem();
    }
    
    fn try_init(revision: Self::Model, rom_path: &PathBuf, app: &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError> { 
        GameBoy::new(rom_path, revision.into_model(), app)
    }
}