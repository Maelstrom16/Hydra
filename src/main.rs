mod graphics;
mod window;

use muda::MenuEvent;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::window::{HydraApp, UserEvent};


pub fn main() {
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
}