use crate::DrawQueues;

pub(crate) struct Renderer {
    //add stuff later
    surface: wgpu::Surface,
    white_pixel: wgpu::BindGroup,
    draw_queues: DrawQueues,
    glyph_brush: wgpu_glyph::GlyphBrush<(), wgpu_glyph::ab_glyph::FontArc>
}

impl Renderer {
    pub(crate) fn render(&mut self, device: &wgpu::Device) {
        todo!()
    }
}

pub(crate) struct  RenderPipelines {
    pub(crate) line_pipeline: wgpu::RenderPipeline,
    pub(crate) polygon_pipline: wgpu::RenderPipeline,
}