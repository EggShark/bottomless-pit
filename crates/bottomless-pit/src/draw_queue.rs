use wgpu::util::DeviceExt;

use crate::Vertex;
use crate::LineVertex;
use crate::cache::TextureCache;
use crate::colour::Colour;
use crate::rect::Rectangle;
use crate::rect::TexturedRect;
use std::f32::consts::PI;

#[derive(Debug)]
pub(crate) struct DrawQueues {
    general_vertices: Vec<Vertex>,
    general_indicies: Vec<u16>,
    rectangle_bind_group_switches: Vec<BindGroupSwitchPoint>,
    line_vertices: Vec<LineVertex>,
}

impl DrawQueues {
    pub(crate) fn new() -> Self {
        Self {
            general_vertices: Vec::new(),
            general_indicies: Vec::new(),
            rectangle_bind_group_switches: Vec::new(),
            line_vertices: Vec::new(),
        }
    }

    pub(crate) fn add_rectangle(&mut self, rectangle: &Rectangle) {
        let vertices = rectangle.get_vertices();
        let number_of_verticies = self.general_vertices.len() as u16;
        let number_of_inidices = self.general_indicies.len();
        // do index math
        let indicies = [
            0 + number_of_verticies, 1 + number_of_verticies, 2 + number_of_verticies,
            3 + number_of_verticies, 0 + number_of_verticies, 2 + number_of_verticies,
        ];

        match self.rectangle_bind_group_switches.last() {
            Some(point) => {
                if point.bind_group != BindGroups::WhitePixel {
                    self.rectangle_bind_group_switches.push(BindGroupSwitchPoint {
                        bind_group: BindGroups::WhitePixel,
                        point: 0 + number_of_inidices,
                    });
                }
            },
            None => {
                self.rectangle_bind_group_switches.push(BindGroupSwitchPoint {
                    bind_group: BindGroups::WhitePixel,
                    point: 0 + number_of_inidices,
                });
            },
        };

        self.general_vertices.extend_from_slice(&vertices);
        self.general_indicies.extend_from_slice(&indicies);
    }

    pub(crate) fn add_textured_rectange(&mut self, cache: &mut TextureCache, rectangle: &TexturedRect, device: &wgpu::Device) {
        let vertices = rectangle.get_vertices();
        let texture_bind_group = rectangle.get_texture_id();
        let number_of_verticies = self.general_vertices.len() as u16;
        let number_of_inidices = self.general_indicies.len();
        // do index math
        let indicies = [
            0 + number_of_verticies, 1 + number_of_verticies, 2 + number_of_verticies,
            3 + number_of_verticies, 0 + number_of_verticies, 2 + number_of_verticies,
        ];

        match cache.get_mut(&rectangle.texture) {
            Some(item) => item.time_since_used = 0,
            None => cache.rebuild_from_index(&rectangle.texture, device),
        }

        match self.rectangle_bind_group_switches.last() {
            Some(point) => {
                if point.bind_group != (BindGroups::Custom{bind_group: texture_bind_group}) {
                    self.rectangle_bind_group_switches.push(BindGroupSwitchPoint {
                        bind_group: BindGroups::Custom {bind_group: texture_bind_group},
                        point: 0 + number_of_inidices,
                    });
                }
            },
            None => {
                self.rectangle_bind_group_switches.push(BindGroupSwitchPoint {
                    bind_group: BindGroups::Custom {bind_group: texture_bind_group},
                    point: 0 + number_of_inidices,
                });
            },
        };

        self.general_vertices.extend_from_slice(&vertices);
        self.general_indicies.extend_from_slice(&indicies);
    }

    pub(crate) fn add_line(&mut self, start: LineVertex, end: LineVertex) {
        self.line_vertices.push(start);
        self.line_vertices.push(end);
    }

    pub(crate) fn add_regular_n_gon(&mut self, number_of_sides: u16, radius: f32, center: (f32, f32), colour: Colour) {
        if number_of_sides < 3 {
            return; // hacky fix for now think of something better later
        }
        let number_of_vertices = self.general_vertices.len() as u16;
        let number_of_inidices = self.general_indicies.len();

        let vertices = (0..number_of_sides)
            .map(|num| (radius * (2.0*PI*num as f32/number_of_sides as f32).cos() + center.0, radius * (2.0*PI*num as f32/number_of_sides as f32).sin() + center.1))
            .map(|(x, y)| Vertex::from_2d([x, y], [0.0, 0.0], colour.to_raw()))
            .collect::<Vec<Vertex>>();
        // do math on monday idk

        let number_of_triangles = number_of_sides - 2;
        let indicies = (1..number_of_triangles + 1)
            .flat_map(|i| [number_of_vertices, i + 1 + number_of_vertices, i + number_of_vertices])
            .collect::<Vec<u16>>();

        match self.rectangle_bind_group_switches.last() {
            Some(point) => {
                if point.bind_group != BindGroups::WhitePixel {
                    self.rectangle_bind_group_switches.push(BindGroupSwitchPoint {
                        bind_group: BindGroups::WhitePixel,
                        point: 0 + number_of_inidices,
                    });
                }
            },
            None => {
                self.rectangle_bind_group_switches.push(BindGroupSwitchPoint {
                    bind_group: BindGroups::WhitePixel,
                    point: 0 + number_of_inidices,
                });
            },
        };

        self.general_vertices.extend_from_slice(&vertices);
        self.general_indicies.extend_from_slice(&indicies);
    }

    pub(crate) fn process_queued(&mut self, device: &wgpu::Device) -> RenderItems {
        let number_of_line_verticies = self.line_vertices.len() as u32;
        let number_of_rectangle_indicies = self.general_indicies.len() as u32;
        let rectangle_vertices = std::mem::take(&mut self.general_vertices);
        let rectangle_indicies = std::mem::take(&mut self.general_indicies);
        let line_verticies = std::mem::take(&mut self.line_vertices);

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
            rectangle_bind_group_switches: std::mem::take(&mut self.rectangle_bind_group_switches),
            line_buffer,
            number_of_line_verticies,
        }
    }
}

pub(crate) struct RenderItems {
    pub(crate) rectangle_buffer: wgpu::Buffer,
    pub(crate) rectangle_index_buffer: wgpu::Buffer,
    pub(crate) number_of_rectangle_indicies: u32,
    pub(crate) rectangle_bind_group_switches: Vec<BindGroupSwitchPoint>, 
    pub(crate) line_buffer: wgpu::Buffer,
    pub(crate) number_of_line_verticies: u32,
}

#[derive(Debug, PartialEq)]
pub(crate) struct BindGroupSwitchPoint {
    pub(crate) bind_group: BindGroups,
    pub(crate) point: usize,
}

#[derive(Debug, PartialEq)]
pub(crate) enum BindGroups {
    WhitePixel,
    Custom{bind_group: u32},
}