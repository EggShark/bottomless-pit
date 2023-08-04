//! Contains the Renderer struct which also contains all the
//! functions and logic to draw things to the screen

use crate::colour::Colour;
use crate::draw_queue::{BindGroups, DrawQueues, SwitchPoint};
use crate::engine_handle::WgpuClump;
use crate::{matrix_math::*, IDENTITY_MATRIX, layouts};
use crate::rect::Rectangle;
use crate::resource_cache::ResourceCache;
use crate::shader::{Shader, ShaderIndex, ShaderOptions};
use crate::text::{Text, TransformedText};
use crate::texture::TextureIndex;
use crate::vectors::Vec2;
use crate::vertex::{LineVertex, Vertex};
use crate::wgpu_glyph;
use crate::wgpu_glyph::{orthographic_projection, Layout};
use crate::WHITE_PIXEL;

use image::GenericImageView;
use winit::dpi::PhysicalSize;

/// The handle used for rendering all objects
pub struct Renderer {
    white_pixel: wgpu::BindGroup,
    draw_queues: DrawQueues,
    line_pipeline: wgpu::RenderPipeline,
    defualt_shader: ShaderIndex,
    clear_colour: Colour,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) camera_matrix: [f32; 16],
    pub(crate) camera_bind_group: wgpu::BindGroup,
    pub(crate) camera_bind_group_layout: wgpu::BindGroupLayout, // used for making shaders
    pub(crate) camera_buffer: wgpu::Buffer,
    pub(crate) glyph_brush: wgpu_glyph::GlyphBrush<(), wgpu_glyph::ab_glyph::FontArc>,
    pub(crate) wgpu_clump: WgpuClump, // its very cringe storing this here and not in engine however texture chace requires it
    pub(crate) size: Vec2<u32>,       // goes here bc normilzing stuff
    pub(crate) bind_group_cache: ResourceCache<wgpu::BindGroup>,
    pub(crate) shader_cache: ResourceCache<Shader>

}

impl Renderer {
    pub(crate) fn new(
        wgpu_clump: WgpuClump,
        size: PhysicalSize<u32>,
        camera_bind_group: wgpu::BindGroup,
        camera_bind_group_layout: wgpu::BindGroupLayout,
        camera_buffer: wgpu::Buffer,
        clear_colour: Colour,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        let texture_format = config.format;

        let bind_group_cache = ResourceCache::new();
        let draw_queues = DrawQueues::new();

        let minecraft_mono =
            wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!("../Monocraft.ttf"))
                .unwrap();
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

        let white_pixel_texture = wgpu_clump.device.create_texture(&wgpu::TextureDescriptor {
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
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            white_pixel_size,
        );

        let white_pixel_view =
            white_pixel_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let white_pixel_sampler = wgpu_clump.device.create_sampler(&wgpu::SamplerDescriptor {
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

        let bind_group_layout =
            wgpu_clump
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("white_pixel_bind_group_layout"),
                });

        let white_pixel = wgpu_clump
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&white_pixel_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&white_pixel_sampler),
                    },
                ],
                label: Some("texture_bind_group"),
            });

        let generic_shader = wgpu_clump
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
            });

        let line_shader = wgpu_clump
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Line_Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/line_shader.wgsl").into()),
            });

        let line_pipeline = make_pipeline(
            &wgpu_clump.device,
            wgpu::PrimitiveTopology::LineList,
            &[&camera_bind_group_layout],
            &[LineVertex::desc()],
            &line_shader,
            texture_format,
            Some("line_renderer"),
        );

        let defualt_index = ShaderIndex::from_module(
            generic_shader,
            0,
            vec![
                layouts::create_texture_layout(&wgpu_clump.device),
                layouts::create_camera_layout(&wgpu_clump.device),
            ],
        );
        let defualt_shader = Shader::from_index(
            &defualt_index,
            &wgpu_clump,
            &config,
            Some("defualt pipleline"),
        );

        let mut shader_cache = ResourceCache::new();
        shader_cache.add_item(defualt_shader, 0);

        Self {
            white_pixel,
            draw_queues,
            line_pipeline,
            defualt_shader: defualt_index,
            glyph_brush,
            clear_colour,
            camera_matrix: IDENTITY_MATRIX,
            camera_bind_group,
            camera_bind_group_layout,
            camera_buffer,
            wgpu_clump,
            size: size.into(),
            bind_group_cache,
            shader_cache,
            config,
        }
    }

    /// draws a textureless rectangle with a specificed colour
    pub fn draw_rectangle(&mut self, position: Vec2<f32>, width: f32, hieght: f32, colour: Colour) {
        let rectangle =
            Rectangle::from_pixels(position, [width, hieght], colour.to_raw(), self.size);
        self.draw_queues.add_rectangle(&rectangle);
    }

    /// Draws a rectangle in WGSL screenspace. point(-1.0, -1.0) is the bottom left corner and (0.0,0.0) is the center
    /// of the screen.
    pub fn draw_screenspace_rectangle(
        &mut self,
        position: Vec2<f32>,
        width: f32,
        hieght: f32,
        colour: Colour,
    ) {
        let rectangle = Rectangle::new(position, [width, hieght], colour.to_raw());
        self.draw_queues.add_rectangle(&rectangle);
    }

    /// draws a textured rectangle, however it will draw the entire texture
    pub fn draw_textured_rectangle(
        &mut self,
        position: Vec2<f32>,
        width: f32,
        hieght: f32,
        texture: &TextureIndex,
    ) {
        let rectangle =
            Rectangle::from_pixels(position, [width, hieght], Colour::White.to_raw(), self.size);
        self.draw_queues.add_textured_rectange(
            &mut self.bind_group_cache,
            &rectangle,
            texture,
            &self.wgpu_clump.device,
        );
    }

    /// Draws a textured rectangle in WGSL screenspace. point(-1.0, -1.0) is the bottom left corner and (0.0,0.0) is the
    /// center of the screen.
    pub fn draw_textured_screenspace_rectangle(
        &mut self,
        position: Vec2<f32>,
        width: f32,
        hieght: f32,
        texture: &TextureIndex,
    ) {
        let rectangle = Rectangle::new(position, [width, hieght], Colour::White.to_raw());
        self.draw_queues.add_textured_rectange(
            &mut self.bind_group_cache,
            &rectangle,
            texture,
            &self.wgpu_clump.device,
        );
    }

    /// draws a textured rectangle with the specifed UV coords.
    /// The image coords are not relative terms but the pixels of the image.
    /// uv_position is the top left corner for the uv rectangle to start, then the width and the height
    /// are just the width and the height of the uv rectangle.
    /// ```rust
    /// renderer.draw_textured_rectangle_with_uv(position, 100.0, 100.0, texture, Vec2{x: 0.0, y: 0.0}, Vec2{x: 100.0, y: 100.0})
    /// ```
    pub fn draw_textured_rectangle_with_uv(
        &mut self,
        position: Vec2<f32>,
        width: f32,
        hieght: f32,
        texture: &TextureIndex,
        uv_position: Vec2<f32>,
        uv_size: Vec2<f32>,
    ) {
        let uv_position = normalize_points(uv_position, texture.size.x, texture.size.y);
        let uv_width = uv_size.x / texture.size.x;
        let uv_height = uv_size.y / texture.size.y;

        let rectangle = Rectangle::from_pixels_with_uv(
            position,
            [width, hieght],
            Colour::White.to_raw(),
            self.size,
            uv_position,
            Vec2 {
                x: uv_width,
                y: uv_height,
            },
        );
        self.draw_queues.add_textured_rectange(
            &mut self.bind_group_cache,
            &rectangle,
            texture,
            &self.wgpu_clump.device,
        );
    }

    /// Draws a textured rectangle that rotates around its center point
    pub fn draw_textured_rect_with_rotation(
        &mut self,
        position: Vec2<f32>,
        width: f32,
        hieght: f32,
        texture: &TextureIndex,
        deg: f32,
    ) {
        let rectangle = Rectangle::from_pixels_with_rotation(
            position,
            [width, hieght],
            Colour::White.to_raw(),
            self.size,
            deg
        );

        self.draw_queues.add_textured_rectange(
            &mut self.bind_group_cache,
            &rectangle,
            texture,
            &self.wgpu_clump.device,
        );
    }

    /// Draws a textured rectangle while allowing you to sepcifiy, rotaion, and UV coridnates
    pub fn draw_textured_rectangle_ex(
        &mut self,
        position: Vec2<f32>,
        width: f32,
        hieght: f32,
        texture: &TextureIndex,
        deg: f32,
        uv_position: Vec2<f32>,
        uv_size: Vec2<f32>,
    ) {
        let uv_position = normalize_points(uv_position, texture.size.x, texture.size.y);
        let uv_size = Vec2{x: uv_size.x / texture.size.x, y: uv_size.y / texture.size.y};

        let rectangle = Rectangle::from_pixels_ex(
            position,
            [width, hieght],
            Colour::White.to_raw(),
            self.size,
            deg,
            uv_position,
            uv_size,
        );

        self.draw_queues.add_textured_rectange(
            &mut self.bind_group_cache,
            &rectangle,
            texture,
            &self.wgpu_clump.device,
        );
    }

    /// draws a triangle at the specificed coordniates with the specified colour
    /// verticies MUST be in CLOCKWISE rotation ex:
    /// ```rust,no_run
    /// render_handle.draw_triangle(Vec2{x: 300.0, y: 0.0}, Vec2{x: 350.0, y: 100.0}, Vec2{x: 250.0, y: 100.0}, Colour::White);
    /// ```
    pub fn draw_triangle(&mut self, p1: Vec2<f32>, p2: Vec2<f32>, p3: Vec2<f32>, colour: Colour) {
        let tex_coords = [0.0, 0.0];
        let colour = colour.to_raw();
        let points = [
            Vertex::from_2d([p1.x, p1.y], tex_coords, colour)
                .pixels_to_screenspace(self.size),
            Vertex::from_2d([p2.x, p2.y], tex_coords, colour)
                .pixels_to_screenspace(self.size),
            Vertex::from_2d([p3.x, p3.y], tex_coords, colour)
                .pixels_to_screenspace(self.size),
        ];
        self.draw_queues.add_triangle(points);
    }

    /// Does the same as draw_triangle but lets you speicify a colour per vertex again
    /// verticies MUST be in CLOCKWISE rotation
    pub fn draw_triangle_with_coloured_verticies(
        &mut self,
        p1: Vec2<f32>,
        p2: Vec2<f32>,
        p3: Vec2<f32>,
        c1: Colour,
        c2: Colour,
        c3: Colour,
    ) {
        let tex_coords = [0.0, 0.0];
        let points = [
            Vertex::from_2d([p1.x, p1.y], tex_coords, c1.to_raw())
                .pixels_to_screenspace(self.size),
            Vertex::from_2d([p2.x, p2.y], tex_coords, c2.to_raw())
                .pixels_to_screenspace(self.size),
            Vertex::from_2d([p3.x, p3.y], tex_coords, c3.to_raw())
                .pixels_to_screenspace(self.size)
        ];
        self.draw_queues.add_triangle(points);
    }

    /// draws a regular polygon of any number of sides
    pub fn draw_regular_n_gon(
        &mut self,
        number_of_sides: u16,
        radius: f32,
        center: Vec2<f32>,
        colour: Colour,
    ) {
        self.draw_queues
            .add_regular_n_gon(number_of_sides, radius, center.to_raw(), colour);
    }

    /// draws a line, WILL DRAW ONTOP OF EVERTHING ELSE DUE TO BEING ITS OWN PIPELINE
    pub fn draw_line(&mut self, start_point: Vec2<f32>, end_point: Vec2<f32>, colour: Colour) {
        let start = LineVertex::new(start_point.to_raw(), colour.to_raw())
            .pixels_to_screenspace(self.size);
        let end = LineVertex::new(end_point.to_raw(), colour.to_raw())
            .pixels_to_screenspace(self.size);

        self.draw_queues.add_line(start, end)
    }

    /// Draws some text that will appear on top of all other elements due to seperate pipelines
    pub fn draw_text(&mut self, text: &str, position: Vec2<f32>, scale: f32, colour: Colour) {
        let text = Text {
            text: text.into(),
            position,
            scale,
            colour,
        };
        self.draw_queues.add_text(text)
    }

    /// Draws text with custom transform matrix
    pub fn draw_text_with_transform(
        &mut self,
        text: &str,
        position: Vec2<f32>,
        scale: f32,
        colour: Colour,
        transform: [f32; 16],
    ) {
        let text = TransformedText {
            text: text.into(),
            position,
            scale,
            colour,
            transformation: transform,
            bounds: (self.size.x as f32, self.size.y as f32),
        };
        self.draw_queues.add_transfromed_text(text)
    }

    /// Sets the current shader to the shader supplied. Everything will be drawn from this shader untill changed or at the next loop
    pub fn set_shader(&mut self, shader: &ShaderIndex) {
        self.draw_queues.add_shader_point(
            &mut self.shader_cache,
            shader,
            &self.wgpu_clump,
            &self.config
        );
    }

    pub fn set_shader_options(&mut self, uniform: &ShaderOptions) {
        self.draw_queues.add_shader_option_point(
            &mut self.bind_group_cache,
            uniform,
            &self.wgpu_clump,
        );
    }

    /// Resets the active shader back to the engines defualt. This is also done automatically at the start of 
    /// each draw call
    pub fn set_to_defualt_shader(&mut self) {
        println!("to defualt !");
        self.draw_queues.add_shader_point(
            &mut self.shader_cache,
            &self.defualt_shader,
            &self.wgpu_clump,
            &self.config
        )
    }

    pub(crate) fn render(
        &mut self,
        size: Vec2<u32>,
        surface: &wgpu::Surface,
    ) -> Result<(), wgpu::SurfaceError> {
        println!("RENDER !");

        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.wgpu_clump
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        let render_items = self.draw_queues.process_queued(&self.wgpu_clump.device);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_colour.into()),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        match self.shader_cache.get_mut(0) {
            Some(shader_thing) => shader_thing.time_since_used = 0,
            None => unreachable!(),
        }

        render_pass.set_pipeline(self.shader_cache[0].resource.get_pipeline());
        render_pass.set_bind_group(0, &self.white_pixel, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

        if render_items.number_of_rectangle_indicies != 0 {
            render_pass.set_vertex_buffer(0, render_items.rectangle_buffer.slice(..));
            render_pass.set_index_buffer(
                render_items.rectangle_index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            let mut current_texture = BindGroups::WhitePixel;

            for (idx, switch_point) in render_items
                .general_switches
                .iter()
                .enumerate()
            {
                println!("switch point {:?}", switch_point);
                match switch_point {
                    SwitchPoint::Shader { id, point} => {
                        render_pass.set_pipeline(self.shader_cache[*id].resource.get_pipeline());
                        println!("shader switched id: {}", id);
                        let draw_range = match render_items.general_switches.get(idx + 1) {
                            Some(switch_point) => {
                                *point as u32..switch_point.get_point() as u32
                            }
                            None => {
                                *point as u32..render_items.number_of_rectangle_indicies
                            }
                        };
                        println!("draw_rangeSS: {:?}", draw_range);
                        render_pass.draw_indexed(draw_range, 0, 0..1);
                    },
                    SwitchPoint::BindGroup { bind_group, point } => {
                        if *bind_group != current_texture {
                            current_texture= *bind_group;
                        }
                
                        let (bind_group, index) = match current_texture {
                            BindGroups::WhitePixel => (&self.white_pixel, 0),
                            BindGroups::Texture { bind_group } => (&self.bind_group_cache[bind_group].resource, 0),
                            BindGroups::ShaderOptions{ bind_group, group_num } => {
                                dbg!(&self.bind_group_cache[bind_group].resource);
                                (&self.bind_group_cache[bind_group].resource, 2)
                            },
                        };

                        println!("index: {}, has been set", index);
                
                        render_pass.set_bind_group(index, bind_group, &[]);
                    
                        let draw_range = match render_items.general_switches.get(idx + 1) {
                            Some(switch_point) => {
                                *point as u32..switch_point.get_point() as u32
                            }
                            None => {
                                *point as u32..render_items.number_of_rectangle_indicies
                            }
                        };
                        println!("draw_rage: {:?}", draw_range);

                        render_pass.draw_indexed(draw_range, 0, 0..1);
                    }
                }
            }
        }

        // line rendering uses diffrent shader have to shift over
        render_pass.set_pipeline(&self.line_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, render_items.line_buffer.slice(..));
        render_pass.draw(0..render_items.number_of_line_verticies, 0..1);
        drop(render_pass);

        let mut staging_belt = wgpu::util::StagingBelt::new(1024);

        render_items
            .transformed_text
            .iter()
            .map(|text| {
                (
                    wgpu_glyph::Section {
                        screen_position: (text.position.x, text.position.y),
                        bounds: text.bounds,
                        text: vec![wgpu_glyph::Text::new(&text.text)
                            .with_scale(text.scale)
                            .with_color(text.colour.to_raw())],
                        layout: Layout::default_single_line()
                            .line_breaker(wgpu_glyph::BuiltInLineBreaker::AnyCharLineBreaker),
                    },
                    text.transformation,
                )
            })
            .for_each(|(section, transform)| {
                let text_transform = unflatten_matrix(transform);
                let ortho = unflatten_matrix(orthographic_projection(size.x, size.y));
                let transform = flatten_matrix(ortho * text_transform);
                self.glyph_brush.queue(section);
                self.glyph_brush
                    .draw_queued_with_transform(
                        &self.wgpu_clump.device,
                        &mut staging_belt,
                        &mut encoder,
                        &view,
                        &self.camera_bind_group,
                        transform,
                    )
                    .unwrap();
            });

        render_items
            .text
            .iter()
            .map(|text| wgpu_glyph::Section {
                screen_position: (text.position.x, text.position.y),
                bounds: (size.x as f32, size.y as f32),
                text: vec![wgpu_glyph::Text::new(&text.text)
                    .with_scale(text.scale)
                    .with_color(text.colour.to_raw())],
                layout: Layout::default_single_line()
                    .line_breaker(wgpu_glyph::BuiltInLineBreaker::AnyCharLineBreaker),
            })
            .for_each(|s| self.glyph_brush.queue(s));

        self.glyph_brush
            .draw_queued(
                &self.wgpu_clump.device,
                &mut staging_belt,
                &mut encoder,
                &view,
                &self.camera_bind_group,
                size.x,
                size.y,
            )
            .unwrap();

        staging_belt.finish();
        self.wgpu_clump
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

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
