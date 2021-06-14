use simple_logger::SimpleLogger;
use winit::{event::{Event, ModifiersState, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::{Menu, WindowBuilder, Hotkey}};

fn main() {
    SimpleLogger::new().init().unwrap();
    let event_loop = EventLoop::new();

    let mut menu = Menu::new();
    menu.add_item(0, "One", None);
    menu.add_item(1, "Two", None);
    menu.add_separator();
    
    let mut sub_menu = Menu::new();
    sub_menu.add_item(2, "Sub One", Hotkey::new(ModifiersState::ALT, winit::event::VirtualKeyCode::F1));
    sub_menu.add_item(3, "Sub Two", Hotkey::new(ModifiersState::ALT, winit::event::VirtualKeyCode::F2));
    sub_menu.add_separator();
    sub_menu.add_item(4, "Sub Three", Hotkey::new(ModifiersState::empty(), winit::event::VirtualKeyCode::F5));

    menu.add_dropdown("Three", sub_menu);

    let window = WindowBuilder::new()
        .with_menu(Some(menu))
        .with_title("A fantastic window!")
        .with_inner_size(winit::dpi::LogicalSize::new(300.0, 128.0))
        .build(&event_loop)
        .unwrap();

        window.request_redraw();

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
            Event::WindowEvent{
                event: WindowEvent::MenuEntryActivated(id),
                window_id,
            } if window_id == window.id() => {
                println!("Activated: {}", id);
            },
            _ => (),
        }
    });
}
