use std::sync::Arc;

use muda::{
    AboutMetadataBuilder, CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code, Modifiers},
};
use rfd::FileDialog;
use winit::window::Window;

use crate::{
    config::Config,
    emulator::gameboy::{AGBRevision, CGBRevision, GBRevision, SGBRevision},
};

pub struct UserInterface {
    hydra_menu: Menu,
    file_submenu: Submenu,
    load_to_console_submenu: Submenu,
}

impl UserInterface {
    pub fn initialize(window: &Arc<Window>, config: &Config) -> Self {
        // Create the main menubar
        let menu = Menu::new();

        let about_menuitem = PredefinedMenuItem::about(
            None,
            Some(
                AboutMetadataBuilder::new()
                    .authors(Some(vec!["Programmed by Kohradon, with love ♥".to_owned()]))
                    .credits(Some("Programmed by Kohradon, with love ♥".to_owned()))
                    .version(Some("Hydra 0.0.1\n------------\nWyrm (GB) 0.1.0\nLemonshark (3DS) 0.0.1"))
                    .build(),
            ),
        );
        let about_submenu = Submenu::with_items("About", true, &[&about_menuitem, &PredefinedMenuItem::separator(), &PredefinedMenuItem::quit(None)]).unwrap();

        let load_to_console_submenu = Submenu::with_items(
            "Load ROM to Console",
            true,
            &[
                &MenuItem::with_id("load_gb", "Game Boy...", true, None),
                &MenuItem::with_id("load_sgb", "Super Game Boy...", false, None),
                &MenuItem::with_id("load_gbc", "Game Boy Color...", true, None),
                &MenuItem::with_id("load_gba", "Game Boy Advance...", true, None),
                &MenuItem::with_id("load_nds", "DS...", true, None),
                &MenuItem::with_id("load_3ds", "3DS...", true, None),
            ],
        )
        .unwrap();

        let file_submenu = Submenu::with_items(
            "File",
            true,
            &[
                &MenuItem::with_id("load_rom", "&Load ROM...", true, None),
                &load_to_console_submenu,
                &PredefinedMenuItem::separator(),
                &MenuItem::new("Save State", true, Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyS))),
                &MenuItem::new("Load State", true, None),
                &PredefinedMenuItem::separator(),
                &MenuItem::new("Reset", true, None),
                &MenuItem::with_id("stop_emulation", "Stop", true, None),
                &PredefinedMenuItem::separator(),
                &MenuItem::new("Exit", true, None),
            ],
        )
        .unwrap();

        let edit_submenu = Submenu::with_items(
            "Edit",
            true,
            &[&MenuItem::new("Cut", true, None), &MenuItem::new("Copy", true, None), &MenuItem::new("Paste", true, None)],
        )
        .unwrap();

        let gameboy_submenu = Submenu::with_items(
            "Game Boy",
            true,
            &[&Submenu::with_items(
                "Default Models",
                true,
                &[
                    &MenuItem::new("Game Boy", false, None),
                    &CheckMenuItem::new("DMG0", true, config.gb.default_models.dmg == GBRevision::DMG0, None),
                    &CheckMenuItem::new("DMG", true, config.gb.default_models.dmg == GBRevision::DMG, None),
                    &CheckMenuItem::new("MGB", true, config.gb.default_models.dmg == GBRevision::MGB, None),
                    &PredefinedMenuItem::separator(),
                    &MenuItem::new("Super Game Boy", false, None),
                    &CheckMenuItem::new("SGB", true, config.gb.default_models.sgb == SGBRevision::SGB, None),
                    &CheckMenuItem::new("SGB2", true, config.gb.default_models.sgb == SGBRevision::SGB2, None),
                    &PredefinedMenuItem::separator(),
                    &MenuItem::new("Game Boy Color", false, None),
                    &CheckMenuItem::new("CGB0", true, config.gb.default_models.cgb == CGBRevision::CGB0, None),
                    &CheckMenuItem::new("CGB", true, config.gb.default_models.cgb == CGBRevision::CGB, None),
                    &PredefinedMenuItem::separator(),
                    &MenuItem::new("Game Boy Advance", false, None),
                    &CheckMenuItem::new("AGB0", true, config.gb.default_models.agb == AGBRevision::AGB0, None),
                    &CheckMenuItem::new("AGB", true, config.gb.default_models.agb == AGBRevision::AGB, None),
                    &PredefinedMenuItem::separator(),
                ],
            )
            .unwrap()],
        )
        .unwrap();

        let gba_submenu = Submenu::with_items(
            "GBA",
            false,
            &[],
        )
        .unwrap();

        let nds_submenu = Submenu::with_items(
            "DS",
            false,
            &[],
        )
        .unwrap();

        let n3ds_submenu = Submenu::with_items(
            "3DS",
            false,
            &[],
        )
        .unwrap();

        menu.append_items(&[&about_submenu, &file_submenu, &gameboy_submenu, &gba_submenu, &nds_submenu, &n3ds_submenu]).unwrap();

        apply_to_window(&menu, window);

        UserInterface {
            hydra_menu: menu,
            file_submenu,
            load_to_console_submenu,
        }
    }
}

fn apply_to_window(menu: &Menu, window: &Arc<Window>) {
    #[cfg(target_os = "windows")]
    unsafe {
        use wgpu::rwh::{HasWindowHandle, RawWindowHandle};
        match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::Win32(handle) => menu.init_for_hwnd(handle.hwnd.into()),
            _ => panic!("Initialized non-WIN32 window on Windows platform")
        }
    };
    #[cfg(target_os = "linux")]
    menu.init_for_gtk_window(&gtk_window, Some(&vertical_gtk_box));
    #[cfg(target_os = "macos")]
    menu.init_for_nsapp();
}
