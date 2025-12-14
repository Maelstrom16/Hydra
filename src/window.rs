use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use muda::MenuEvent;
use muda::accelerator::{Accelerator, Code, Modifiers};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::macos::WindowAttributesExtMacOS;
use winit::window::{Window, WindowId};

use crate::common::emulator::{self, EmuMessage};
use crate::common::errors::HydraIOError;
use crate::config::Config;
use crate::gameboy;
use crate::graphics::Graphics;
use crate::ui::UserInterface;

const GB_FILE_FILTER: (&str, &[&str]) = ("Game Boy (Color)", &["gb", "gbc"]);

pub struct HydraApp {
    config: Config,
    window: Option<Arc<Window>>,
    graphics: Option<Arc<RwLock<Graphics>>>,
    ui: Option<UserInterface>,
    proxy: EventLoopProxy<UserEvent>,

    emulator: Option<Sender<EmuMessage>>,

    _temp_counter: u64,
    _temp_time: std::time::Instant,
}

impl HydraApp {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        HydraApp {
            config: Config::from_toml(),
            window: None, // Initialized on app startup
            graphics: None, // Initialized on app startup
            ui: None, // Initialized on app startup
            proxy,

            emulator: None, // Initialized when opening a ROM

            _temp_counter: 0,
            _temp_time: std::time::Instant::now(),
        }
    }

    fn init_app(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Hydra");
        self.window = Some(Arc::new(event_loop.create_window(window_attributes).unwrap()));
        self.graphics = Some(Arc::new(RwLock::new(futures::executor::block_on(Graphics::new(self.window.clone().unwrap(), None)))));
        self.ui = Some(UserInterface::from_config(&self.config));
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn get_window(&self) -> Arc<Window> {
        Arc::clone(self.window.as_ref().unwrap())
    }

    pub fn get_graphics(&self) -> Arc<RwLock<Graphics>> {
        Arc::clone(self.graphics.as_ref().unwrap())
    }

    pub fn get_proxy(&self) -> EventLoopProxy<UserEvent> {
        self.proxy.clone()
    }

    fn try_init_emulator<F>(&mut self, filters: &[(&str, &[&str])], func: F)
    where
        F: Fn(&PathBuf, &HydraApp) -> Result<Sender<EmuMessage>, HydraIOError>,
    {
        println!("Loading ROM.");
        let file_dialog = filters.iter().fold(rfd::FileDialog::new(), |a, elem| a.add_filter(elem.0, elem.1));
        match file_dialog.pick_file() {
            Some(path) => match func(&path, &self) {
                // If a file was selected, try to initialize Emulator
                Ok(emu) => {
                    // If Emulator construction succeeds, save communication channel to app state
                    println!("Successfully loaded {}. Launching emulator.", path.file_name().unwrap().display());
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

    fn try_init_generic(&mut self) {
        self.try_init_emulator(&[GB_FILE_FILTER], |path, this| {
            emulator::init_from_file(path, this)
        })
    }

    fn try_init_gameboy(&mut self, model: gameboy::Model) {
        self.try_init_emulator(&[GB_FILE_FILTER], |path, this| {
            gameboy::GameBoy::from_model(path, model, this)
        })
    }
}

impl ApplicationHandler<UserEvent> for HydraApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Resuming");
        if let None = self.window {
            self.init_app(event_loop);
        }
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
                    self._temp_counter += 1;
                    let now = std::time::Instant::now();
                    let diff = (now - self._temp_time).as_secs_f64();
                    if diff > 1.0 {
                        println!("{} frames, {} fps", self._temp_counter, self._temp_counter as f64 / diff);
                        self._temp_counter = 0;
                        self._temp_time = now;
                    }
                }
            }
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => match event.state {
                ElementState::Pressed => match event.physical_key {
                    _ => println!("{:?}", event.physical_key),
                },
                ElementState::Released => {}
            },
            WindowEvent::Resized(_) => {
                if let Some(graphics) = &self.graphics {
                    graphics.read().unwrap().resize();
                }
            }
            _ => (),
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::MenuEvent(e) => {
                match e.id.0.as_str() {
                    "load_rom" => self.try_init_generic(),
                    "load_gb" => self.try_init_gameboy(gameboy::Model::GameBoy(None)),
                    "load_gb_dmg0" => self.try_init_gameboy(gameboy::Model::GameBoy(Some(gameboy::GBRevision::DMG0))),
                    "load_gb_dmg" => self.try_init_gameboy(gameboy::Model::GameBoy(Some(gameboy::GBRevision::DMG0))),
                    "load_gb_mgb" => self.try_init_gameboy(gameboy::Model::GameBoy(Some(gameboy::GBRevision::DMG0))),
                    "load_sgb" => self.try_init_gameboy(gameboy::Model::SuperGameBoy(None)),
                    "load_sgb_sgb" => self.try_init_gameboy(gameboy::Model::SuperGameBoy(Some(gameboy::SGBRevision::SGB))),
                    "load_sgb_sgb2" => self.try_init_gameboy(gameboy::Model::SuperGameBoy(Some(gameboy::SGBRevision::SGB2))),
                    "load_gbc" => self.try_init_gameboy(gameboy::Model::GameBoyColor(None)),
                    "load_gbc_cgb0" => self.try_init_gameboy(gameboy::Model::GameBoyColor(Some(gameboy::CGBRevision::CGB0))),
                    "load_gbc_cgb" => self.try_init_gameboy(gameboy::Model::GameBoyColor(Some(gameboy::CGBRevision::CGB))),
                    // "load_gba" Will be implemented later
                    "toggle_revisions" => self.ui.as_ref().unwrap().toggle_revisions(&mut self.config),

                    "stop_emulation" => self.emulator.as_ref().unwrap().send(EmuMessage::Stop).unwrap(),
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
