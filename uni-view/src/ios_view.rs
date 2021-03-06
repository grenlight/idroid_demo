use libc::c_void;
use std::marker::{Send, Sync};

extern crate objc;
use self::objc::{*, runtime::{Class, Object}, rc::StrongPtr};
extern crate core_graphics;
use self::core_graphics::{ geometry::CGRect, base::CGFloat };

#[repr(C)]
pub struct AppViewObj {
    pub view: *mut Object,
    pub metal_layer: *mut c_void,
    // pub get_inner_size: extern "C" fn() -> crate::ViewSize,
}

pub struct AppView {
    pub view: *mut Object,
    pub scale_factor: f64,
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
}

impl AppView {
    pub fn new(obj: AppViewObj) -> Self {
        let scale_factor = get_scale_factor(obj.view);
        let s: CGRect = unsafe { msg_send![obj.view, frame] };
        let physical = crate::ViewSize {
            width: (s.size.width * scale_factor) as u32,
            height: (s.size.height * scale_factor) as u32
        };
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: physical.width,
            height: physical.height,
        };
        let instance = wgpu::Instance::new();
        let device = get_device(&instance);
        let surface = instance.create_surface_with_metal_layer(obj.metal_layer);
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        AppView {
            view: obj.view,
            scale_factor,
            instance,
            device,
            surface,
            sc_desc,
            swap_chain,
        }
    }

    pub fn test(&self) {
        println!("AppView fn in rust");
    }
}

impl crate::GPUContext for AppView {
    fn update_swap_chain(&mut self) {
        let size = self.get_view_size();
        self.sc_desc.width = size.width;
        self.sc_desc.height = size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn get_view_size(&self) -> crate::ViewSize {
        let s: CGRect = unsafe { msg_send![self.view, frame] };
        crate::ViewSize {
            width: (s.size.width * self.scale_factor) as u32,
            height: (s.size.height * self.scale_factor) as u32
        }
    }   
}

fn get_scale_factor(obj: *mut Object) -> f64 {
    let s: CGFloat = unsafe { msg_send![obj, contentScaleFactor] };
    s as f64
}

fn get_device(instance: &wgpu::Instance) -> wgpu::Device {
        let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
            power_preference: wgpu::PowerPreference::LowPower,
        });
       adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        })
}

