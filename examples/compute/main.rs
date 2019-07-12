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
    window.set_title("compute");

    let v = AppView::new(window);
    // let mut triangle = idroid::Triangle::new(v);
    let mut fluid = idroid::fluid::LBMFlow::new(v);

    fluid.enter_frame();
}
