use wgpu::util::DeviceExt;

use crate::colour::Colour;
use crate::engine_handle::WgpuClump;
use crate::layouts;
use crate::rect::Rectangle;
use crate::resource_cache::ResourceCache;
use crate::shader::{ShaderIndex, ShaderOptions, Shader};
use crate::text::{Text, TransformedText};
use crate::texture::TextureIndex;
use crate::vertex::{LineVertex, Vertex};
use std::f32::consts::PI;

#[derive(Debug)]
pub(crate) struct DrawQueues {
    general_vertices: Vec<Vertex>,
    general_indicies: Vec<u16>,
    general_switches: Vec<SwitchPoint>,
    text: Vec<Text>,
    transformed_text: Vec<TransformedText>,
    line_vertices: Vec<LineVertex>,
}

impl DrawQueues {
    pub(crate) fn new() -> Self {
        Self {
            general_vertices: Vec::new(),
            general_indicies: Vec::new(),
            general_switches: Vec::new(),
            text: Vec::new(),
            transformed_text: Vec::new(),
            line_vertices: Vec::new(),
        }
    }

    #[rustfmt::skip]
    // once again need fmt skip for indicies
    pub(crate) fn add_rectangle(&mut self, rectangle: &Rectangle) {
        let vertices = rectangle.get_vertices();
        let number_of_verticies = self.general_vertices.len() as u16;
        let number_of_inidices = self.general_indicies.len();
        // do index math
        let indicies = [
            number_of_verticies, 1 + number_of_verticies, 2 + number_of_verticies,
            3 + number_of_verticies, number_of_verticies, 2 + number_of_verticies,
        ];

        match self.general_switches.iter().filter(|i| i.is_texture()).last() {
            Some(point) => {
                if !point.is_white_pixel() {
                    self.general_switches
                        .push(SwitchPoint::BindGroup {
                            bind_group: BindGroups::WhitePixel,
                            point: number_of_inidices,
                        });
                }
            }
            None => {
                self.general_switches
                    .push(SwitchPoint::BindGroup {
                        bind_group: BindGroups::WhitePixel,
                        point: number_of_inidices,
                    });
            }
        };

        self.general_vertices.extend_from_slice(&vertices);
        self.general_indicies.extend_from_slice(&indicies);
    }

    #[rustfmt::skip]
    // the indicies math I want formated becuase 2 triagnles same for above
    pub(crate) fn add_textured_rectange(
        &mut self,
        cache: &mut ResourceCache<wgpu::BindGroup>,
        rectangle: &Rectangle,
        texture: &TextureIndex,
        device: &wgpu::Device,
    ) {
        let vertices = rectangle.get_vertices();
        let texture_bind_group = texture.id;
        let number_of_verticies = self.general_vertices.len() as u16;
        let number_of_inidices = self.general_indicies.len();
        // do index math
        let indicies = [
            number_of_verticies, 1 + number_of_verticies, 2 + number_of_verticies,
            3 + number_of_verticies, number_of_verticies, 2 + number_of_verticies,
        ];

        match cache.get_mut(texture.id) {
            Some(item) => item.time_since_used = 0,
            None => {
                let bind_group_layout = layouts::create_texture_layout(device);
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(texture.get_view()),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(texture.get_sampler()),
                        },
                    ],
                    label: Some("diffuse_bind_group"),
                });

                cache.add_item(bind_group, texture.id);
            },
        }

        match self.general_switches.iter().filter(|i| i.is_texture()).last() {
            Some(point) => {
                match point {
                    SwitchPoint::BindGroup {
                        bind_group: BindGroups::Texture { bind_group: bind_group_id },
                        ..
                    } => {
                        if *bind_group_id != texture_bind_group {
                            self.general_switches
                                .push(SwitchPoint::BindGroup {
                                    bind_group: BindGroups::Texture {
                                        bind_group: texture_bind_group,
                                    },
                                    point: number_of_inidices,
                                });
                        }
                    },
                    _ => {
                        self.general_switches
                            .push(SwitchPoint::BindGroup {
                                bind_group: BindGroups::Texture {
                                    bind_group: texture_bind_group,
                                },
                                point: number_of_inidices,
                            });
                    }
                }
            }
            None => {
                self.general_switches
                    .push(SwitchPoint::BindGroup {
                        bind_group: BindGroups::Texture {
                            bind_group: texture_bind_group,
                        },
                        point: number_of_inidices,
                    });
            }
        };

        self.general_vertices.extend_from_slice(&vertices);
        self.general_indicies.extend_from_slice(&indicies);
    }

    pub(crate) fn add_line(&mut self, start: LineVertex, end: LineVertex) {
        self.line_vertices.push(start);
        self.line_vertices.push(end);
    }

    pub(crate) fn add_regular_n_gon(
        &mut self,
        number_of_sides: u16,
        radius: f32,
        center: [f32; 2],
        colour: Colour,
    ) {
        if number_of_sides < 3 {
            return; // hacky fix for now think of something better later
        }
        let number_of_vertices = self.general_vertices.len() as u16;
        let number_of_inidices = self.general_indicies.len();

        let vertices = (0..number_of_sides)
            .map(|num| {
                (
                    radius * (2.0 * PI * num as f32 / number_of_sides as f32).cos() + center[0],
                    radius * (2.0 * PI * num as f32 / number_of_sides as f32).sin() + center[1],
                )
            })
            .map(|(x, y)| Vertex::from_2d([x, y], [0.0, 0.0], colour.to_raw()))
            .collect::<Vec<Vertex>>();
        // do math on monday idk

        let number_of_triangles = number_of_sides - 2;
        let indicies = (1..number_of_triangles + 1)
            .flat_map(|i| {
                [
                    number_of_vertices,
                    i + 1 + number_of_vertices,
                    i + number_of_vertices,
                ]
            })
            .collect::<Vec<u16>>();

        match self.general_switches.iter().filter(|i| i.is_texture()).last() {
            Some(point) => {
                if !point.is_white_pixel() {
                    self.general_switches
                        .push(SwitchPoint::BindGroup {
                            bind_group: BindGroups::WhitePixel,
                            point: number_of_inidices,
                        });
                }
            }
            None => {
                self.general_switches
                    .push(SwitchPoint::BindGroup {
                        bind_group: BindGroups::WhitePixel,
                        point: number_of_inidices,
                    });
            }
        };

        self.general_vertices.extend_from_slice(&vertices);
        self.general_indicies.extend_from_slice(&indicies);
    }

    pub(crate) fn add_triangle(&mut self, points: [Vertex; 3]) {
        let number_of_verticies = self.general_vertices.len() as u16;
        let number_of_inidices = self.general_indicies.len();
        let indicies = [number_of_verticies, number_of_verticies+1, number_of_verticies+2];
        
        match self.general_switches.iter().filter(|i| i.is_texture()).last() {
            Some(point) => {
                if !point.is_white_pixel() {
                    self.general_switches
                        .push(SwitchPoint::BindGroup {
                            bind_group: BindGroups::WhitePixel,
                            point: number_of_inidices,
                        });
                }
            }
            None => {
                self.general_switches
                    .push(SwitchPoint::BindGroup {
                        bind_group: BindGroups::WhitePixel,
                        point: number_of_inidices,
                    });
            }
        };

        self.general_vertices.extend_from_slice(&points);
        self.general_indicies.extend_from_slice(&indicies);
    }

    pub(crate) fn add_shader_point(&mut self,
        shader_cache: &mut ResourceCache<Shader>,
        shader: &ShaderIndex,
        wgpu: &WgpuClump,
        config: &wgpu::SurfaceConfiguration,
    ) {
        let number_of_inidices = self.general_indicies.len();

        match shader_cache.get_mut(shader.id) {
            Some(cached_shader) => cached_shader.time_since_used = 0,
            None => {
                let new_shader = Shader::from_index(shader, wgpu, config, Some("User_Shader"));
                shader_cache.add_item(new_shader, shader.id);
            }
        }

        self.general_switches.push(SwitchPoint::Shader {
            id: shader.id,
            point: number_of_inidices,
        });
    }

    pub(crate) fn add_shader_option_point(
        &mut self,
        bind_group_cache: &mut ResourceCache<wgpu::BindGroup>,
        options: &ShaderOptions,
        wgpu: &WgpuClump,
    ) {
        let number_of_indices = self.general_indicies.len();

        match bind_group_cache.get_mut(options.id) {
            Some(bind_group) => bind_group.time_since_used = 0,
            None => {
                let bind_group = options.rebuild_bindgroup(wgpu);
                bind_group_cache.add_item(bind_group, options.id);
            }
        }

        self.general_switches.push(SwitchPoint::BindGroup {
            bind_group: BindGroups::ShaderOptions {
                bind_group: options.id,
                group_num: 2,
            },
            point: number_of_indices,
        });
    }

    pub(crate) fn add_text(&mut self, text: Text) {
        self.text.push(text);
    }

    pub(crate) fn add_transfromed_text(&mut self, text: TransformedText) {
        self.transformed_text.push(text);
    }

    pub(crate) fn process_queued(&mut self, device: &wgpu::Device) -> RenderItems {
        let number_of_line_verticies = self.line_vertices.len() as u32;
        let number_of_rectangle_indicies = self.general_indicies.len() as u32;
        let rectangle_vertices = std::mem::take(&mut self.general_vertices);
        let rectangle_indicies = std::mem::take(&mut self.general_indicies);
        let line_verticies = std::mem::take(&mut self.line_vertices);
        let text = std::mem::take(&mut self.text);
        let transformed_text = std::mem::take(&mut self.transformed_text);

        let rectangle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("General Buffer"),
            contents: bytemuck::cast_slice(&rectangle_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let rectangle_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("General Index Buffer"),
            contents: bytemuck::cast_slice(&rectangle_indicies),
            usage: wgpu::BufferUsages::INDEX,
        });

        let line_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Buffer"),
            contents: bytemuck::cast_slice(&line_verticies),
            usage: wgpu::BufferUsages::VERTEX,
        });

        RenderItems {
            rectangle_buffer,
            rectangle_index_buffer,
            number_of_rectangle_indicies,
            general_switches: std::mem::take(&mut self.general_switches),
            line_buffer,
            number_of_line_verticies,
            text,
            transformed_text,
        }
    }
}

pub(crate) struct RenderItems {
    pub(crate) rectangle_buffer: wgpu::Buffer,
    pub(crate) rectangle_index_buffer: wgpu::Buffer,
    pub(crate) number_of_rectangle_indicies: u32,
    pub(crate) general_switches: Vec<SwitchPoint>,
    pub(crate) line_buffer: wgpu::Buffer,
    pub(crate) number_of_line_verticies: u32,
    pub(crate) text: Vec<Text>,
    pub(crate) transformed_text: Vec<TransformedText>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BindGroups {
    WhitePixel,
    Texture{ bind_group: u32 },
    ShaderOptions {
        bind_group: u32,
        group_num: u32,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SwitchPoint {
    BindGroup {
        bind_group: BindGroups,
        point: usize,
    },
    Shader {
        id: u32,
        point: usize,
    }
}

impl SwitchPoint {
    pub fn is_white_pixel(&self) -> bool {
        match self {
            Self::BindGroup {
                bind_group: BindGroups::WhitePixel,
                ..
            } => true,
            _ => false,
        }
    }

    pub fn is_texture(&self) -> bool {
        match self {
            Self::BindGroup { .. } => true,
            _ => false,
        }
    }

    pub fn get_point(&self) -> usize {
        match self {
            Self::BindGroup { point, .. } => *point,
            Self::Shader { point, .. } => *point,
        }
    }
}