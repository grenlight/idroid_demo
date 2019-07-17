use crate::SurfaceView;

use uni_view::{AppView, GPUContext};

pub struct LBMFlow {
    app_view: AppView,

    lattice_num_x: f32,
    lattice_num_y: f32,
    particle_num_x: f32,
    particle_num_y: f32,

    uniform_buf: wgpu::Buffer,
    fluid_buffer: wgpu::Buffer,
    staging_buffer: wgpu::Buffer,

    shader_collide: crate::shader::Shader,
    bind_group0: wgpu::BindGroup,
    collide_pipeline: wgpu::ComputePipeline,

    bd: wgpu::BufferDescriptor,
    swap: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FluidUniform {
    // e: [[f32; 2]; 9],
    e: [f32; 18],
    // lattice 在正规化坐标空间的大小
    lattice_size: [f32; 2],
    // 格子数
    lattice_num: [f32; 2],
    // weight: [f32; 9],
    // swap: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FluidCell {
    color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LatticeCell {
    cell: [f32; 9],
}

fn init_data() -> Vec<FluidCell> {
    let mut fluid: Vec<FluidCell> = vec![];
    for _ in 0..6 {
        fluid.push(FluidCell { color: [0.5, 0.0, 1.0] });
    }
    fluid
}

fn get_fluid_uniform(lattice_num_x: f32, lattice_num_y: f32, swap: i32) -> FluidUniform {
    let w0 = 4.0 / 9.0;
    let w1 = 1.0 / 9.0;
    let w2 = 1.0 / 36.0;
    // cell structure (subcell numbers):
    // 6 2 5
    // 3 0 1
    // 7 4 8
    let e: [f32; 18] = [
        0.0, 0.0, 1.0, 0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0,
        1.0,
    ];
    let weight: [f32; 9] = [w0, w1, w1, w1, w1, w2, w2, w2, w2];
    FluidUniform {
        e,
        lattice_size: [2.0 / lattice_num_x, 2.0 / lattice_num_y],
        lattice_num: [lattice_num_x, lattice_num_y],
        // weight,
        // swap,
    }
}

impl LBMFlow {
    pub fn new(app_view: AppView) -> Self {
        use std::mem;
        let mut app_view = app_view;
        let lattice_num_x = 4.0;
        let lattice_num_y = 4.0;
        let particle_num_x = 100.0;
        let particle_num_y = 100.0;

        let swap = 0_i32;

        let mut encoder =
            app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        // use to save FluidUniform e[18]'s value
        let fluid_buf_range = (6 * mem::size_of::<FluidCell>()) as wgpu::BufferAddress;

        let fluid_data = init_data();
        let (fluid_buffer, staging_buffer) = crate::utils::create_storage_buffer(
            &mut app_view.device,
            &mut encoder,
            &fluid_data,
            fluid_buf_range,
        );

        // let uniform_buf = crate::utils::create_uniform_buffer2(
        //     &mut app_view.device,
        //     &mut encoder,
        //     get_fluid_uniform(lattice_num_x, lattice_num_y, swap),
        //     128,
        // );
        // hold  BufferDescriptor
        
        let bd = wgpu::BufferDescriptor {
            size: mem::size_of::<FluidUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::TRANSFER_DST,
        };
        let uniform_buf = app_view.device.create_buffer(&bd);
        let sb = app_view
            .device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::TRANSFER_DST)
            .fill_from_slice(&[get_fluid_uniform(lattice_num_x, lattice_num_y, swap)]);
        encoder.copy_buffer_to_buffer(&sb, 0, &uniform_buf, 0, mem::size_of::<FluidUniform>() as wgpu::BufferAddress);

        // Create pipeline layout
        let bind_group_layout =
            app_view.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutBinding {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::UniformBuffer,
                    },
                    wgpu::BindGroupLayoutBinding {
                        binding: 1,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageBuffer,
                    },
                ],
            });

        let bind_group0 = app_view.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0..(std::mem::size_of::<FluidUniform>() as wgpu::BufferAddress),
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &fluid_buffer,
                        range: 0..fluid_buf_range,
                    },
                },
            ],
        });
        let pipeline_layout =
            app_view.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&bind_group_layout],
            });

        // Create the compute pipeline
        let shader_collide =
            crate::shader::Shader::new_by_compute("fluid/collide", &mut app_view.device);
        let collide_pipeline =
            app_view.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                layout: &pipeline_layout,
                compute_stage: shader_collide.cs_stage(),
            });

        app_view.device.get_queue().submit(&[encoder.finish()]);

        LBMFlow {
            app_view,
            lattice_num_x,
            lattice_num_y,
            particle_num_x,
            particle_num_y,

            uniform_buf,
            fluid_buffer,
            staging_buffer,

            shader_collide,
            bind_group0,
            collide_pipeline,

            bd,
            swap,
        }
    }
}

impl SurfaceView for LBMFlow {
    fn update(&mut self, _event: wgpu::winit::WindowEvent) {
        //empty
    }

    fn touch_moved(&mut self, _position: crate::math::Position) {}

    fn resize(&mut self) {
        self.app_view.update_swap_chain();
    }

    fn enter_frame(&mut self) {
        let mut encoder = self
            .app_view
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        {
            let mut cpass = encoder.begin_compute_pass();
            cpass.set_pipeline(&self.collide_pipeline);
            cpass.set_bind_group(0, &self.bind_group0, &[]);
            cpass.dispatch(6, 1, 1);
        }
        let fluid_buf_range = (6 * std::mem::size_of::<FluidCell>()) as wgpu::BufferAddress;

        encoder.copy_buffer_to_buffer(
            &self.fluid_buffer,
            0,
            &self.staging_buffer,
            0,
            fluid_buf_range,
        );

        self.app_view.device.get_queue().submit(&[encoder.finish()]);

        // print staging_buffer's value
        self.staging_buffer.map_read_async(
            0,
            fluid_buf_range,
            |result: wgpu::BufferMapAsyncResult<&[f32]>| {
                // data value should be: [0.0, 0.0, 1.0, 0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 1.0,
                // 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0,1.0,];
                println!("{:?}", result.unwrap().data);
            },
        );
    }
}
