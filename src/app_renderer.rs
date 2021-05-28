use direct2d::{
    brush::SolidColorBrush,
    geometry::{Geometry, Path},
    math::{ColorF, Matrix3x2F},
    render_target::{GenericRenderTarget, HwndRenderTarget},
    RenderTarget,
};
use directwrite::{
    enums::{ParagraphAlignment, TextAlignment},
    text_renderer::{self, TextRenderer},
    TextFormat, TextLayout,
};
use std::{ffi::c_void, ptr::null_mut};
use winapi::{
    shared::windef::HWND__,
    um::{
        d2d1::{ID2D1GeometrySink, ID2D1SimplifiedGeometrySink},
        dwrite::{DWRITE_GLYPH_OFFSET, DWRITE_MATRIX},
    },
};

use crate::render_primitives::RenderPrimitives;

pub struct AppRenderer {
    render_target: GenericRenderTarget,
    d2d_factory: direct2d::Factory,
    dwrite_factory: directwrite::Factory,
    hwnd: *mut c_void,
}

impl AppRenderer {
    pub fn new(hwnd: *mut c_void, width: u32, height: u32) -> AppRenderer {
        let dwrite_factory = directwrite::Factory::new().unwrap();
        let d2d_factory = direct2d::factory::Factory::new().unwrap();
        let render_target = AppRenderer::create_render_target(&d2d_factory, hwnd, width, height);

        AppRenderer {
            render_target,
            d2d_factory,
            dwrite_factory,
            hwnd,
        }
    }

    fn create_render_target(
        d2d_factory: &direct2d::Factory,
        hwnd: *mut c_void,
        width: u32,
        height: u32,
    ) -> GenericRenderTarget {
        HwndRenderTarget::create(d2d_factory)
            .with_pixel_size(width, height)
            .with_dpi(96.0, 96.0) // Non scale dpi
            .with_hwnd(hwnd as *mut HWND__)
            .build()
            .unwrap()
            .as_generic()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.render_target =
            AppRenderer::create_render_target(&self.d2d_factory, self.hwnd, width, height);
    }

    pub fn draw(&mut self, render_primitives: &[RenderPrimitives]) {
        self.render_target.begin_draw();

        for p in render_primitives {
            match p {
                RenderPrimitives::Clear { color } => {
                    self.render_target.clear(*color);
                }
                RenderPrimitives::VertCenteredText {
                    text,
                    rect,
                    color_fill,
                    color_stroke,
                    weight_stroke,
                    size_font,
                    size_space,
                } => {
                    let t_format = TextFormat::create(&self.dwrite_factory)
                        .with_size(*size_font)
                        .with_family("Lucida Sans")
                        .build()
                        .unwrap();

                    t_format
                        .set_paragraph_alignment(ParagraphAlignment::Center)
                        .unwrap();
                    t_format.set_text_alignment(TextAlignment::Center).unwrap();

                    let text_layout = TextLayout::create(&self.dwrite_factory)
                        .with_text(&text)
                        .with_size(rect.2, rect.3)
                        .with_font(&t_format)
                        .with_centered(true)
                        .build()
                        .unwrap();

                    text_layout
                        .draw(
                            &mut OutlinedTextRenderer {
                                d2d_factory: &mut self.d2d_factory,
                                render_target: &mut self.render_target,
                                fill_color: (*color_fill).into(),
                                stroke_color: (*color_stroke).into(),
                                stroke_weight: *weight_stroke,
                                space: *size_space,
                            },
                            rect.0,
                            rect.1,
                            text_renderer::Context(null_mut()),
                        )
                        .unwrap();
                }
            }
        }

        self.render_target.end_draw().unwrap();
    }
}

struct OutlinedTextRenderer<'a> {
    d2d_factory: &'a mut direct2d::Factory,
    render_target: &'a mut GenericRenderTarget,
    fill_color: ColorF,
    stroke_color: ColorF,
    stroke_weight: f32,
    space: f32,
}

impl TextRenderer for OutlinedTextRenderer<'_> {
    fn current_transform(
        &self,
        _: text_renderer::Context,
    ) -> directwrite::error::DWResult<winapi::um::dwrite::DWRITE_MATRIX> {
        Ok(DWRITE_MATRIX {
            m11: 0.0,
            m12: 0.0,
            m21: 0.0,
            m22: 0.0,
            dx: 0.0,
            dy: 0.0,
        })
    }

    fn pixels_per_dip(&self, _: text_renderer::Context) -> directwrite::error::DWResult<f32> {
        Ok(1.0)
    }

    fn is_pixel_snapping_disabled(
        &self,
        _: text_renderer::Context,
    ) -> directwrite::error::DWResult<bool> {
        Ok(false)
    }

    fn draw_glyph_run(
        &mut self,
        context: &text_renderer::DrawGlyphRun,
    ) -> directwrite::error::DWResult<()> {
        let path = Path::create(&self.d2d_factory).unwrap();
        let mut psink: *mut ID2D1GeometrySink = null_mut();
        unsafe {
            (*path.get_raw()).Open(&mut psink);
        };
        let mut offsets = Vec::with_capacity(context.glyph_offsets.len());
        for (i, _) in context.glyph_offsets.iter().enumerate() {
            offsets.push(DWRITE_GLYPH_OFFSET {
                advanceOffset: (i as f32) * self.space
                    - ((context.glyph_offsets.len() as f32 - 1.0) * self.space) / 2.0,
                ascenderOffset: 0.0,
            });
        }
        context
            .font_face
            .get_glyph_run_outline(
                context.font_em_size,
                context.glyph_indices,
                Some(context.glyph_advances),
                Some(&offsets),
                context.is_sideways,
                context.bidi_level % 2 == 1,
                psink as *mut ID2D1SimplifiedGeometrySink,
            )
            .unwrap();

        let matrix = Matrix3x2F::new([
            [1.0, 0.0],
            [0.0, 1.0],
            [context.baseline_origin_x, context.baseline_origin_y],
        ]);

        let transformed = path.transformed(&matrix).unwrap();

        let brush = SolidColorBrush::create(self.render_target)
            .with_color(self.fill_color)
            .build()
            .unwrap();
        self.render_target.fill_geometry(&transformed, &brush);

        let brush = SolidColorBrush::create(self.render_target)
            .with_color(self.stroke_color)
            .build()
            .unwrap();
        self.render_target
            .draw_geometry(&transformed, &brush, self.stroke_weight, None);

        Ok(())
    }

    fn draw_inline_object(
        &mut self,
        _: &text_renderer::DrawInlineObject,
    ) -> directwrite::error::DWResult<()> {
        Ok(())
    }

    fn draw_strikethrough(
        &mut self,
        _: &text_renderer::DrawStrikethrough,
    ) -> directwrite::error::DWResult<()> {
        Ok(())
    }

    fn draw_underline(
        &mut self,
        _: &text_renderer::DrawUnderline,
    ) -> directwrite::error::DWResult<()> {
        Ok(())
    }
}
