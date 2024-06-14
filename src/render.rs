//! Contains the Renderer struct which also contains all the
//! functions and logic to draw things to the screen
use crate::colour::Colour;
use crate::context::WgpuClump;
use crate::engine_handle::Engine;
use crate::resource::{ResourceId, ResourceManager};
use crate::shader::Shader;
use crate::texture::UniformTexture;
use crate::vectors::Vec2;
use crate::{Game, vec2};

pub(crate) fn make_pipeline(
    device: &wgpu::Device,
    topology: wgpu::PrimitiveTopology,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    vertex_buffers: &[wgpu::VertexBufferLayout],
    shader: &wgpu::ShaderModule,
    texture_format: wgpu::TextureFormat,
    label: Option<&str>,
) -> wgpu::RenderPipeline {
    let layout_label = label.map(|label| format!("{}_layout", label));

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: layout_label.as_deref(), // somehow converss Option<String> to Option<&str>
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main", //specify the entry point (can be whatever as long as it exists)
            buffers: vertex_buffers, // specfies what type of vertices we want to pass to the shader,
            compilation_options: wgpu::PipelineCompilationOptions::default()
        },
        fragment: Some(wgpu::FragmentState {
            // techically optional. Used to store colour data to the surface
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                // tells wgpu what colour outputs it should set up.
                format: texture_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING), // specifies that the blending should just replace old pixel data wiht new data,
                write_mask: wgpu::ColorWrites::ALL,            // writes all colours
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw, // triagnle must be counter-clock wise to be considered facing forawrd
            cull_mode: Some(wgpu::Face::Back), // all triagnles not front facing are culled
            // setting this to anything other than fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // requires Features::DEPTH_CLIP_CONTROLL,
            unclipped_depth: false,
            // requires Features::CONSERVATIVE_RASTERIZATION,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,                         // determines how many samples the pipeline will use
            mask: !0, // how many samples the pipeline will use (in this case all of them)
            alpha_to_coverage_enabled: false, // something to do with AA
        },
        multiview: None,
    })
}

/// RenderInformation contains all the information needed to render objects to the sreen.
/// You should never have to worry about lifetimes as the way the event loop is strutured
/// RenderInformation will live long enough, just properly annotate your functions like so:
/// ```rust
/// fn render<'a, 'b>(&'b mut self, mut render_handle: RenderInformation<'a, 'b>) where 'b: 'a {
///     // do something here
/// }
/// ```
pub(crate) fn render<T>(game: &mut T, engine: &mut Engine) -> Result<(), wgpu::SurfaceError>
where
    T: Game,
{
    // there is a chance an .unwrap() would panic bc of an unloaded resource
    if engine.is_loading() {
        return Ok(());
    }

    let render_handle = RenderHandle::from(engine);

    game.render(render_handle);

    Ok(())
}

pub struct RenderHandle<'a> {
    // ME WHEN BC HACKS :3
    encoder: Option<wgpu::CommandEncoder>,
    surface: Option<wgpu::SurfaceTexture>,
    resources: &'a ResourceManager,
    defualt_id: ResourceId<Shader>,
    defualt_view: wgpu::TextureView,
    defualt_view_size: Vec2<u32>,
    camera_bindgroup: &'a wgpu::BindGroup,
    wgpu: &'a WgpuClump,
}

impl<'a> RenderHandle<'a> {
    pub fn begin_pass<'o, 'p>(&'o mut self, clear_colour: Colour) -> Renderer<'o, 'p> {

        let mut pass = match &mut self.encoder {
            Some(encoder) => {
                Self::create_pass(encoder, &self.defualt_view, clear_colour.into())
            }
            None => unreachable!(),
        };

        let pipeline = &self
            .resources
            .get_pipeline(&self.defualt_id)
            .unwrap()
            .pipeline;

        pass.set_pipeline(pipeline);
        pass.set_bind_group(1, self.camera_bindgroup, &[]);

        Renderer {
            pass,
            size: self.defualt_view_size,
            defualt_id: self.defualt_id,
            resources: &self.resources,
            camera_bindgroup: &self.camera_bindgroup,
            wgpu: &self.wgpu
        }
    }

    pub fn begin_texture_pass<'o, 'p>(&'o mut self, texture: &'o mut UniformTexture, clear_colour: Colour) -> Renderer<'o, 'p> {
        let size = texture.get_size();
        
        let mut pass = match &mut self.encoder {
            Some(encoder) => {
                Self::create_pass(encoder, texture.make_render_view(), clear_colour.into())
            }
            None => unreachable!(),
        };


        let pipeline = &self
        .resources
        .get_pipeline(&self.defualt_id)
        .unwrap()
        .pipeline;

        pass.set_pipeline(pipeline);
        pass.set_bind_group(1, self.camera_bindgroup, &[]);

        Renderer {
            pass,
            size,
            defualt_id: self.defualt_id,
            resources: &self.resources,
            camera_bindgroup: &self.camera_bindgroup,
            wgpu: &self.wgpu
        }
    }

    fn create_pass<'p>(encoder: &'p mut wgpu::CommandEncoder, view: &'p wgpu::TextureView, clear_colour: wgpu::Color) -> wgpu::RenderPass<'p> {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_colour),
                    store: wgpu::StoreOp::Store,
                },
            })],
            timestamp_writes: None,
            occlusion_query_set: None,
            depth_stencil_attachment: None,
        });

        render_pass
    }

}

impl<'a> From<&'a mut Engine> for RenderHandle<'a> {
    fn from(value: &'a mut Engine) -> Self {
        let encoder = value.get_wgpu().device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render encoder")
        });

        let texture = value
            .get_current_texture()
            .unwrap();

        let defualt_view_size = texture.texture.size();
        let defualt_view_size = vec2!(defualt_view_size.width, defualt_view_size.height);
        let defualt_view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            encoder: Some(encoder),
            surface: Some(texture),
            resources: value.get_resources(),
            defualt_id: value.defualt_pipe_id(),
            defualt_view,
            defualt_view_size,
            camera_bindgroup: value.camera_bindgroup(),
            wgpu: value.get_wgpu(),
        }
    }
}

impl Drop for RenderHandle<'_> {
    fn drop(&mut self) {
        self.wgpu.queue.submit(std::iter::once(self.encoder.take().unwrap().finish()));

        self.surface.take().unwrap().present();
    }
}

pub struct Renderer<'o, 'p> where 'o: 'p {
    pub(crate) pass: wgpu::RenderPass<'p>,
    pub(crate) size: Vec2<u32>,
    pub(crate) resources: &'o ResourceManager,
    pub(crate) defualt_id: ResourceId<Shader>,
    pub(crate) camera_bindgroup: &'o wgpu::BindGroup,
    pub(crate) wgpu: &'o WgpuClump,
}

impl<'p, 'o> Renderer<'p, 'o> {
    pub fn reset_camera(&mut self) {
        self.pass.set_bind_group(1, self.camera_bindgroup, &[]);
    }

    pub fn get_size(&self) -> Vec2<u32> {
        self.size
    }
}
