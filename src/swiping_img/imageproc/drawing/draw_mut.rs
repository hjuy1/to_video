#![allow(unused_variables)]
use super::{
    super::{point::Point, rect::Rect as image_Rect},
    draw_if_in_bounds, Canvas,
};
use std::cmp::{max, min};
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

// Implements the Midpoint Ellipse Drawing Algorithm https://web.archive.org/web/20160128020853/http://tutsheap.com/c/mid-point-ellipse-drawing-algorithm/). (Modified from Bresenham's algorithm)
//
// Takes a function that determines how to render the points on the ellipse.
fn draw_ellipse<F>(mut render_func: F, center: (i32, i32), width_radius: i32, height_radius: i32)
where
    F: FnMut(i32, i32, i32, i32),
{
    let (x0, y0) = center;
    let w2 = (width_radius * width_radius) as f32;
    let h2 = (height_radius * height_radius) as f32;
    let mut x = 0;
    let mut y = height_radius;
    let mut px = 0.0;
    let mut py = 2.0 * w2 * y as f32;

    render_func(x0, y0, x, y);

    // Top and bottom regions.
    let mut p = h2 - (w2 * height_radius as f32) + (0.25 * w2);
    while px < py {
        x += 1;
        px += 2.0 * h2;
        if p < 0.0 {
            p += h2 + px;
        } else {
            y -= 1;
            py += -2.0 * w2;
            p += h2 + px - py;
        }

        render_func(x0, y0, x, y);
    }

    // Left and right regions.
    p = h2 * (x as f32 + 0.5).powi(2) + (w2 * (y - 1).pow(2) as f32) - w2 * h2;
    while y > 0 {
        y -= 1;
        py += -2.0 * w2;
        if p > 0.0 {
            p += w2 - py;
        } else {
            x += 1;
            px += 2.0 * h2;
            p += w2 - py + px;
        }

        render_func(x0, y0, x, y);
    }
}

pub trait DrawMut: Canvas
where
    Self: Sized,
{
    fn draw_cubic_bezier_curve_mut(
        &mut self,
        start: (f32, f32),
        end: (f32, f32),
        control_a: (f32, f32),
        control_b: (f32, f32),
        color: Self::Pixel,
    ) {
        // Bezier Curve function from: https://pomax.github.io/bezierinfo/#control
        let cubic_bezier_curve = |t: f32| {
            let t2 = t * t;
            let t3 = t2 * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;
            let x = (start.0 * mt3)
                + (3.0 * control_a.0 * mt2 * t)
                + (3.0 * control_b.0 * mt * t2)
                + (end.0 * t3);
            let y = (start.1 * mt3)
                + (3.0 * control_a.1 * mt2 * t)
                + (3.0 * control_b.1 * mt * t2)
                + (end.1 * t3);
            (x.round(), y.round()) // round to nearest pixel, to avoid ugly line artifacts
        };

        let distance = |point_a: (f32, f32), point_b: (f32, f32)| {
            ((point_a.0 - point_b.0).powi(2) + (point_a.1 - point_b.1).powi(2)).sqrt()
        };

        // Approximate curve's length by adding distance between control points.
        let curve_length_bound: f32 =
            distance(start, control_a) + distance(control_a, control_b) + distance(control_b, end);

        // Use hyperbola function to give shorter curves a bias in number of line segments.
        let num_segments: i32 = ((curve_length_bound.powi(2) + 800.0).sqrt() / 8.0) as i32;

        // Sample points along the curve and connect them with line segments.
        let t_interval = 1f32 / (num_segments as f32);
        let mut t1 = 0f32;
        for i in 0..num_segments {
            let t2 = (i as f32 + 1.0) * t_interval;
            self.draw_line_segment_mut(cubic_bezier_curve(t1), cubic_bezier_curve(t2), color);
            t1 = t2;
        }
    }
    fn draw_filled_circle_mut(&mut self, center: (i32, i32), radius: i32, color: Self::Pixel) {
        let mut x = 0i32;
        let mut y = radius;
        let mut p = 1 - radius;
        let x0 = center.0;
        let y0 = center.1;

        while x <= y {
            self.draw_line_segment_mut(
                ((x0 - x) as f32, (y0 + y) as f32),
                ((x0 + x) as f32, (y0 + y) as f32),
                color,
            );
            self.draw_line_segment_mut(
                ((x0 - y) as f32, (y0 + x) as f32),
                ((x0 + y) as f32, (y0 + x) as f32),
                color,
            );
            self.draw_line_segment_mut(
                ((x0 - x) as f32, (y0 - y) as f32),
                ((x0 + x) as f32, (y0 - y) as f32),
                color,
            );
            self.draw_line_segment_mut(
                ((x0 - y) as f32, (y0 - x) as f32),
                ((x0 + y) as f32, (y0 - x) as f32),
                color,
            );

            x += 1;
            if p < 0 {
                p += 2 * x + 1;
            } else {
                y -= 1;
                p += 2 * (x - y) + 1;
            }
        }
    }

    fn draw_filled_ellipse_mut(
        &mut self,
        center: (i32, i32),
        width_radius: i32,
        height_radius: i32,
        color: Self::Pixel,
    ) {
        // Circle drawing algorithm is faster, so use it if the given ellipse is actually a circle.
        if width_radius == height_radius {
            self.draw_filled_circle_mut(center, width_radius, color);
            return;
        }

        let draw_line_pairs = |x0: i32, y0: i32, x: i32, y: i32| {
            self.draw_line_segment_mut(
                ((x0 - x) as f32, (y0 + y) as f32),
                ((x0 + x) as f32, (y0 + y) as f32),
                color,
            );
            self.draw_line_segment_mut(
                ((x0 - x) as f32, (y0 - y) as f32),
                ((x0 + x) as f32, (y0 - y) as f32),
                color,
            );
        };

        draw_ellipse(draw_line_pairs, center, width_radius, height_radius);
    }
    fn draw_hollow_circle_mut(&mut self, center: (i32, i32), radius: i32, color: Self::Pixel) {
        let mut x = 0i32;
        let mut y = radius;
        let mut p = 1 - radius;
        let x0 = center.0;
        let y0 = center.1;

        while x <= y {
            draw_if_in_bounds(self, x0 + x, y0 + y, color);
            draw_if_in_bounds(self, x0 + y, y0 + x, color);
            draw_if_in_bounds(self, x0 - y, y0 + x, color);
            draw_if_in_bounds(self, x0 - x, y0 + y, color);
            draw_if_in_bounds(self, x0 - x, y0 - y, color);
            draw_if_in_bounds(self, x0 - y, y0 - x, color);
            draw_if_in_bounds(self, x0 + y, y0 - x, color);
            draw_if_in_bounds(self, x0 + x, y0 - y, color);

            x += 1;
            if p < 0 {
                p += 2 * x + 1;
            } else {
                y -= 1;
                p += 2 * (x - y) + 1;
            }
        }
    }
    fn draw_hollow_ellipse_mut(
        &mut self,
        center: (i32, i32),
        width_radius: i32,
        height_radius: i32,
        color: Self::Pixel,
    ) {
        // Circle drawing algorithm is faster, so use it if the given ellipse is actually a circle.
        if width_radius == height_radius {
            self.draw_hollow_circle_mut(center, width_radius, color);
            return;
        }

        let draw_quad_pixels = |x0: i32, y0: i32, x: i32, y: i32| {
            draw_if_in_bounds(self, x0 + x, y0 + y, color);
            draw_if_in_bounds(self, x0 - x, y0 + y, color);
            draw_if_in_bounds(self, x0 + x, y0 - y, color);
            draw_if_in_bounds(self, x0 - x, y0 - y, color);
        };

        draw_ellipse(draw_quad_pixels, center, width_radius, height_radius);
    }
    fn draw_cross_mut(&mut self, color: Self::Pixel, x: i32, y: i32) {
        let (width, height) = self.dimensions();
        let idx = |x, y| (3 * (y + 1) + x + 1) as usize;
        let stencil = [0u8, 1u8, 0u8, 1u8, 1u8, 1u8, 0u8, 1u8, 0u8];

        for sy in -1..2 {
            let iy = y + sy;
            if iy < 0 || iy >= height as i32 {
                continue;
            }

            for sx in -1..2 {
                let ix = x + sx;
                if ix < 0 || ix >= width as i32 {
                    continue;
                }

                if stencil[idx(sx, sy)] == 1u8 {
                    self.draw_pixel(ix as u32, iy as u32, color);
                }
            }
        }
    }
    fn draw_antialiased_line_segment_mut<B>(
        &mut self,
        start: (i32, i32),
        end: (i32, i32),
        color: Self::Pixel,
        blend: B,
    ) where
        B: Fn(Self::Pixel, Self::Pixel, f32) -> Self::Pixel,
    {
    }
    fn draw_line_segment_mut(&mut self, start: (f32, f32), end: (f32, f32), color: Self::Pixel) {
        let (width, height) = self.dimensions();
        let in_bounds = |x, y| x >= 0 && x < width as i32 && y >= 0 && y < height as i32;

        let line_iterator = BresenhamLineIter::new(start, end);

        for point in line_iterator {
            let x = point.0;
            let y = point.1;

            if in_bounds(x, y) {
                self.draw_pixel(x as u32, y as u32, color);
            }
        }
    }

    fn draw_polygon_with_mut<L>(&mut self, poly: &[Point<i32>], color: Self::Pixel, plotter: L)
    where
        L: Fn(&mut Self, (f32, f32), (f32, f32), Self::Pixel),
    {
        if poly.is_empty() {
            return;
        }
        if poly[0] == poly[poly.len() - 1] {
            panic!(
                "First point {:?} == last point {:?}",
                poly[0],
                poly[poly.len() - 1]
            );
        }

        let mut y_min = i32::MAX;
        let mut y_max = i32::MIN;
        for p in poly {
            y_min = min(y_min, p.y);
            y_max = max(y_max, p.y);
        }

        let (width, height) = self.dimensions();

        // Intersect polygon vertical range with image bounds
        y_min = max(0, min(y_min, height as i32 - 1));
        y_max = max(0, min(y_max, height as i32 - 1));

        let mut closed: Vec<Point<i32>> = poly.to_vec();
        closed.push(poly[0]);

        let edges: Vec<&[Point<i32>]> = closed.windows(2).collect();
        let mut intersections = Vec::new();

        for y in y_min..y_max + 1 {
            for edge in &edges {
                let p0 = edge[0];
                let p1 = edge[1];

                if p0.y <= y && p1.y >= y || p1.y <= y && p0.y >= y {
                    if p0.y == p1.y {
                        // Need to handle horizontal lines specially
                        intersections.push(p0.x);
                        intersections.push(p1.x);
                    } else if p0.y == y || p1.y == y {
                        if p1.y > y {
                            intersections.push(p0.x);
                        }
                        if p0.y > y {
                            intersections.push(p1.x);
                        }
                    } else {
                        let fraction = (y - p0.y) as f32 / (p1.y - p0.y) as f32;
                        let inter = p0.x as f32 + fraction * (p1.x - p0.x) as f32;
                        intersections.push(inter.round() as i32);
                    }
                }
            }

            intersections.sort_unstable();
            intersections.chunks(2).for_each(|range| {
                let mut from = min(range[0], width as i32);
                let mut to = min(range[1], width as i32 - 1);
                if from < width as i32 && to >= 0 {
                    // draw only if range appears on the canvas
                    from = max(0, from);
                    to = max(0, to);

                    for x in from..to + 1 {
                        self.draw_pixel(x as u32, y as u32, color);
                    }
                }
            });

            intersections.clear();
        }

        for edge in &edges {
            let start = (edge[0].x as f32, edge[0].y as f32);
            let end = (edge[1].x as f32, edge[1].y as f32);
            plotter(self, start, end, color);
        }
    }

    fn draw_antialiased_polygon_mut<B>(&mut self, poly: &[Point<i32>], color: Self::Pixel, blend: B)
    where
        B: Fn(Self::Pixel, Self::Pixel, f32) -> Self::Pixel,
    {
        self.draw_polygon_with_mut(poly, color, |image, start, end, color| {
            image.draw_antialiased_line_segment_mut(
                (start.0 as i32, start.1 as i32),
                (end.0 as i32, end.1 as i32),
                color,
                &blend,
            )
        });
    }

    fn draw_hallow_polygon_mut(&mut self, poly: &[Point<f32>], color: Self::Pixel) {
        if poly.is_empty() {
            return;
        }
        if poly.len() < 2 {
            panic!(
                "Polygon only has {} points, but at least two are needed.",
                poly.len(),
            );
        }
        if poly[0] == poly[poly.len() - 1] {
            panic!(
                "First point {:?} == last point {:?}",
                poly[0],
                poly[poly.len() - 1]
            );
        }
        for window in poly.windows(2) {
            self.draw_line_segment_mut(
                (window[0].x, window[0].y),
                (window[1].x, window[1].y),
                color,
            );
        }
        let first = poly[0];
        let last = poly.iter().last().unwrap();
        self.draw_line_segment_mut((first.x, first.y), (last.x, last.y), color);
    }

    fn draw_polygon_mut(&mut self, poly: &[Point<i32>], color: Self::Pixel) {
        self.draw_polygon_with_mut(poly, color, |image, start, end, color| {
            image.draw_line_segment_mut(start, end, color)
        });
    }

    fn draw_filled_rect_mut(&mut self, rect: image_Rect, color: Self::Pixel) {
        let canvas_bounds = image_Rect::at(0, 0).of_size(self.width(), self.height());
        if let Some(intersection) = canvas_bounds.intersect(rect) {
            for dy in 0..intersection.height() {
                for dx in 0..intersection.width() {
                    let x = intersection.left() as u32 + dx;
                    let y = intersection.top() as u32 + dy;
                    self.draw_pixel(x, y, color);
                }
            }
        }
    }

    fn draw_filled_rounded_rect_mut(&mut self, rect: image_Rect, radius: i32, color: Self::Pixel) {
        let (left, right, top, bottom) = (rect.left(), rect.right(), rect.top(), rect.bottom());
        // 绘制四个圆角
        self.draw_filled_circle_mut((left + radius, top + radius), radius, color);
        self.draw_filled_circle_mut((left + radius, bottom - radius), radius, color);
        self.draw_filled_circle_mut((right - radius, top + radius), radius, color);
        self.draw_filled_circle_mut((right - radius, bottom - radius), radius, color);

        // 绘制矩形的顶部和底部，去除圆角部分
        self.draw_filled_rect_mut(
            image_Rect::at(left, top + radius)
                .of_size(rect.width(), rect.height() - 2 * radius as u32),
            color,
        );
        // 绘制矩形的左侧和右侧，去除圆角部分
        self.draw_filled_rect_mut(
            image_Rect::at(left + radius, top)
                .of_size(rect.width() - 2 * radius as u32, rect.height()),
            color,
        );
    }

    fn draw_hollow_rect_mut(&mut self, rect: image_Rect, color: Self::Pixel) {
        let left = rect.left() as f32;
        let right = rect.right() as f32;
        let top = rect.top() as f32;
        let bottom = rect.bottom() as f32;

        self.draw_line_segment_mut((left, top), (right, top), color);
        self.draw_line_segment_mut((left, bottom), (right, bottom), color);
        self.draw_line_segment_mut((left, top), (left, bottom), color);
        self.draw_line_segment_mut((right, top), (right, bottom), color);
    }

    fn draw_hollow_rounded_rect_mut(&mut self, rect: image_Rect, radius: i32, color: Self::Pixel) {
        // let left = rect.left() as f32;
        // let right = rect.right() as f32;
        // let top = rect.top() as f32;
        // let bottom = rect.bottom() as f32;

        // self.draw_line_segment_mut((left, top), (right, top), color);
        // self.draw_line_segment_mut((left, bottom), (right, bottom), color);
        // self.draw_line_segment_mut((left, top), (left, bottom), color);
        // self.draw_line_segment_mut((right, top), (right, bottom), color);
    }
}

impl<C: Canvas> DrawMut for C {}
