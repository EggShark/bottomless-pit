//! Contains the Renderer struct which also contains all the
//! functions and logic to draw things to the screen

use crate::engine_handle::{Engine, WgpuCache, WgpuClump};
use crate::Game;
use crate::vectors::Vec2;

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
/// RenderInformation will live long enough, just properly annotate your functions.
pub struct RenderInformation<'pass, 'others> {
    pub(crate) size: Vec2<u32>,
    pub(crate) render_pass: wgpu::RenderPass<'pass>,
    pub(crate) bind_groups: &'others WgpuCache<wgpu::BindGroup>,
    pub(crate) pipelines: &'others WgpuCache<wgpu::RenderPipeline>,
    pub(crate) defualt_id: wgpu::Id<wgpu::RenderPipeline>,
    pub(crate) camera_bindgroup: &'others wgpu::BindGroup,
    pub(crate) texture_sampler: &'others wgpu::Sampler,
    pub(crate) wgpu: &'others WgpuClump,
}

pub(crate) fn render<T>(game: &mut T, engine: &mut Engine) -> Result<(), wgpu::SurfaceError> where T: Game, {
    let wgpu = engine.get_wgpu();
    let output = engine.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = wgpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
    
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(engine.wgpu_colour()),
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    });

    render_pass.set_bind_group(1, engine.camera_bindgroup(), &[]);
    
    let render_info = RenderInformation {
        size: engine.get_window_size(),
        render_pass,
        bind_groups: &engine.bindgroups,
        pipelines: &engine.pipelines,
        defualt_id: engine.defualt_pipe_id(),
        camera_bindgroup: engine.camera_bindgroup(),
        texture_sampler: engine.get_texture_sampler(),
        wgpu,
    };

    game.render(render_info);

    wgpu.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}