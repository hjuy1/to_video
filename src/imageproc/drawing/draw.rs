use super::{
    super::{definitions::Image, point::Point, rect::Rect},
    DrawMut,
};
use image::{GenericImage, ImageBuffer};
use std::mem::swap;

/// Iterates over the coordinates in a line segment using
/// [Bresenham's line drawing algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm).
pub struct BresenhamLineIter {
    dx: f32,
    dy: f32,
    x: i32,
    y: i32,
    error: f32,
    end_x: i32,
    is_steep: bool,
    y_step: i32,
}

impl BresenhamLineIter {
    /// Creates a [`BresenhamLineIter`](struct.BresenhamLineIter.html) which will iterate over the integer coordinates
    /// between `start` and `end`.
    pub fn new(start: (f32, f32), end: (f32, f32)) -> BresenhamLineIter {
        let (mut x0, mut y0) = (start.0, start.1);
        let (mut x1, mut y1) = (end.0, end.1);

        let is_steep = (y1 - y0).abs() > (x1 - x0).abs();
        if is_steep {
            swap(&mut x0, &mut y0);
            swap(&mut x1, &mut y1);
        }

        if x0 > x1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
        }

        let dx = x1 - x0;

        BresenhamLineIter {
            dx,
            dy: (y1 - y0).abs(),
            x: x0 as i32,
            y: y0 as i32,
            error: dx / 2f32,
            end_x: x1 as i32,
            is_steep,
            y_step: if y0 < y1 { 1 } else { -1 },
        }
    }
}

impl Iterator for BresenhamLineIter {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<(i32, i32)> {
        if self.x > self.end_x {
            None
        } else {
            let ret = if self.is_steep {
                (self.y, self.x)
            } else {
                (self.x, self.y)
            };

            self.x += 1;
            self.error -= self.dy;
            if self.error < 0f32 {
                self.y += self.y_step;
                self.error += self.dx;
            }

            Some(ret)
        }
    }
}

pub trait Draw: GenericImage
where
    Self: Sized,
{
    fn draw_cubic_bezier_curve(
        &self,
        start: (f32, f32),
        end: (f32, f32),
        control_a: (f32, f32),
        control_b: (f32, f32),
        color: Self::Pixel,
    ) -> Image<Self::Pixel> {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_cubic_bezier_curve_mut(start, end, control_a, control_b, color);
        out
    }

    fn draw_filled_circle(
        &self,
        center: (i32, i32),
        radius: i32,
        color: Self::Pixel,
    ) -> Image<Self::Pixel> {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_filled_circle_mut(center, radius, color);
        out
    }

    fn draw_filled_ellipse(
        &self,
        center: (i32, i32),
        width_radius: i32,
        height_radius: i32,
        color: Self::Pixel,
    ) -> Image<Self::Pixel> {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_filled_ellipse_mut(center, width_radius, height_radius, color);
        out
    }

    fn draw_hollow_circle(
        &self,
        center: (i32, i32),
        radius: i32,
        color: Self::Pixel,
    ) -> Image<Self::Pixel> {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_hollow_circle_mut(center, radius, color);
        out
    }

    fn draw_hollow_ellipse(
        &self,
        center: (i32, i32),
        width_radius: i32,
        height_radius: i32,
        color: Self::Pixel,
    ) -> Image<Self::Pixel> {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_hollow_ellipse_mut(center, width_radius, height_radius, color);
        out
    }

    fn draw_cross(&self, color: Self::Pixel, x: i32, y: i32) -> Image<Self::Pixel> {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_cross_mut(color, x, y);
        out
    }

    fn draw_antialiased_line_segment<B>(
        &self,
        start: (i32, i32),
        end: (i32, i32),
        color: Self::Pixel,
        blend: B,
    ) -> Image<Self::Pixel>
    where
        B: Fn(Self::Pixel, Self::Pixel, f32) -> Self::Pixel,
    {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_antialiased_line_segment_mut(start, end, color, blend);
        out
    }

    fn draw_line_segment(
        &self,
        start: (f32, f32),
        end: (f32, f32),
        color: Self::Pixel,
    ) -> Image<Self::Pixel> {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_line_segment_mut(start, end, color);
        out
    }

    fn draw_polygon_with<L>(
        &self,
        poly: &[Point<i32>],
        color: Self::Pixel,
        plotter: L,
    ) -> Image<Self::Pixel>
    where
        L: Fn(&mut Image<Self::Pixel>, (f32, f32), (f32, f32), Self::Pixel),
    {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_polygon_with_mut(poly, color, plotter);
        out
    }

    fn draw_antialiased_polygon<B>(
        &self,
        poly: &[Point<i32>],
        color: Self::Pixel,
        blend: B,
    ) -> Image<Self::Pixel>
    where
        B: Fn(Self::Pixel, Self::Pixel, f32) -> Self::Pixel,
    {
        self.draw_polygon_with(poly, color, |image, start, end, color| {
            image.draw_antialiased_line_segment_mut(
                (start.0 as i32, start.1 as i32),
                (end.0 as i32, end.1 as i32),
                color,
                &blend,
            )
        })
    }

    fn draw_hallow_polygon(&self, poly: &[Point<f32>], color: Self::Pixel) -> Image<Self::Pixel> {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_hallow_polygon_mut(poly, color);
        out
    }

    fn draw_polygon(&self, poly: &[Point<i32>], color: Self::Pixel) -> Image<Self::Pixel> {
        self.draw_polygon_with(poly, color, |image, start, end, color| {
            image.draw_line_segment_mut(start, end, color)
        })
    }

    fn draw_filled_rect(&self, rect: Rect, color: Self::Pixel) -> Image<Self::Pixel> {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_filled_rect_mut(rect, color);
        out
    }

    fn draw_filled_rounded_rect(
        &self,
        rect: Rect,
        radius: i32,
        color: Self::Pixel,
    ) -> Image<Self::Pixel> {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_filled_rounded_rect_mut(rect, radius, color);
        out
    }

    fn draw_hollow_rect(&self, rect: Rect, color: Self::Pixel) -> Image<Self::Pixel> {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_hollow_rect_mut(rect, color);
        out
    }

    fn draw_hollow_rounded_rect(
        &self,
        rect: Rect,
        radius: i32,
        color: Self::Pixel,
    ) -> Image<Self::Pixel> {
        let mut out = ImageBuffer::new(self.width(), self.height());
        out.copy_from(self, 0, 0).unwrap();
        out.draw_hollow_rounded_rect_mut(rect, radius, color);
        out
    }
}

impl<I: GenericImage> Draw for I {}
