use glium::glutin::{window::WindowBuilder, ContextBuilder};
use glium::glutin::event_loop::{EventLoop, ControlFlow};
use glium::glutin::event::{Event, WindowEvent, ElementState, MouseButton};
use glium::{Display, Surface};

use std::collections::HashMap;

// This example shows how to dynamically open new windows.

fn main() {
    let event_loop = EventLoop::new();

    let wb = WindowBuilder::new()
        .with_title("Test main");

    let cb = ContextBuilder::new();

    let display = Display::new(wb, cb, &event_loop).unwrap();

    let mut counter = 0;
    let mut extra_windows = HashMap::new();

    event_loop.run(move |ev, event_loop, cf| {
        match ev {
            Event::WindowEvent { event: WindowEvent::CloseRequested, window_id } => {
                if window_id == display.gl_window().window().id() {
                    // Close the application when main window is closed
                    *cf = ControlFlow::Exit;
                } else {
                    // For other windows we only close the specified window
                    extra_windows.remove(&window_id);
                }
            }
            // Clicking on the main window spawns new windows
            Event::WindowEvent { event: WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            }, window_id } if window_id == display.gl_window().window().id() => {
                counter += 1;

                let wb = WindowBuilder::new()
                    .with_title(&format!("Test {}", counter));

                let cb = ContextBuilder::new();

                let extra_display = Display::new(wb, cb, &event_loop).unwrap();

                let id = extra_display.gl_window().window().id();
                extra_windows.insert(id, extra_display);
            }
            Event::RedrawRequested(id) => {
                if id == display.gl_window().window().id() {
                    let mut target = display.draw();
                    // We draw the main window in red
                    target.clear_color(1.0, 0.0, 0.0, 1.0);
                    target.finish().unwrap();
                } else if let Some(extra_display) = extra_windows.get(&id) {
                    let mut target = extra_display.draw();
                    // The other windows are drawn in blue
                    target.clear_color(0.0, 0.0, 1.0, 1.0);
                    target.finish().unwrap();
                }
            },
            _ => (),
        }
    });
}

