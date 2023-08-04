use crc32fast::Hasher;
use crevice::std140::{AsStd140, Std140};
use wgpu::util::DeviceExt;
use wgpu::{ShaderModule, RenderPipeline};
use crate::engine_handle::{WgpuClump, Engine};
use crate::{layouts, IDENTITY_MATRIX};
use crate::resource_cache::ResourceCache;
use crate::texture::Texture;
use crate::vertex::Vertex;
use crate::render::make_pipeline;
use std::slice::Iter;

pub struct ShaderIndex {
    shader: ShaderModule,
    layouts: Vec<wgpu::BindGroupLayout>,
    pub(crate) id: u32,
}

impl ShaderIndex {
    pub(crate) fn new(path: &str, wgpu: &WgpuClump, layouts: Vec<wgpu::BindGroupLayout>) -> Result<Self, std::io::Error> {
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
            layouts,
            id,
        })
    }

    pub(crate) fn from_module(moudle: ShaderModule, id: u32, layouts: Vec<wgpu::BindGroupLayout>) -> Self {
        Self {
            shader: moudle,
            layouts,
            id,
        }
    }
}

// how on earth am i gona do this 😔
// bindgroup is the onlything I need to give engine so like uh need to rebuild
// this can also act as the index? as its just a bind group and we have a bindgroup
// cahce not sure how I am gonna generate IDs? randomly????
pub struct ShaderOptions {
    buffer: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    pub(crate) id: u32,
}

impl ShaderOptions {
    pub fn new<Uniform: AsStd140>(uniform: &Uniform, engine_handle: &mut Engine) -> Self {
        // needs to add to resource cache
        let id = 12348901; // just for testing

        let wgpu = engine_handle.get_wgpu();

        let buffer = wgpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor{
                label: Some("User Uniform Buffer"),
                contents: bytemuck::cast_slice(&[IDENTITY_MATRIX]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            });

        let layout = layouts::create_uniform_layout(&wgpu.device);

        let bind_group = Self::make_bind_group(&layout, buffer.as_entire_binding(), wgpu);
        dbg!(&bind_group);

        engine_handle.add_to_bind_group_cache(bind_group, id);

        Self {
            buffer,
            layout,
            id,
        }
    }

    pub(crate) fn make_bind_group(layout: &wgpu::BindGroupLayout, resource: wgpu::BindingResource, wgpu: &WgpuClump) -> wgpu::BindGroup {
        wgpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource,
                }],
                label: Some("User uniform BindGroup"),
            })
    }

    pub(crate) fn rebuild_bindgroup(&self, wgpu: &WgpuClump) -> wgpu::BindGroup {
        Self::make_bind_group(&self.layout, self.buffer.as_entire_binding(), wgpu)
    }

    pub fn update_uniform<Uniform: AsStd140>(&self, uniform: &Uniform, engine_handle: &mut Engine) {
        engine_handle
            .get_wgpu()
            .queue
            .write_buffer(
                &self.buffer,
                0,
                bytemuck::cast_slice(&[IDENTITY_MATRIX]),
            );
    }
}

pub(crate) struct Shader {
    pipeline: RenderPipeline,
}

impl Shader {
    pub fn from_index(index: &ShaderIndex, wgpu_clump: &WgpuClump, config: &wgpu::SurfaceConfiguration, label: Option<&str>) -> Self {
        // double heap allocation IK but arrayvec didnt work
        let bg_layouts = index.layouts.iter().collect::<Vec<&wgpu::BindGroupLayout>>();

        let shader_pipeline = make_pipeline(
            &wgpu_clump.device,
            wgpu::PrimitiveTopology::TriangleList,
            &bg_layouts,
            &[Vertex::desc()],
            &index.shader,
            config.format,
            label,
        );

        Self {
            pipeline: shader_pipeline,
        }
    }

    pub fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

pub(crate) fn create_shader(
    shader_cache: &mut ResourceCache<Shader>,
    layouts: Vec<wgpu::BindGroupLayout>,
    path: &str,
    wgpu: &WgpuClump,
    config: &wgpu::SurfaceConfiguration,
    label: Option<&str>,
) -> Result<ShaderIndex, std::io::Error> {
    let shader_index = ShaderIndex::new(path, wgpu, layouts)?;

    let shader = Shader::from_index(&shader_index, wgpu, config, label);
    
    shader_cache.add_item(shader, shader_index.id);
    Ok(shader_index)
}