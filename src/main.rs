mod common;
mod config;
mod gameboy;
mod graphics;
mod window;

use std::fs;

use muda::MenuEvent;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::{
    common::errors::{self, HydraIOError},
    config::Config,
    window::{HydraApp, UserEvent},
};

const CONFIG_PATH: &str = "config.toml";

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&String::from("-i")) {
        // If initialize flag is set, delete config file (to be reset below).
        println!("Resetting config.toml.");
        if let Err(e) = std::fs::remove_file(CONFIG_PATH) {
            println!(
                "Failed to delete config.toml: {}\nProgram will continue using old configurations.",
                e
            );
        }
    }
    let config = propagate_or!(
        Ok(toml::from_slice::<Config>(
            std::fs::read(CONFIG_PATH)?.as_slice()
        )?),
        Config::default()
    );

    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();

    // Forward muda::MenuEvent to winit::EventLoop
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        proxy.send_event(UserEvent::MenuEvent(event));
    }));

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = HydraApp::default();
    event_loop.run_app(&mut app);

    if let Err(e) = propagate!(Ok(fs::write(
        CONFIG_PATH,
        toml::to_string_pretty(&config)?
    )?)) {
        println!(
            "Failed to save config.toml: {}\nProgram will continue using old configurations.",
            e
        );
    }
}
