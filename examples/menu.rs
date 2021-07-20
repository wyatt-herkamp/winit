use simple_logger::SimpleLogger;
use winit::{
    event::{Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Hotkey, Menu, WindowBuilder},
};

fn main() {
    SimpleLogger::new().init().unwrap();
    let event_loop = EventLoop::new();

    let mut main = Menu::default();

    let mut first = Menu::new();
    first.add_item(
        0,
        "One",
        Hotkey::new(ModifiersState::CTRL, winit::event::VirtualKeyCode::Return),
    );

    main.add_dropdown("First", first);
    let window = WindowBuilder::new()
        .with_menu(Some(main))
        .with_title("A fantastic window (with a menu)!")
        .with_inner_size(winit::dpi::LogicalSize::new(300.0, 128.0))
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::MenuEntryActivated(id),
                ..
            } => {
                println!("Activated: {}", id);
            }
            _ => (),
        }
    });
}
