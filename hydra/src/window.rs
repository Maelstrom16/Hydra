use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use muda::MenuEvent;
use muda::accelerator::{Accelerator, Code, Modifiers};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::{KeyCode, PhysicalKey};
#[cfg(target_os = "macos")]
use winit::platform::macos::WindowAttributesExtMacOS;
use winit::window::{Window, WindowId};

use crate::audio::Audio;
use crate::common::errors::HydraIOError;
use crate::config::Config;
use crate::emulator::gameboy::{Cgb, Dmg, GameBoy, Sgb};
use crate::emulator::gba::{self, GameBoyAdvance};
use crate::emulator::n3ds::N3ds;
use crate::emulator::nds::Nds;
use crate::emulator::{self, AllEmulator, EmuMessage, Emulator, gameboy};
use crate::input::{ControllerState, SdlContainer};
use crate::graphics::Graphics;
use crate::ui::UserInterface;

pub struct HydraApp {
    config: Config,
    window: Option<Arc<Window>>,
    graphics: Option<Arc<RwLock<Graphics>>>,
    sdl: SdlContainer,
    audio: Option<Arc<RwLock<Audio>>>,
    ui: Option<UserInterface>,
    proxy: EventLoopProxy<UserEvent>,

    emulator: Option<Sender<EmuMessage>>,

    frame_counter: u64,
    last_second: std::time::Instant,
}

impl HydraApp {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        HydraApp {
            config: Config::from_toml(),
            window: None, // Initialized on app startup
            graphics: None, // Initialized on app startup
            sdl: SdlContainer::new(),
            audio: None, // Initialized on app startup
            ui: None, // Initialized on app startup
            proxy,

            emulator: None, // Initialized when opening a ROM

            frame_counter: 0,
            last_second: std::time::Instant::now(),
        }
    }

    fn init_app(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Hydra");
        self.window = Some(Arc::new(event_loop.create_window(window_attributes).unwrap()));
        self.graphics = Some(Arc::new(RwLock::new(futures::executor::block_on(Graphics::new(self.window.clone().unwrap())))));
        self.audio = Some(Arc::new(RwLock::new(Audio::new())));
        self.ui = Some(UserInterface::initialize(self.window.as_ref().unwrap(), &self.config));
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn clone_window(&self) -> Arc<Window> {
        Arc::clone(self.window.as_ref().unwrap())
    }

    pub fn clone_controllers(&self) -> Arc<RwLock<ControllerState>> {
        self.sdl.clone_p1()
    }

    pub fn clone_graphics(&self) -> Arc<RwLock<Graphics>> {
        Arc::clone(self.graphics.as_ref().unwrap())
    }

    pub fn clone_audio(&self) -> Arc<RwLock<Audio>> {
        Arc::clone(self.audio.as_ref().unwrap())
    }

    pub fn clone_proxy(&self) -> EventLoopProxy<UserEvent> {
        self.proxy.clone()
    }

    fn try_init_emulator<E: Emulator>(&mut self, model: E::Model) {
        println!("Loading ROM.");
        let file_dialog = E::FILE_FILTERS.iter().fold(rfd::FileDialog::new(), |a, elem| a.add_filter(elem.0, elem.1));
        match file_dialog.pick_file() {
            Some(path) => match E::try_init(model, &path, &self) {
                // If a file was selected, try to initialize Emulator
                Ok(emu) => {
                    // If Emulator construction succeeds, close current emulator (if any) and save communication channel to app state
                    self.stop_emulation();
                    println!("Successfully loaded {}. Launching {}.", path.file_name().unwrap().display(), E::CORE_NAME);
                    // println!("{} - {} FPS", path.file_prefix().unwrap().display(), 0);
                    self.reset_frame_counter();
                    self.emulator = Some(emu);
                }
                Err(e) => {
                    // If Emulator construction fails, show an error message
                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Error)
                        .set_buttons(rfd::MessageButtons::Ok)
                        .set_title("Error Initializing Emulator")
                        .set_description(e.to_string())
                        .show();
                }
            },
            None => {} // No file selected -- do nothing
        };
    }

    fn stop_emulation(&mut self) {
        if let Some(emulator) = self.emulator.take() {
            emulator.send(EmuMessage::Stop);
        }
    }

    fn reset_frame_counter(&mut self) {
        self.last_second = std::time::Instant::now() - std::time::Duration::from_secs(1);
        self.tick_frame_counter();
    }

    fn tick_frame_counter(&mut self) {
        if let Some(_) = self.emulator {
            self.window.as_ref().unwrap().set_title(&("Hydra - ".to_string() + &self.frame_counter.to_string() + " FPS"));
            self.frame_counter = 0;
            self.last_second += std::time::Duration::from_secs(1);
        }
    }
}

impl ApplicationHandler<UserEvent> for HydraApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Resuming");
        if let None = self.window {
            self.init_app(event_loop);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.sdl.tick();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(graphics) = &self.graphics {
                    graphics.read().unwrap().render();
                    self.frame_counter += 1;
                    let diff = self.last_second.elapsed().as_secs_f64();
                    if diff > 1.0 {
                        self.tick_frame_counter();
                    }
                }
            }
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                if let Some(emu) = &self.emulator {
                    emu.send(EmuMessage::KeyboardInput(event));
                }
            }
            WindowEvent::Resized(_) => {
                if let Some(graphics) = &self.graphics {
                    graphics.read().unwrap().resize();
                }
            }
            _ => (),
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        macro_rules! emu_message_arm {
            ($message:expr) => {
                {self.emulator.as_ref().inspect(|e| e.send($message).unwrap());}
            };
        }
        match event {
            UserEvent::MenuEvent(e) => {
                match e.id.0.as_str() {
                    "load_rom" => self.try_init_emulator::<AllEmulator>(()),
                    "load_gb" => self.try_init_emulator::<GameBoy<Dmg>>(self.config.gb.default_models.dmg),
                    "load_sgb" => self.try_init_emulator::<GameBoy<Sgb>>(self.config.gb.default_models.sgb),
                    "load_gbc" => self.try_init_emulator::<GameBoy<Cgb>>(self.config.gb.default_models.cgb),
                    "load_gba" => self.try_init_emulator::<GameBoyAdvance>(gba::GbaTarget::Gb(self.config.gb.default_models.agb)),
                    "load_nds" => self.try_init_emulator::<Nds>(()),
                    "load_n3ds" => self.try_init_emulator::<N3ds>(()),

                    "save_state_1" => emu_message_arm!(EmuMessage::SaveStateSlot(1)),
                    "save_state_2" => emu_message_arm!(EmuMessage::SaveStateSlot(2)),
                    "save_state_3" => emu_message_arm!(EmuMessage::SaveStateSlot(3)),
                    "save_state_4" => emu_message_arm!(EmuMessage::SaveStateSlot(4)),
                    "save_state_5" => emu_message_arm!(EmuMessage::SaveStateSlot(5)),
                    "save_state_6" => emu_message_arm!(EmuMessage::SaveStateSlot(6)),
                    "save_state_7" => emu_message_arm!(EmuMessage::SaveStateSlot(7)),
                    "save_state_8" => emu_message_arm!(EmuMessage::SaveStateSlot(8)),
                    "save_state_9" => emu_message_arm!(EmuMessage::SaveStateSlot(9)),
                    "save_state_10" => emu_message_arm!(EmuMessage::SaveStateSlot(10)),

                    "load_state_1" => emu_message_arm!(EmuMessage::LoadStateSlot(1)),
                    "load_state_2" => emu_message_arm!(EmuMessage::LoadStateSlot(2)),
                    "load_state_3" => emu_message_arm!(EmuMessage::LoadStateSlot(3)),
                    "load_state_4" => emu_message_arm!(EmuMessage::LoadStateSlot(4)),
                    "load_state_5" => emu_message_arm!(EmuMessage::LoadStateSlot(5)),
                    "load_state_6" => emu_message_arm!(EmuMessage::LoadStateSlot(6)),
                    "load_state_7" => emu_message_arm!(EmuMessage::LoadStateSlot(7)),
                    "load_state_8" => emu_message_arm!(EmuMessage::LoadStateSlot(8)),
                    "load_state_9" => emu_message_arm!(EmuMessage::LoadStateSlot(9)),
                    "load_state_10" => emu_message_arm!(EmuMessage::LoadStateSlot(10)),

                    "pause_emulation" => {
                        if let Some(ref emulator) = self.emulator {
                            emulator.send(EmuMessage::Pause).unwrap();
                            self.window.as_ref().unwrap().set_title("Hydra - Paused");
                        }
                    }

                    "reset_emulation" => {
                        if let Some(ref emulator) = self.emulator {
                            emulator.send(EmuMessage::Reset).unwrap();
                        }
                    }

                    "stop_emulation" => {
                        self.stop_emulation();
                        self.graphics.as_mut().unwrap().write().unwrap().clear_screen_texture();
                        self.window.as_ref().unwrap().set_title("Hydra");
                    }

                    "bug_report" => {
                        webbrowser::open("https://github.com/Maelstrom16/Hydra/issues/new");
                    }
                    _ => {}
                }
            }
            UserEvent::RedrawRequest => self.window.as_ref().unwrap().request_redraw(),
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        println!("Thank you for supporting Hydra <3");
        self.config.write_to_toml();
    }
}

#[derive(Debug)]
pub enum UserEvent {
    MenuEvent(MenuEvent),
    RedrawRequest
}