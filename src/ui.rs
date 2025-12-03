use muda::{
    AboutMetadataBuilder, CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code, Modifiers},
};
use rfd::FileDialog;

use crate::{
    config::Config,
    gameboy::{AGBRevision, CGBRevision, GBRevision, SGBRevision},
};

pub struct UserInterface {
    hydra_menu: Menu,
    file_submenu: Submenu,
    load_to_console_submenu_abridged: Submenu,
    load_to_console_submenu_full: Submenu,
}

impl UserInterface {
    pub fn from_config(config: &Config) -> Self {
        // Create the main menubar
        let menu = Menu::new();

        let about_menuitem = PredefinedMenuItem::about(
            None,
            Some(
                AboutMetadataBuilder::new()
                    .authors(Some(vec!["Programmed by Kohradon, with love ♥".to_owned()]))
                    .credits(Some("Programmed by Kohradon, with love ♥".to_owned()))
                    .version(Some("Hydra 0.0.1\n------------\nWyrm (GB) 0.0.1\nLindwyrm (GBA) 0.0.0"))
                    .build(),
            ),
        );
        let about_submenu = Submenu::with_items("About", true, &[&about_menuitem, &PredefinedMenuItem::separator(), &PredefinedMenuItem::quit(None)]).unwrap();

        let toggle_revisions_checkmenuitem = CheckMenuItem::with_id("toggle_revisions", "Show All Revisions", true, false, None);
        let load_to_console_submenu_abridged = Submenu::with_items(
            "Load ROM to Console",
            true,
            &[
                &MenuItem::with_id("load_gb", "Game Boy...", true, None),
                &MenuItem::with_id("load_sgb", "Super Game Boy...", false, None),
                &MenuItem::with_id("load_gbc", "Game Boy Color...", true, None),
                &MenuItem::with_id("load_gba", "Game Boy Advance...", true, None),
                &toggle_revisions_checkmenuitem,
            ],
        )
        .unwrap();

        let load_to_console_submenu_full = Submenu::with_items(
            "Load ROM to Console",
            true,
            &[
                &MenuItem::with_id("load_gb_dmg0", "Game Boy (DMG0)...", true, None),
                &MenuItem::with_id("load_gb_dmg", "Game Boy (DMG)...", true, None),
                &MenuItem::with_id("load_gb_mgb", "Game Boy Pocket...", true, None),
                &MenuItem::with_id("load_sgb_sgb", "Super Game Boy...", false, None),
                &MenuItem::with_id("load_sgb_sgb2", "Super Game Boy 2...", false, None),
                &MenuItem::with_id("load_gbc_cgb0", "Game Boy Color (CGB0)...", true, None),
                &MenuItem::with_id("load_gbc_cgb", "Game Boy Color (CGB)...", true, None),
                &MenuItem::with_id("load_gba_agb0", "Game Boy Advance (AGB0)...", true, None),
                &MenuItem::with_id("load_gba_agb", "Game Boy Advance (AGB)...", true, None),
                &toggle_revisions_checkmenuitem,
            ],
        )
        .unwrap();

        let file_submenu = Submenu::with_items(
            "File",
            true,
            &[
                &MenuItem::with_id("load_rom", "&Load ROM...", true, None),
                &load_to_console_submenu_abridged,
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

        menu.append_items(&[&about_submenu, &file_submenu, &gameboy_submenu]).unwrap();

        init_for_window(&menu);

        UserInterface {
            hydra_menu: menu,
            file_submenu,
            load_to_console_submenu_abridged,
            load_to_console_submenu_full,
        }
    }

    pub fn toggle_revisions(&self, config: &mut Config) {
        if config.gb.show_all_revisions {
            self.file_submenu.remove(&self.load_to_console_submenu_full);
            self.file_submenu.insert(&self.load_to_console_submenu_abridged, 1);
        } else {
            self.file_submenu.remove(&self.load_to_console_submenu_abridged);
            self.file_submenu.insert(&self.load_to_console_submenu_full, 1);
        }
        config.gb.show_all_revisions = !config.gb.show_all_revisions;
    }
}

fn init_for_window(menu: &Menu) {
    #[cfg(target_os = "windows")]
    unsafe {
        menu.init_for_hwnd(window_hwnd)
    };
    #[cfg(target_os = "linux")]
    menu.init_for_gtk_window(&gtk_window, Some(&vertical_gtk_box));
    #[cfg(target_os = "macos")]
    menu.init_for_nsapp();
}
