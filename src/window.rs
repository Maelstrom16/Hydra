use std::sync::Arc;

use muda::accelerator::{Accelerator, Code, Modifiers};
use rand::Rng;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::macos::WindowAttributesExtMacOS;
use winit::window::{Window, WindowId};
use muda::{AboutMetadata, AboutMetadataBuilder, CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};

use crate::graphics::Graphics;

#[derive(Default)]
pub struct HydraApp {
    window: Option<Arc<Window>>,
    menu: Option<Menu>,
    graphics: Option<Graphics>,

    _temp_buffer: Vec<u8>,
    _temp_counter: u64,
    _temp_size: usize,
    _temp_time: Option<std::time::Instant>
}

impl HydraApp {
    fn init_window(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Hydra");
        self.window = Some(Arc::new(event_loop.create_window(window_attributes).unwrap()));
        // Create the main menubar
        let menu = Menu::new();

        let about_metadata: AboutMetadata = AboutMetadataBuilder::new()
            .authors(Some(vec!["Programmed by Kohradon, with love ♥".to_owned()]))
            .credits(Some("Programmed by Kohradon, with love ♥".to_owned()))
            .version(Some("Hydra 0.0.1\n------------\nWyrm (GB) 0.0.1\nLindwyrm (GBA) 0.0.0"))
            .build();
        let about_submenu = Submenu::with_items("About", true, &[
            &PredefinedMenuItem::about(None, Some(about_metadata)),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None)
        ]).unwrap();
        let file_submenu = Submenu::with_items("File", true, &[
            &MenuItem::with_id("load_rom", "&Load ROM...", true, None),
            &Submenu::with_items("Load ROM to Console", true, &[
                &MenuItem::new("Game Boy...", true, None),
                &MenuItem::new("Super Game Boy...", false, None),
                &MenuItem::new("Game Boy Color...", true, None),
                &MenuItem::new("Game Boy Advance...", true, None),
                &CheckMenuItem::new("Show All Revisions", true, false, None)
            ]).unwrap(),
            &PredefinedMenuItem::separator(),
            &MenuItem::new("Save State", true, Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyS))),
            &MenuItem::new("Load State", true, None),
            &PredefinedMenuItem::separator(),
            &MenuItem::new("Reset", true, None),
            &PredefinedMenuItem::separator(),
            &MenuItem::new("Exit", true, None)
        ]).unwrap();
        let edit_submenu = Submenu::with_items("Edit", true, &[
            &MenuItem::new("Cut", true, None),
            &MenuItem::new("Copy", true, None),
            &MenuItem::new("Paste", true, None)
        ]).unwrap();
        let gameboy_submenu = Submenu::with_items("Game Boy", true, &[
            &Submenu::with_items("Default Models", true, &[
                &MenuItem::new("Game Boy", false, None),
                &CheckMenuItem::new("DMG0", true, false, None),
                &CheckMenuItem::new("DMG", true, false, None),
                &CheckMenuItem::new("MGB", true, true, None),
                &PredefinedMenuItem::separator(),
                &MenuItem::new("Super Game Boy", false, None),
                &CheckMenuItem::new("SGB", true, false, None),
                &CheckMenuItem::new("SGB2", true, true, None),
                &PredefinedMenuItem::separator(),
                &MenuItem::new("Game Boy Color", false, None),
                &CheckMenuItem::new("CGB0", true, false, None),
                &CheckMenuItem::new("CGB", true, true, None),
                &PredefinedMenuItem::separator(),
                &MenuItem::new("Game Boy Advance", false, None),
                &CheckMenuItem::new("AGB0", true, false, None),
                &CheckMenuItem::new("AGB", true, true, None),
                &PredefinedMenuItem::separator()
            ]).unwrap()
        ]).unwrap();

        menu.append_items(&[&about_submenu, &file_submenu, &edit_submenu, &gameboy_submenu]).unwrap();
        #[cfg(target_os = "windows")]
        unsafe { menu.init_for_hwnd(window_hwnd) };
        #[cfg(target_os = "linux")]
        menu.init_for_gtk_window(&gtk_window, Some(&vertical_gtk_box));
        #[cfg(target_os = "macos")]
        menu.init_for_nsapp();

        self.menu = Some(menu);
        self._temp_time = Some(std::time::Instant::now());
    }
}

impl ApplicationHandler<UserEvent> for HydraApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Resuming");
        if let None = self.window {self.init_window(event_loop);}
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                if let Some(graphics) = &self.graphics {
                    // Test texture generation 
                    let upper_bound: usize = (self._temp_size*self._temp_size*4) as usize; //TODO: Remove when finished testing
                    for _ in 0..upper_bound/4 {
                        self._temp_buffer[rand::rng().random_range(0..upper_bound)] = 0;
                    }
                    // Update and render
                    graphics.update_screen_texture(self._temp_buffer.as_slice());
                    graphics.render();

                    self._temp_counter += 1;
                    let now = std::time::Instant::now();
                    let diff = (now - self._temp_time.unwrap()).as_secs_f64();
                    if diff > 1.0 {
                        println!("{} frames, {} fps", self._temp_counter, self._temp_counter as f64/diff);
                        self._temp_counter = 0;
                        self._temp_time = Some(now);
                    }
                }
            },
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                match event.state {
                    ElementState::Pressed => match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyA) => {
                            self._temp_size = 160;
                            self._temp_buffer = vec![255; self._temp_size*self._temp_size*4];
                            self.graphics.as_mut().unwrap().resize_screen_texture(self._temp_size as u32, self._temp_size as u32); //TODO: Remove when finished testing
                        },
                        _ => println!("{:?}", event.physical_key)
                    },
                    ElementState::Released => {}
                }
            },
            WindowEvent::Resized(_) => if let Some(graphics) = &self.graphics {
                graphics.resize();
            },
            _ => ()
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::MenuEvent(e) => match e.id.0.as_str() {
                "load_rom" => {
                    println!("Loading ROM.");
                    self._temp_size = 8;
                    self._temp_buffer = vec![255; self._temp_size*self._temp_size*4];
                    self.graphics = Some(futures::executor::block_on(Graphics::new(self.window.clone().unwrap(), Some(muda::dpi::PhysicalSize::<u32> {width: self._temp_size as u32, height: self._temp_size as u32})))); 
                    self.window.as_ref().unwrap().request_redraw();
                },
                _ => {}
            }
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        println!("Thank you for supporting Hydra <3");
    }
}

pub enum UserEvent {
    MenuEvent(MenuEvent)
}