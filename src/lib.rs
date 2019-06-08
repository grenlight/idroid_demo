use std::ffi::{CStr, CString};
use std::os::raw::c_char;

extern crate libc;

pub use uni_view::*;

mod depth_stencil;
mod geometry;
mod math;
mod matrix_helper;
mod texture;
mod utils;
mod vertex;

// #[cfg(not(target_os = "ios"))]
mod shader;

mod triangle;
pub use triangle::Triangle;

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn create_triangle(view: uni_view::AppViewObj) -> *mut libc::c_void {
    let rust_view = uni_view::AppView::new(view);
    let obj = Triangle::new(rust_view);
    let boxed_trait: Box<dyn SurfaceView> = Box::new(obj);
    let boxed_boxed_trait = Box::new(boxed_trait);
    let heap_pointer = Box::into_raw(boxed_boxed_trait);

    heap_pointer as *mut libc::c_void
}

#[cfg(target_os = "ios")]
#[no_mangle]
pub unsafe extern "C" fn enter_frame(obj: *mut libc::c_void) -> *mut libc::c_void {
    let mut obj: Box<Box<dyn SurfaceView>> = Box::from_raw(obj as *mut _);
    obj.enter_frame();

    // 重新将所有权移出
    Box::into_raw(obj) as *mut libc::c_void
}
