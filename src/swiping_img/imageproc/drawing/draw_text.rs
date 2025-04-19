use super::{
    super::{definitions::Clamp, pixelops::weighted_sum, rect},
    Canvas,
};
use ab_glyph::{point, Font, GlyphId, OutlinedGlyph, PxScale, ScaleFont};
use image::Pixel;

fn layout_glyphs(
    scale: impl Into<PxScale> + Copy,
    font: &impl Font,
    text: &str,
    mut f: impl FnMut(OutlinedGlyph, ab_glyph::Rect),
) -> (u32, u32) {
    let (mut w, mut h) = (0f32, 0f32);

    let font = font.as_scaled(scale);
    let mut last: Option<GlyphId> = None;

    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        let glyph = glyph_id.with_scale_and_position(scale, point(w, font.ascent()));
        w += font.h_advance(glyph_id);
        if let Some(g) = font.outline_glyph(glyph) {
            if let Some(last) = last {
                w += font.kern(glyph_id, last);
            }
            last = Some(glyph_id);
            let bb = g.px_bounds();
            h = h.max(bb.height());
            f(g, bb);
        }
    }

    (w as u32, h as u32)
}

/// Get the width and height of the given text, rendered with the given font and scale.
///
/// Note that this function *does not* support newlines, you must do this manually.
pub fn text_size(scale: impl Into<PxScale> + Copy, font: &impl Font, text: &str) -> (u32, u32) {
    layout_glyphs(scale, font, text, |_, _| {})
}

pub trait DrawText: Canvas
where
    <Self::Pixel as Pixel>::Subpixel: Into<f32> + Clamp<f32>,
{
    fn draw_text() {}

    fn draw_text_mut(
        &mut self,
        color: Self::Pixel,
        x: i32,
        y: i32,
        scale: impl Into<PxScale> + Copy,
        font: &impl Font,
        text: &str,
    ) {
        let image_width = self.width() as i32;
        let image_height = self.height() as i32;

        layout_glyphs(scale, font, text, |g, bb| {
            g.draw(|gx, gy, gv| {
                let image_x = gx as i32 + x + bb.min.x.round() as i32;
                let image_y = gy as i32 + y + bb.min.y.round() as i32;
                let gv = gv.clamp(0.0, 1.0);

                if (0..image_width).contains(&image_x) && (0..image_height).contains(&image_y) {
                    let image_x = image_x as u32;
                    let image_y = image_y as u32;
                    let pixel = self.get_pixel(image_x, image_y);
                    let weighted_color = weighted_sum(pixel, color, 1.0 - gv, gv);
                    self.draw_pixel(image_x, image_y, weighted_color);
                }
            })
        });
    }

    fn draw_text_center_mut(
        &mut self,
        color: Self::Pixel,
        rect: rect::Rect,
        scale: impl Into<PxScale> + Copy,
        font: &impl Font,
        text: &str,
    ) {
        // 将文本按行分割并去除每行的前后空格
        let lines: Vec<&str> = text.lines().map(str::trim).collect();

        // 计算文本原始高度
        let row = u32::try_from(lines.len()).unwrap();
        let text_raw_height = row * font.as_scaled(scale).height() as u32;

        // 计算文本原始宽度
        let text_raw_width = lines
            .iter()
            .map(|line| text_size(scale, &font, line).0)
            .max()
            .unwrap_or(0);

        // 解构矩形区域
        let (rect_left, rect_top, rect_width, rect_height) =
            (rect.left(), rect.top(), rect.width(), rect.height());

        // 根据矩形区域和文本原始尺寸计算最终字体大小
        let scale = if text_raw_width > rect_width || text_raw_height > rect_height {
            let x_radio = rect_width as f32 / text_raw_width as f32;
            let y_radio = rect_height as f32 / text_raw_height as f32;
            PxScale::from(scale.into().x * (x_radio.min(y_radio)))
        } else {
            scale.into()
        };

        // 重新计算文本高度
        let h = font.as_scaled(scale).height() as u32;

        // 计算文本顶部位置
        let top_ = rect_top + i32::try_from(rect_height - h * row).unwrap() / 2;

        // 遍历每行文本并绘制
        for (row, line) in lines.iter().enumerate() {
            self.draw_text_mut(
                color,
                rect_left
                    + i32::try_from((rect_width - text_size(scale, font, line).0) / 2).unwrap(),
                top_ + i32::try_from(h).unwrap() * i32::try_from(row).unwrap(),
                scale,
                font,
                line,
            );
        }
    }
}

impl<C: Canvas> DrawText for C where <C::Pixel as Pixel>::Subpixel: Into<f32> + Clamp<f32> {}
