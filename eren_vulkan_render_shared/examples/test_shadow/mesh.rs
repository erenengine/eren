use std::mem::offset_of;
use std::sync::Arc;

use ash::vk;
use glam::Vec3;

use eren_vulkan_render_shared::{
    command::CommandPool,
    device::{
        BufferWithMemoryCreationError, CopyCommandBufferError, Device, MapMemoryError,
        MemoryUploadSlice,
    },
};
use thiserror::Error;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vec3, // layout(location = 0)
    pub normal: Vec3,   // layout(location = 1)
}

impl Vertex {
    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, position) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, normal) as u32,
            },
        ]
    }
}

pub struct MeshBuffer {
    device: Arc<Device>,

    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub vertex_offset: vk::DeviceSize,
    pub index_offset: vk::DeviceSize,
    pub index_count: u32,
}

#[derive(Debug, Error)]
pub enum MeshBufferInitializationError {
    #[error("Failed to create buffer with memory: {0}")]
    CreateBufferWithMemory(#[from] BufferWithMemoryCreationError),

    #[error("Failed to upload data to memory: {0}")]
    UploadDataToMemory(#[from] MapMemoryError),

    #[error("Failed to copy buffer: {0}")]
    CopyBuffer(#[from] CopyCommandBufferError),
}

impl MeshBuffer {
    pub fn new(
        device: Arc<Device>,
        command_pool: &CommandPool,
        vertices: &[Vertex],
        indices: &[u16],
    ) -> Result<MeshBuffer, MeshBufferInitializationError> {
        let vertex_size = (std::mem::size_of::<Vertex>() * vertices.len()) as vk::DeviceSize;
        let index_size = (std::mem::size_of::<u16>() * indices.len()) as vk::DeviceSize;

        let index_offset = (vertex_size + 3) & !3;
        let total_size = index_offset + index_size;

        let (staging_buffer, staging_memory) = device.create_buffer_with_memory(
            total_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let vertex_bytes = unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * std::mem::size_of::<Vertex>(),
            )
        };

        let index_bytes = unsafe {
            std::slice::from_raw_parts(
                indices.as_ptr() as *const u8,
                indices.len() * std::mem::size_of::<u16>(),
            )
        };

        let slices = [
            MemoryUploadSlice {
                src: vertex_bytes,
                dst_offset: 0,
            },
            MemoryUploadSlice {
                src: index_bytes,
                dst_offset: index_offset,
            },
        ];

        device.upload_slices_to_memory(staging_memory, total_size, &slices)?;

        let (buffer, memory) = device.create_buffer_with_memory(
            total_size,
            vk::BufferUsageFlags::TRANSFER_DST
                | vk::BufferUsageFlags::VERTEX_BUFFER
                | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        command_pool.copy_buffer(staging_buffer, buffer, total_size)?;
        device.destroy_buffer_with_memory(staging_buffer, staging_memory);

        Ok(MeshBuffer {
            device,
            buffer,
            memory,
            vertex_offset: 0,
            index_offset,
            index_count: indices.len() as u32,
        })
    }
}

impl Drop for MeshBuffer {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.device
            .destroy_buffer_with_memory(self.buffer, self.memory);
    }
}
