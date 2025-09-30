//! Bottomles Pit supports multipass rendering. This means you can go through
//! multiple render passes within the [Game::render] function. This can be used
//! to render to a texture then render to the screen. The [RenderHandle] is used
//! to create render passes and then within the passes the [Renderer] is used to
//! actually interface with materials and other objects that render.
//! ```rust
//! impl Game for MyCoolGame {
//!     fn render<'o>(&'o mut self, mut render_handle: RenderHandle<'o>) {
//!         {
//!             let mut p1 = render_handle.begin_texture_pass(&mut self.uniform_texture, Colour::WHITE);
//!             // render what ever you want to the texture
//!         }
//!         let mut p2 = render_handle.begin_pass(Colour::Black);
//!         // render whatever you want to the screen
//!     }
//! }
//! ```

use crate::colour::Colour;
use crate::context::WgpuClump;
use crate::engine_handle::Engine;
use crate::resource::{ResourceId, ResourceManager};
use crate::shader::Shader;
use crate::texture::UniformTexture;
use crate::vectors::Vec2;
use crate::{vec2, Game};

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
            entry_point: Some("vs_main"), //specify the entry point (can be whatever as long as it exists)
            buffers: vertex_buffers, // specfies what type of vertices we want to pass to the shader,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            // techically optional. Used to store colour data to the surface
            module: shader,
            entry_point: Some("fs_main"),
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
        cache: None,
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

/// The renderer is used to create render passes or [Renderer]s
pub struct RenderHandle<'a> {
    // ME WHEN BC HACKS :3
    encoder: Option<wgpu::CommandEncoder>,
    surface: Option<wgpu::SurfaceTexture>,
    pub(crate) resources: &'a ResourceManager,
    default_id: ResourceId<Shader>,
    default_view: wgpu::TextureView,
    default_view_size: Vec2<u32>,
    camera_bindgroup: &'a wgpu::BindGroup,
    pub(crate) wgpu: &'a WgpuClump,
    format: wgpu::TextureFormat,
}

impl<'a> RenderHandle<'a> {
    /// Creates a render pass that will render onto the windows surface.
    pub fn begin_pass<'p>(&mut self, clear_colour: Colour) -> Renderer<'_, 'p> {
        let mut pass = match &mut self.encoder {
            Some(encoder) => Self::create_pass(encoder, &self.default_view, clear_colour.into()),
            None => unreachable!(),
        };

        let pipeline = &self
            .resources
            .get_pipeline(&self.default_id)
            .unwrap()
            .pipeline;

        pass.set_pipeline(pipeline);
        pass.set_bind_group(1, self.camera_bindgroup, &[]);

        Renderer {
            pass,
            size: self.default_view_size,
            default_id: self.default_id,
            resources: self.resources,
            camera_bindgroup: self.camera_bindgroup,
            wgpu: self.wgpu,
        }
    }

    /// Creates a render pass that can be used to render to a texture. This is usefull for
    /// lightmaps, or generating heightmaps. Any advanced graphics techique that requires
    /// uniform textures.
    pub fn begin_texture_pass<'o, 'p>(
        &'o mut self,
        texture: &'o mut UniformTexture,
        clear_colour: Colour,
    ) -> Renderer<'o, 'p> {
        let size = texture.get_size();

        let mut pass = match &mut self.encoder {
            Some(encoder) => Self::create_pass(
                encoder,
                texture.make_render_view(self.wgpu, self.format),
                clear_colour.into(),
            ),
            None => unreachable!(),
        };

        let pipeline = &self
            .resources
            .get_pipeline(&self.default_id)
            .unwrap()
            .pipeline;

        pass.set_pipeline(pipeline);
        pass.set_bind_group(1, self.camera_bindgroup, &[]);

        Renderer {
            pass,
            size,
            default_id: self.default_id,
            resources: self.resources,
            camera_bindgroup: self.camera_bindgroup,
            wgpu: self.wgpu,
        }
    }

    fn create_pass<'p>(
        encoder: &'p mut wgpu::CommandEncoder,
        view: &'p wgpu::TextureView,
        clear_colour: wgpu::Color,
    ) -> wgpu::RenderPass<'p> {
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
        let context = value.context.as_ref().unwrap();

        let encoder = context
            .wgpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        let texture = context.get_surface_texture().unwrap();

        let default_view_size = texture.texture.size();
        let default_view_size = vec2!(default_view_size.width, default_view_size.height);
        let default_view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            encoder: Some(encoder),
            surface: Some(texture),
            resources: value.get_resources(),
            default_id: value.default_pipe_id(),
            default_view,
            default_view_size,
            camera_bindgroup: &context.camera_bind_group,
            wgpu: &context.wgpu,
            format: context.get_texture_format(),
        }
    }
}

impl Drop for RenderHandle<'_> {
    fn drop(&mut self) {
        self.wgpu
            .queue
            .submit(std::iter::once(self.encoder.take().unwrap().finish()));

        self.surface.take().unwrap().present();
    }
}

/// Use the Renderer with [Materials](crate::material::Material) to draw things
/// to the current render surface
pub struct Renderer<'o, 'p>
where
    'o: 'p,
{
    pub(crate) pass: wgpu::RenderPass<'p>,
    pub(crate) size: Vec2<u32>,
    pub(crate) resources: &'o ResourceManager,
    pub(crate) default_id: ResourceId<Shader>,
    pub(crate) camera_bindgroup: &'o wgpu::BindGroup,
    pub(crate) wgpu: &'o WgpuClump,
}

impl<'p, 'o> Renderer<'p, 'o> {
    /// Resets the camera to the default camera.
    pub fn reset_camera(&mut self) {
        self.pass.set_bind_group(1, self.camera_bindgroup, &[]);
    }

    /// Gives the size of the current render surface
    pub fn get_size(&self) -> Vec2<u32> {
        self.size
    }
}
