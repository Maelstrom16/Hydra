mod audio;
mod common;
mod config;
mod emulator;
mod graphics;
mod input;
mod ui;
mod window;

use std::{path::Path, println, unimplemented};

use muda::MenuEvent;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::window::{HydraApp, UserEvent};

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|elem| elem == "--benchmark") && let Some(input) = args.get(i + 1) {
        benchmark(input);
    } else {
        launch();
    }
}

fn launch() {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();

    // Forward muda::MenuEvent to winit::EventLoop
    let proxy = event_loop.create_proxy();
    let menu_proxy = proxy.clone();
    MenuEvent::set_event_handler(Some(move |event| {
        menu_proxy.send_event(UserEvent::MenuEvent(event));
    }));

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = HydraApp::new(proxy);
    event_loop.run_app(&mut app);
}

/// Debug function used to test efficiency of other code
/// 
/// Will likely be moved into a separate module as additional, more complex tests are written
fn benchmark(input: &String) {
    struct TestCpu {
        rom: Vec<u8>,
        pc: usize
    }
    impl common::arm::ArmCpu for TestCpu {
        fn fetch_instruction(&mut self) -> common::arm::ArmAddress {
            let result = u32::from_le_bytes(self.rom[self.pc..self.pc+4].try_into().unwrap());
            self.pc += 4;
            return common::arm::ArmAddress::Inst32(result)
        }
        fn get_architecture(&self) -> &common::arm::ArmArchitecture {
            unimplemented!()
        }
    }
    let rom = std::fs::read(Path::new(input)).unwrap();
    let mut cpu = TestCpu{rom, pc: 0};
    
    println!("Decoding 32-bit ARM for {}. Length: {:#010X} bytes", input, cpu.rom.len());

    let durs: [f64; 5] = std::array::from_fn(|_| {
        cpu.pc = 0;
        let mut i = cpu.rom.len();

        let start_instant = std::time::Instant::now();
        while i > 0 {
            std::hint::black_box(common::arm::decode_instruction(std::hint::black_box(&mut cpu)));
            i -= 4;
        }
        start_instant.elapsed().as_secs_f64()
    });

    println!("\nDone in {} seconds.", durs.iter().sum::<f64>());
    println!("Best: {}", durs.into_iter().reduce(f64::min).unwrap_or(0.0));
    println!("Worst: {}", durs.into_iter().reduce(f64::max).unwrap_or(0.0));
    println!("Average: {}", durs.iter().sum::<f64>() / 5.0);
}
