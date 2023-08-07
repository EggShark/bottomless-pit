use std::num::NonZeroU64;

// potentially just store ids to a hashmap to 
// avoid reproductions
struct Material {
    pipeline_id: NonZeroU64,
    vertex_buffer: wgpu::Buffer,
    vertex_count: usize,
    index_buffer: wgpu::Buffer,
    index_count: usize,
    texture_id: NonZeroU64,
}