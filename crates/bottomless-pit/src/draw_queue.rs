use wgpu::util::DeviceExt;

use crate::Vertex;
use crate::LineVertex;
use crate::rect::Rectangle;
use crate::rect::TexturedRect;

pub(crate) struct DrawQueues<'a> {
    rect_vertices: Vec<Vertex>,
    rect_indicies: Vec<u16>,
    rectangle_bind_group_switches: Vec<BindGroupSwitchPoint<'a>>,
    line_vertices: Vec<LineVertex>,
}

impl<'a> DrawQueues<'a> {
    pub(crate) fn new() -> Self {
        Self {
            rect_vertices: Vec::new(),
            rect_indicies: Vec::new(),
            rectangle_bind_group_switches: Vec::new(),
            line_vertices: Vec::new(),
        }
    }

    pub(crate) fn new_with_data(rect_verts: Option<Vec<Vertex>>, rect_indicies: Option<Vec<u16>>, rectangle_bind_group_switches: Option<Vec<BindGroupSwitchPoint<'a>>>, line_vertexes: Option<Vec<LineVertex>>) -> Self {
        let rect_vertices = rect_verts.unwrap_or(Vec::new());
        let rect_indicies = rect_indicies.unwrap_or(Vec::new());
        let rectangle_bind_group_switches = rectangle_bind_group_switches.unwrap_or(Vec::new());
        let line_vertices = line_vertexes.unwrap_or(Vec::new());
        Self {
            rect_vertices,
            rect_indicies,
            rectangle_bind_group_switches,
            line_vertices,
        }
    }

    // pub const RECT_INDICIES: &[u16] = &[
    //     0, 1, 2,
    //     3, 0, 2,
    // ];

    pub(crate) fn add_rectangle(&mut self, rectangle: &Rectangle) {
        let vertices = rectangle.get_vertices();
        let number_of_rectanges = self.rect_vertices.len() as u16 % 4;
        // do index math
        let indicies = [
            0 + (4 * number_of_rectanges), 1 + (4 * number_of_rectanges), 2 + (4 * number_of_rectanges),
            3 + (4 * number_of_rectanges), 0 + (4 * number_of_rectanges), 2 + (4 * number_of_rectanges),
        ];

        let last_bind_group = &self.rectangle_bind_group_switches[self.rectangle_bind_group_switches.len() - 1].bind_group;
        if self.rectangle_bind_group_switches.is_empty() || last_bind_group != &BindGroups::WhitePixel {
            self.rectangle_bind_group_switches.push(BindGroupSwitchPoint {
                bind_group: BindGroups::WhitePixel,
                point: number_of_rectanges.into(),
            });
        }

        self.rect_vertices.extend_from_slice(&vertices);
        self.rect_indicies.extend_from_slice(&indicies);
    }

    pub(crate) fn add_textured_rectange(&mut self, rectangle: &'a TexturedRect) {
        let vertices = rectangle.get_vertices();
        let texture_bind_group = rectangle.get_bind_group();
        let number_of_rectanges = self.rect_vertices.len() as u16 % 4;
        // do index math
        let indicies = [
            0 + (4 * number_of_rectanges), 1 + (4 * number_of_rectanges), 2 + (4 * number_of_rectanges),
            3 + (4 * number_of_rectanges), 0 + (4 * number_of_rectanges), 2 + (4 * number_of_rectanges),
        ];

        self.rectangle_bind_group_switches.push(BindGroupSwitchPoint {
            bind_group: BindGroups::Custom {bind_group: texture_bind_group},
            point: number_of_rectanges.into()
        });

        self.rect_vertices.extend_from_slice(&vertices);
        self.rect_indicies.extend_from_slice(&indicies);
    }

    pub(crate) fn add_line(&mut self, start: LineVertex, end: LineVertex) {
        self.line_vertices.push(start);
        self.line_vertices.push(end);
    }

    pub(crate) fn process_queued(&mut self, device: wgpu::Device) -> RenderItems {
        let rectangle_vertices = std::mem::take(&mut self.rect_vertices);
        let rectangle_indicies = std::mem::take(&mut self.rect_indicies);
        let line_verticies = std::mem::take(&mut self.line_vertices);

        let rectangle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("General Buffer"),
            contents: bytemuck::cast_slice(&rectangle_vertices),
            usage: wgpu::BufferUsages::VERTEX
        });

        let rectangle_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("General Index Buffer"),
            contents: bytemuck::cast_slice(&rectangle_indicies),
            usage: wgpu::BufferUsages::VERTEX
        });

        let line_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Buffer"),
            contents: bytemuck::cast_slice(&line_verticies),
            usage: wgpu::BufferUsages::VERTEX
        });

        RenderItems {
            rectangle_buffer,
            rectangle_index_buffer,
            rectangle_bind_group_switches: std::mem::take(&mut self.rectangle_bind_group_switches),
            line_buffer
        }
    }
}

pub(crate) struct RenderItems<'a> {
    pub(crate) rectangle_buffer: wgpu::Buffer,
    pub(crate) rectangle_index_buffer: wgpu::Buffer,
    pub(crate) rectangle_bind_group_switches: Vec<BindGroupSwitchPoint<'a>>, 
    pub(crate) line_buffer: wgpu::Buffer,
}

pub(crate) struct BindGroupSwitchPoint<'a> {
    pub(crate) bind_group: BindGroups<'a>,
    pub(crate) point: usize,
}

pub(crate) enum BindGroups<'a> {
    WhitePixel,
    Custom{bind_group: &'a wgpu::BindGroup},
}

// this is a really shitty implmentation of this espcially for the custom part
// but I really only need this to check if the previous value was a WhitePixel
impl PartialEq for BindGroups<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::WhitePixel, Self::WhitePixel) => true,
            (Self::Custom{..}, Self::Custom{..}) => true,
            _ => false,
        }
    }
}