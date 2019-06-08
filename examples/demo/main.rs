extern crate idroid;
use idroid::SurfaceView;

extern crate uni_view;
use uni_view::{AppView, ViewSize};

extern crate lazy_static;
use lazy_static::*;

extern crate objc;
use objc::*;

use log::info;
use std::time::{Duration, Instant};

lazy_static! {
    static ref instance: wgpu::Instance = wgpu::Instance::new();
}

fn main() {
    use wgpu::winit::{
        ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, Window, WindowEvent,
    };

    env_logger::init();

    let mut events_loop = EventsLoop::new();
    let window = Window::new(&events_loop).unwrap();
    window.set_max_dimensions(Some((400_u32, 700_u32).into()));
    window.set_title("title");

    // let screen_scale: fn() -> f32 = screen_scale;
    let v = AppView::new(window);
    // let mut triangle = idroid::Triangle::new(idroid::AppViewWrapper(v));
    let mut triangle = idroid::Triangle::new(v);

    let mut running = true;
    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();
    let fixed_time_stamp = Duration::new(0, 16666667);

    while running {
        events_loop.poll_events(|event| match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // let physical = size.to_physical(window.get_hidpi_factor());
                // println!("Resizing to {:?}", physical);
                triangle.resize();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    running = false;
                }
                WindowEvent::CursorMoved { position, .. } => {}
                _ => {}
            },
            _ => (),
        });

        let now = Instant::now();
        accumulator += now - previous_clock;

        if accumulator >= fixed_time_stamp {
            previous_clock = now;
            triangle.enter_frame();
        } else {
            std::thread::sleep(fixed_time_stamp - accumulator);
        }

        running &= !cfg!(feature = "metal-auto-capture");
    }
    // let triangle = idroid::Triangle::new()
}
