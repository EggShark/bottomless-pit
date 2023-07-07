use crc32fast::Hasher;
use wgpu::{ShaderModule, RenderPipeline};
use crate::engine_handle::WgpuClump;
use crate::resource_cache::ResourceCache;
use crate::texture::Texture;
use crate::vertex::Vertex;
use crate::render::make_pipeline;

pub struct ShaderIndex {
    shader: ShaderModule,
    pub(crate) id: u32,
}

impl ShaderIndex {
    pub(crate) fn new(path: &str, wgpu: &WgpuClump) -> Result<Self, std::io::Error> {
        let file = std::fs::read(path)?;
        let mut hasher = Hasher::new();
        hasher.update(&file);
        let id = hasher.finalize();
        let shader_module = wgpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(String::from_utf8_lossy(&file)),
            });
        Ok(Self {
            shader: shader_module,
            id,
        })
    }
}

pub(crate) struct Shader {
    pipeline: RenderPipeline,
}

impl Shader {
    pub fn from_index(index: &ShaderIndex, wgpu_clump: &WgpuClump, camera_bind_group_layout: &wgpu::BindGroupLayout, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader_pipeline = make_pipeline(
            &wgpu_clump.device,
            wgpu::PrimitiveTopology::TriangleList,
            &[
                &Texture::make_bind_group_layout(&wgpu_clump.device),
                camera_bind_group_layout,
            ],
            &[Vertex::desc()],
            &index.shader,
            config.format,
            Some("shader_pipeline"),
        );

        Self {
            pipeline: shader_pipeline,
        }
    }
}

pub(crate) fn create_shader(
    shader_cache: &mut ResourceCache<Shader>,
    path: &str,
    wgpu: &WgpuClump,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    config: &wgpu::SurfaceConfiguration,
) -> Result<ShaderIndex, std::io::Error> {
    let shader_index = ShaderIndex::new(path, wgpu)?;

    let shader = Shader::from_index(&shader_index, wgpu, camera_bind_group_layout, config);
    
    shader_cache.add_item(shader, shader_index.id);
    Ok(shader_index)
}