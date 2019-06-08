extern crate libc;
extern crate wgpu;

use std::ops::Deref;

#[cfg(target_os = "macos")]
#[path = "mac_view.rs"]
mod app_view;

#[cfg(not(target_os = "macos"))]
#[path = "ios_view.rs"]
mod app_view;

pub use app_view::*;

#[cfg(target_os = "ios")]
#[path = "ios_fs.rs"]
pub mod fs ;

#[cfg(not(target_os = "ios"))]
#[path = "mac_fs.rs"]
pub mod fs ;


mod ffi;


#[repr(C)]
pub struct ViewSize {
    pub width: u32,
    pub height: u32,
}

pub trait GPUContext {
    fn get_view_size(&self) -> ViewSize;
    fn update_swap_chain(&mut self);
}

pub trait SurfaceView {
    fn resize(&mut self);
    fn render(&mut self);

    fn get_swap_chain(&mut self) -> &mut wgpu::SwapChain;

    fn enter_frame(&mut self);
}