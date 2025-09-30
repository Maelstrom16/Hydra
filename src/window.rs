use std::sync::Arc;

use muda::accelerator::{Accelerator, Code, Modifiers};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::macos::WindowAttributesExtMacOS;
use winit::window::{Window, WindowId};
use muda::{AboutMetadata, AboutMetadataBuilder, CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};

use crate::wgpu::{self, Graphics};

#[derive(Default)]
pub struct HydraApp {
    window: Option<Arc<Window>>,
    menu: Option<Menu>,
    graphics: Option<Graphics>
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
                    graphics.render_start();
                }
            },
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                match event.state {
                    ElementState::Pressed => println!("{:?}", event.physical_key),
                    ElementState::Released => {}
                }
            },
            _ => ()
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::MenuEvent(e) => match e.id.0.as_str() {
                "load_rom" => {
                    println!("Loading ROM.");
                    self.graphics = Some(futures::executor::block_on(wgpu::Graphics::new(self.window.clone().unwrap())));
                    self.window.as_ref().unwrap().request_redraw();
                },
                _ => {}
            }
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        println!("Thank you for supporting Hydra <3")
    }
}

pub enum UserEvent {
    MenuEvent(MenuEvent)
}