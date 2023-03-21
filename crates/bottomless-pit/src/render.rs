use crate::DrawQueues;
use crate::cache::TextureIndex;
use crate::{LineVertex, WHITE_PIXEL, Texture, Vertex};
use crate::engine_handle::WgpuClump;
use crate::vectors::Vec2;
use crate::colour::Colour; 
use crate::cache::TextureCache;
use crate::camera::Camera;
use crate::BindGroups;
use crate::matrix_math::*;
use crate::rect::Rectangle;

use image::GenericImageView;
use winit::dpi::PhysicalSize;

pub(crate) struct Renderer {
    //add stuff later
    white_pixel: wgpu::BindGroup,
    draw_queues: DrawQueues,
    glyph_brush: wgpu_glyph::GlyphBrush<(), wgpu_glyph::ab_glyph::FontArc>,
    pipelines: RenderPipelines,
    clear_colour: Colour,
    pub(crate) wgpu_clump: WgpuClump, // its very cringe storing this here and not in engine however texture chace requires it
    pub(crate) texture_cache: TextureCache,
}

impl Renderer {
    pub(crate) fn new(
        wgpu_clump: WgpuClump, 
        surface: &wgpu::Surface, 
        adapter: &wgpu::Adapter, 
        size: PhysicalSize<u32>, 
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        clear_colour: Colour,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let texture_cache = TextureCache::new();
        let draw_queues = DrawQueues::new();

        let minecraft_mono = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!("../assets/Minecraft-Mono.ttf")).unwrap();
        let glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_font(minecraft_mono)
            .build(&wgpu_clump.device, wgpu::TextureFormat::Bgra8UnormSrgb);
        
        let white_pixel_image = image::load_from_memory(WHITE_PIXEL).unwrap();
        let white_pixel_rgba = white_pixel_image.to_rgba8();
        let (width, height) = white_pixel_image.dimensions();
        let white_pixel_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let white_pixel_texture = wgpu_clump.device.create_texture(&wgpu::TextureDescriptor{
            size: white_pixel_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            view_formats: &[],
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("White_Pixel"),
        });

        wgpu_clump.queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &white_pixel_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &white_pixel_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(width * 4),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
            white_pixel_size,
        );

        let white_pixel_view = white_pixel_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let white_pixel_sampler = wgpu_clump.device.create_sampler(&wgpu::SamplerDescriptor{
            // what to do when given cordinates outside the textures height/width
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // what do when give less or more than 1 pixel to sample
            // linear interprelates between all of them nearest gives the closet colour
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = wgpu_clump.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float {filterable: true},
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ],
            label: Some("white_pixel_bind_group_layout"),
        });

        let white_pixel = wgpu_clump.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&white_pixel_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&white_pixel_sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

        let pipelines = RenderPipelines::new(&wgpu_clump, camera_bind_group_layout, texture_format);

        Self {
            white_pixel,
            draw_queues,
            glyph_brush,
            pipelines,
            clear_colour,
            wgpu_clump,
            texture_cache,
        }
    }

    pub fn draw_rectangle(&mut self, position: Vec2<f32>, width: f32, hieght: f32, colour: Colour) {
        let rectangle = Rectangle::new(position, [width, hieght], colour.to_raw());
        self.draw_queues.add_rectangle(&rectangle);
    }

    pub fn draw_textured_rectangle(&mut self, position: Vec2<f32>, width: f32, hieght: f32, texture: &TextureIndex) {
        let rectangle = Rectangle::new(position, [width, hieght], Colour::White.to_raw());
        self.draw_queues.add_textured_rectange(&mut self.texture_cache, &rectangle, texture, &self.wgpu_clump.device);
    }

    pub fn draw_line(&mut self, start_point: Vec2<f32>, end_point: Vec2<f32>, colour: Colour) {
        let start = LineVertex::new(start_point.to_raw(), colour.to_raw());
        let end = LineVertex::new(end_point.to_raw(), colour.to_raw());
        self.draw_queues.add_line(start, end) 
    } 

    pub(crate) fn render(&mut self, size: Vec2<u32>, camera: &wgpu::BindGroup, surface: &wgpu::Surface) -> Result<(), wgpu::SurfaceError>{
        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.wgpu_clump.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });
        
        let render_items = self.draw_queues.process_queued(&self.wgpu_clump.device);

        let text_sections = render_items.text
            .iter()
            .map(|text| wgpu_glyph::Section{
                screen_position: (text.position.x, text.position.y),
                bounds: (size.x as f32, size.y as f32),
                text: vec![wgpu_glyph::Text::new(&text.text).with_scale(text.scale).with_color(text.colour.to_raw())],
                ..Default::default()
            })
            .collect::<Vec<wgpu_glyph::Section>>();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment{
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_colour.into()),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        
        render_pass.set_pipeline(&self.pipelines.polygon_pipeline);
        render_pass.set_bind_group(1, camera, &[]);
        if render_items.number_of_rectangle_indicies != 0 {
            render_pass.set_vertex_buffer(0, render_items.rectangle_buffer.slice(..));
            render_pass.set_index_buffer(render_items.rectangle_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            let mut current_bind_group = &render_items.rectangle_bind_group_switches[0];
            for (idx, bind_group_switch_point) in render_items.rectangle_bind_group_switches.iter().enumerate() {
                if bind_group_switch_point.bind_group != current_bind_group.bind_group {
                    current_bind_group = &bind_group_switch_point;
                }
                let bind_group = match &current_bind_group.bind_group {
                    &BindGroups::WhitePixel => &self.white_pixel,
                    &BindGroups::Custom {bind_group} => &self.texture_cache[bind_group].bind_group,
                };
                render_pass.set_bind_group(0, bind_group, &[]);
                let draw_range = match render_items.rectangle_bind_group_switches.get(idx + 1) {
                    Some(switch_point) => current_bind_group.point as u32..switch_point.point as u32,
                    None => current_bind_group.point as u32..render_items.number_of_rectangle_indicies,
                };
                println!("{:?}", draw_range);
                render_pass.draw_indexed(draw_range, 0, 0..1);
            }
        }
        render_pass.set_pipeline(&self.pipelines.line_pipeline);
        render_pass.set_bind_group(0, camera, &[]);
        render_pass.set_vertex_buffer(0, render_items.line_buffer.slice(..));
        render_pass.draw(0..render_items.number_of_line_verticies, 0..1);
        drop(render_pass);

        let mut staging_belt = wgpu::util::StagingBelt::new(1024);
        //let text_transform = flatten_matrix(unflatten_matrix(orthographic_projection(self.size.width, self.size.height)) * get_text_rotation_matrix(&test_section, self.counter, &mut self.glyph_brush));

        render_items.transformed_text.iter()
            .map(|text| (wgpu_glyph::Section{
                screen_position: (text.position.x, text.position.y),
                bounds: (size.x as f32, size.y as f32),
                text: vec![wgpu_glyph::Text::new(&text.text).with_scale(text.scale).with_color(text.colour.to_raw())],
                ..Default::default()
                }, text.transformation))
            .for_each(|(section, transform)| {
                let text_transform = unflatten_matrix(transform);
                let ortho = unflatten_matrix(orthographic_projection(size.x, size.y));
                let transform = flatten_matrix(ortho * text_transform);
                self.glyph_brush.queue(section);
                self.glyph_brush.draw_queued_with_transform(
                    &self.wgpu_clump.device, &mut staging_belt, &mut encoder, &view, &camera, transform,
                ).unwrap();
            });

            text_sections.into_iter().for_each(|s| self.glyph_brush.queue(s));
            self.glyph_brush.draw_queued(&self.wgpu_clump.device, &mut staging_belt, &mut encoder, &view, camera, size.x, size.y).unwrap();

        staging_belt.finish();
        self.wgpu_clump.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub(crate) struct RenderPipelines {
    pub(crate) line_pipeline: wgpu::RenderPipeline,
    pub(crate) polygon_pipeline: wgpu::RenderPipeline,
}

impl RenderPipelines {
    pub fn new(wgpu_clump: &WgpuClump, camera_bind_group_layout: &wgpu::BindGroupLayout, texture_format: wgpu::TextureFormat) -> Self {
        let polygon_shader = wgpu_clump.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        let render_pipeline_layout = wgpu_clump.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &Texture::make_bind_group_layout(&wgpu_clump.device),
                camera_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let generic_shader = wgpu_clump.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        let line_shader = wgpu_clump.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: Some("Line_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/line_shader.wgsl").into()),
        });

        let polygon_pipeline = make_pipeline(&wgpu_clump.device,
            wgpu::PrimitiveTopology::TriangleList,
            &[&Texture::make_bind_group_layout(&wgpu_clump.device), camera_bind_group_layout], 
            &[Vertex::desc()],
            &generic_shader,
            texture_format,
            Some("Generic_pipeline")
        );

        let line_pipeline = make_pipeline(
            &wgpu_clump.device,
            wgpu::PrimitiveTopology::LineList,
            &[camera_bind_group_layout],
            &[LineVertex::desc()], 
            &line_shader, 
            texture_format, 
            Some("line_renderer")
        );

        Self {
            polygon_pipeline,
            line_pipeline,
        }
    }
}

fn make_pipeline(
    device: &wgpu::Device, 
    topology: wgpu::PrimitiveTopology, 
    bind_group_layouts: &[&wgpu::BindGroupLayout], 
    vertex_buffers: &[wgpu::VertexBufferLayout], 
    shader: &wgpu::ShaderModule, 
    texture_format: wgpu::TextureFormat,
    label: Option<&str>
) -> wgpu::RenderPipeline {
    let layout_label = match label {
        Some(label) => Some(format!("{}_layout", label)),
        None => None
    };

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: layout_label.as_deref(), // somehow converss Option<String> to Option<&str>
        bind_group_layouts,
        push_constant_ranges: &[],
    });
    
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(&layout),
        vertex: wgpu::VertexState{
            module: &shader,
            entry_point: "vs_main", //specify the entry point (can be whatever as long as it exists)
            buffers: vertex_buffers, // specfies what type of vertices we want to pass to the shader,
        },
        fragment: Some(wgpu::FragmentState{ // techically optional. Used to store colour data to the surface
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState{ // tells wgpu what colour outputs it should set up.
                format: texture_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING), // specifies that the blending should just replace old pixel data wiht new data,
                write_mask: wgpu::ColorWrites::ALL, // writes all colours
            })],
        }),
        primitive: wgpu::PrimitiveState{
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
        multisample: wgpu::MultisampleState{
            count: 1, // determines how many samples the pipeline will use
            mask: !0, // how many samples the pipeline will use (in this case all of them)
            alpha_to_coverage_enabled: false, // something to do with AA
        },
        multiview: None,
    })
}