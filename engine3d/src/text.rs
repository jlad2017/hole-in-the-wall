use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};

pub struct Text {
    glyph_brush: wgpu_glyph::GlyphBrush<'a, (), twox_hash::RandomXxHashBuilder64>,
}

impl Text {
    pub fn new(font_path: &str) -> Self {
        let font = ab_glyph::FontArc::try_from_slice(include_bytes!(path))?;
        let glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, render_format);
    }

    pub fn queue(&self, text: &str, pos: (f32, f32), color: [f32; 4], scale: f32) {
        let text = Text::new(text).with_color(color).with_scale(scale);

        self.glyph_brush.queue(Section {
            screen_position: pos,
            // bounds: (size.width as f32, size.height as f32),
            text: vec![text],
            ..Section::default()
        });
    }

    pub fn render_queued(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::utilStagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &TextureView,
        size: winit::dpi::PhysicalSize<u32>,
    ) {
        self.glyph_brush
            .draw_queued(
                device,
                staging_belt,
                encoder,
                frame_view
                size.width,
                size.height,
            )
            .expect("Draw queued");
    }

    /* https://github.com/MichaelBaker/wgpu-glyph-example */
    pub fn load_font<'a>(
        path: &str,
        device: &wgpu::Device,
        render_format: wgpu::TextureFormat,
    ) -> wgpu_glyph::GlyphBrush<'a, (), twox_hash::RandomXxHashBuilder64> {
    }
}
