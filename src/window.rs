use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use muda::accelerator::{Accelerator, Code, Modifiers};
use muda::{AboutMetadata, AboutMetadataBuilder, CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use rand::Rng;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::macos::WindowAttributesExtMacOS;
use winit::window::{Window, WindowId};

use crate::common::emulator::{self, Emulator};
use crate::common::errors::HydraIOError;
use crate::config::Config;
use crate::{gameboy, graphics};
use crate::graphics::Graphics;
use crate::ui::UserInterface;

#[derive(Default)]
pub struct HydraApp {
    config: Option<Config>,
    emulator: Option<Arc<dyn Emulator>>,
    window: Option<Arc<Window>>,
    ui: Option<UserInterface>,
    graphics: Option<Arc<RwLock<Graphics>>>,

    _temp_counter: u64,
    _temp_time: Option<std::time::Instant>,
}

impl HydraApp {
    fn init_app(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Hydra");
        self.window = Some(Arc::new(event_loop.create_window(window_attributes).unwrap()));
        self.graphics = Some(Arc::new(RwLock::new(futures::executor::block_on(Graphics::new(
            self.window.clone().unwrap(),
            None,
        )))));

        self.config = Some(Config::from_toml());
        self.ui = Some(UserInterface::from_config(self.config.as_ref().unwrap()));
        self._temp_time = Some(std::time::Instant::now());
    }

    fn try_init_emulator<F>(&mut self, filters: &[(&str, &[&str])], func: F)
    where
        F: Fn(&PathBuf, Arc<Window>, Arc<RwLock<Graphics>>, &Config) -> Result<Arc<dyn Emulator>, HydraIOError>,
    {
        println!("Loading ROM.");
        let file_dialog = filters.iter().fold(rfd::FileDialog::new(), |a, elem| a.add_filter(elem.0, elem.1));
        match file_dialog.pick_file() {
            Some(path) => match func(&path, Arc::clone(self.window.as_ref().unwrap()), Arc::clone(self.graphics.as_ref().unwrap()), self.config.as_ref().unwrap()) {
                // If a file was selected, try to initialize Emulator
                Ok(mut emu) => {
                    // If Emulator construction succeeds, save to app state and launch it
                    println!("Successfully loaded {}. Launching emulator.", path.file_name().unwrap().display());
                    Arc::clone(&emu).main_thread();
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

    fn try_init_gameboy(&mut self, model: gameboy::Model) {
        self.try_init_emulator(&[("Game Boy (Color)", &["gb", "gbc"])], |path, window, graphics, config| {
            gameboy::GameBoy::from_model(path, model, window, graphics, config).and_then(|emu| Ok(Arc::new(emu) as Arc<dyn Emulator>))
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
                self.graphics.as_ref().unwrap().read().unwrap().render();
                
                if let Some(graphics) = &self.graphics {
                    self._temp_counter += 1;
                    let now = std::time::Instant::now();
                    let diff = (now - self._temp_time.unwrap()).as_secs_f64();
                    if diff > 1.0 {
                        println!("{} frames, {} fps", self._temp_counter, self._temp_counter as f64 / diff);
                        self._temp_counter = 0;
                        self._temp_time = Some(now);
                    }
                }
            }
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                match event.state {
                    ElementState::Pressed => match event.physical_key {
                        _ => println!("{:?}", event.physical_key),
                    },
                    ElementState::Released => {}
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
        match event {
            UserEvent::MenuEvent(e) => {
                match e.id.0.as_str() {
                    "load_rom" => self.try_init_emulator(&[("Game Boy (Color)", &["gb", "gbc"])], |path, window, graphics, config| emulator::init_from_file(path, window, graphics, config)),
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
                    "toggle_revisions" => self.ui.as_mut().unwrap().toggle_revisions(self.config.as_mut().unwrap()),
                    _ => {}
                }
            }
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        println!("Thank you for supporting Hydra <3");
        self.config.as_ref().unwrap().write_to_toml();
    }
}

pub enum UserEvent {
    MenuEvent(MenuEvent),
}
