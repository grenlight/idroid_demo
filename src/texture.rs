use wgpu::{Extent3d, Sampler, TextureView};

#[allow(dead_code)]
pub fn from_file(
    image_name: &str, device: &mut wgpu::Device, encoder: &mut wgpu::CommandEncoder,
) -> (TextureView, Extent3d, Sampler) {
    // 动态加载本地文件
    let path = uni_view::fs::FileSystem::get_texture_file_path(image_name);

    let image_bytes = match std::fs::read(&path) {
        Ok(code) => code,
        Err(e) => panic!("Unable to read {:?}: {:?}", path, e),
    };

    let img = image::load_from_memory(&image_bytes)
        .expect("Failed to load image.")
        .to_rgba();
    let (width, height) = img.dimensions();
    let texture_extent = wgpu::Extent3d {
        width: width,
        height: height,
        depth: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::TRANSFER_DST,
    });
    let texture_view = texture.create_default_view();

    let texels: Vec<u8> = img.into_raw();
    let temp_buf = device
        .create_buffer_mapped(texels.len(), wgpu::BufferUsage::TRANSFER_SRC)
        .fill_from_slice(&texels);
    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &temp_buf,
            offset: 0,
            row_pitch: 4 * width,
            image_height: height,
        },
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
        texture_extent,
    );

    (texture_view, texture_extent, default_sampler(device))
}

// empty texture as a OUTPUT_ATTACHMENT
#[allow(dead_code)]
pub fn empty(
    device: &mut wgpu::Device, format: wgpu::TextureFormat, extent: Extent3d,
) -> TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: format,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
    });
    let texture_view = texture.create_default_view();
    texture_view
}

#[allow(dead_code)]
pub fn empty_view(device: &mut wgpu::Device, width: u32, height: u32) -> TextureView {
    crate::texture::empty(
        device,
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::Extent3d {
            width: width,
            height: height,
            depth: 1,
        },
    )
}

pub fn default_sampler(device: &wgpu::Device) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        lod_min_clamp: -100.0,
        lod_max_clamp: 100.0,
        compare_function: wgpu::CompareFunction::Always,
    })
}
