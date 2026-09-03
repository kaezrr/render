use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Mat4;
use glam::Quat;
use glam::Vec3;
use wgpu::Buffer;
use wgpu::VertexAttribute;
use wgpu::VertexBufferLayout;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy)]
pub struct Instance {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct InstanceRaw {
    model: [f32; 16],
}

impl From<&Instance> for InstanceRaw {
    fn from(value: &Instance) -> Self {
        Self {
            model: Mat4::from_scale_rotation_translation(
                value.scale,
                value.rotation,
                value.position,
            )
            .to_cols_array(),
        }
    }
}

impl InstanceRaw {
    const ATTRIBS: [VertexAttribute; 4] = wgpu::vertex_attr_array![
        5 => Float32x4,
        6 => Float32x4,
        7 => Float32x4,
        8 => Float32x4,
    ];

    pub const fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Debug)]
pub struct InstanceBundle {
    pub instances: Vec<Instance>,
    pub buffer: Buffer,
}

impl InstanceBundle {
    pub fn new(device: &wgpu::Device, instances: Vec<Instance>) -> Self {
        let raw_instances = instances.iter().map(InstanceRaw::from).collect::<Vec<_>>();

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("instance_buffer"),
            contents: bytemuck::cast_slice(&raw_instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self { instances, buffer }
    }
}
