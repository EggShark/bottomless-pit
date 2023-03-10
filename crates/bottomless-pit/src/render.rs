use crate::DrawQueues;
use crate::engine_handle::DeviceQueue;
use crate::vectors::Vec2;
use crate::colour::Colour; 
use crate::cache::TextureCache;
use crate::camera::Camera;
use crate::BindGroups;
use crate::matrix_math::*;

pub(crate) struct Renderer {
    //add stuff later
    surface: wgpu::Surface,
    white_pixel: wgpu::BindGroup,
    draw_queues: DrawQueues,
    glyph_brush: wgpu_glyph::GlyphBrush<(), wgpu_glyph::ab_glyph::FontArc>,
    pipelines: RenderPipelines,
    camera: Camera,
    clear_colour: Colour,
}

impl Renderer {
    pub(crate) fn render(&mut self, wgpu_things: &DeviceQueue, size: Vec2<u32>, cache: TextureCache) -> Result<(), wgpu::SurfaceError>{
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = wgpu_things.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });
        
        let render_items = self.draw_queues.process_queued(&wgpu_things.device);

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
        
        render_pass.set_pipeline(&self.pipelines.polygon_pipline);
        render_pass.set_bind_group(1, &self.camera.bind_group, &[]);
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
                    &BindGroups::Custom {bind_group} => &cache[bind_group].bind_group,
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
        render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
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
                    &wgpu_things.device, &mut staging_belt, &mut encoder, &view, &self.camera.bind_group, transform,
                ).unwrap();
            });

            text_sections.into_iter().for_each(|s| self.glyph_brush.queue(s));
            self.glyph_brush.draw_queued(&wgpu_things.device, &mut staging_belt, &mut encoder, &view, &self.camera.bind_group, size.x, size.y).unwrap();

        staging_belt.finish();
        wgpu_things.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub(crate) struct  RenderPipelines {
    pub(crate) line_pipeline: wgpu::RenderPipeline,
    pub(crate) polygon_pipline: wgpu::RenderPipeline,
}