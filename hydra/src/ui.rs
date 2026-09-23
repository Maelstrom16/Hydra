use std::{fmt::Display, fs, sync::Arc};

use muda::{
    AboutMetadataBuilder, CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code, Modifiers},
};
use rfd::FileDialog;
use winit::window::Window;

use crate::{
    common::errors::HydraIOError, config::Config, emulator::gameboy::{AgbRevision, CgbRevision, DmgRevision, SgbRevision}, propagate, propagate_or,
};

pub struct UserInterface {
    hydra_menu: Menu,
    file_submenu: Submenu,
    load_to_console_submenu: Submenu,
}

impl UserInterface {
    const CONTROL_MODIFIER: Modifiers = if cfg!(target_os = "macos") {Modifiers::SUPER} else {Modifiers::CONTROL};

    pub fn initialize(window: &Arc<Window>, config: &Config) -> Self {
        // Create the main menubar
        let menu = Menu::new();

        let about_menuitem = PredefinedMenuItem::about(
            None,
            Some(
                AboutMetadataBuilder::new()
                    .authors(Some(vec!["Programmed by Kohradon, with love ♥".to_owned()]))
                    .credits(Some("Programmed by Kohradon, with love ♥".to_owned()))
                    .version(Some(propagate!(
                        toml::from_slice::<'_, toml::Value>(&fs::read("hydra/Cargo.toml")?)?
                        .get("package")
                            .and_then(|p| p.get("version"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unable to find version in Cargo.toml")
                            .to_owned())
                        .unwrap_or_else(|e| e.to_string())
                    ))
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
                &MenuItem::with_id("load_nds", "DS...", false, None),
                &MenuItem::with_id("load_3ds", "3DS...", false, None),
            ],
        )
        .unwrap();
    
        let save_state_submenu = Submenu::with_items(
            "Save State", 
            true, 
            &[
                &MenuItem::with_id("save_state_1", "Slot &1", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER), Code::Digit1))),
                &MenuItem::with_id("save_state_2", "Slot &2", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER), Code::Digit2))),
                &MenuItem::with_id("save_state_3", "Slot &3", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER), Code::Digit3))),
                &MenuItem::with_id("save_state_4", "Slot &4", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER), Code::Digit4))),
                &MenuItem::with_id("save_state_5", "Slot &5", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER), Code::Digit5))),
                &MenuItem::with_id("save_state_6", "Slot &6", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER), Code::Digit6))),
                &MenuItem::with_id("save_state_7", "Slot &7", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER), Code::Digit7))),
                &MenuItem::with_id("save_state_8", "Slot &8", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER), Code::Digit8))),
                &MenuItem::with_id("save_state_9", "Slot &9", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER), Code::Digit9))),
                &MenuItem::with_id("save_state_10", "Slot 1&0", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER), Code::Digit0))),
            ]
        )
        .unwrap();

        let load_state_submenu = Submenu::with_items(
            "Load State", 
            true, 
            &[
                &MenuItem::with_id("load_state_1", "Slot &1", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER | Modifiers::SHIFT), Code::Digit1))),
                &MenuItem::with_id("load_state_2", "Slot &2", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER | Modifiers::SHIFT), Code::Digit2))),
                &MenuItem::with_id("load_state_3", "Slot &3", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER | Modifiers::SHIFT), Code::Digit3))),
                &MenuItem::with_id("load_state_4", "Slot &4", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER | Modifiers::SHIFT), Code::Digit4))),
                &MenuItem::with_id("load_state_5", "Slot &5", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER | Modifiers::SHIFT), Code::Digit5))),
                &MenuItem::with_id("load_state_6", "Slot &6", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER | Modifiers::SHIFT), Code::Digit6))),
                &MenuItem::with_id("load_state_7", "Slot &7", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER | Modifiers::SHIFT), Code::Digit7))),
                &MenuItem::with_id("load_state_8", "Slot &8", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER | Modifiers::SHIFT), Code::Digit8))),
                &MenuItem::with_id("load_state_9", "Slot &9", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER | Modifiers::SHIFT), Code::Digit9))),
                &MenuItem::with_id("load_state_10", "Slot 1&0", true, Some(Accelerator::new(Some(Self::CONTROL_MODIFIER | Modifiers::SHIFT), Code::Digit0))),
            ]
        )
        .unwrap();

        let file_submenu = Submenu::with_items(
            "File",
            true,
            &[
                &MenuItem::with_id("load_rom", "&Load ROM...", true, None),
                &load_to_console_submenu,
                &PredefinedMenuItem::separator(),
                &save_state_submenu,
                &load_state_submenu,
                &MenuItem::with_id("pause_emulation", "Pause/Unpause", true, None),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id("reset_emulation","Reset", true, None),
                &MenuItem::with_id("stop_emulation", "Stop", true, None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
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
                    &CheckMenuItem::new("DMG0", true, config.gb.default_models.dmg == DmgRevision::DMG0, None),
                    &CheckMenuItem::new("DMG", true, config.gb.default_models.dmg == DmgRevision::DMG, None),
                    &CheckMenuItem::new("MGB", true, config.gb.default_models.dmg == DmgRevision::MGB, None),
                    &PredefinedMenuItem::separator(),
                    &MenuItem::new("Super Game Boy", false, None),
                    &CheckMenuItem::new("SGB", true, config.gb.default_models.sgb == SgbRevision::SGB, None),
                    &CheckMenuItem::new("SGB2", true, config.gb.default_models.sgb == SgbRevision::SGB2, None),
                    &PredefinedMenuItem::separator(),
                    &MenuItem::new("Game Boy Color", false, None),
                    &CheckMenuItem::new("CGB0", true, config.gb.default_models.cgb == CgbRevision::CGB0, None),
                    &CheckMenuItem::new("CGB", true, config.gb.default_models.cgb == CgbRevision::CGB, None),
                    &PredefinedMenuItem::separator(),
                    &MenuItem::new("Game Boy Advance", false, None),
                    &CheckMenuItem::new("AGB0", true, config.gb.default_models.agb == AgbRevision::AGB0, None),
                    &CheckMenuItem::new("AGB", true, config.gb.default_models.agb == AgbRevision::AGB, None),
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

        let help_submenu = Submenu::with_items(
            "Help", 
            true, 
            &[
                &MenuItem::with_id("bug_report", "Report a Bug", true, None)
            ]
        )
        .unwrap();

        menu.append_items(&[&about_submenu, &file_submenu, &help_submenu]).unwrap();

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
