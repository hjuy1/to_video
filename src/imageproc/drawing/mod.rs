mod canvas;
mod draw;
mod draw_mut;
mod draw_text;

#[allow(unused_imports)]
pub use self::{
    canvas::Canvas,
    draw::Draw,
    draw_mut::DrawMut,
    draw_text::{text_size, DrawText},
};

// Set pixel at (x, y) to color if this point lies within image bounds,
// otherwise do nothing.
fn draw_if_in_bounds<C>(canvas: &mut C, x: i32, y: i32, color: C::Pixel)
where
    C: Canvas,
{
    if x >= 0 && x < canvas.width() as i32 && y >= 0 && y < canvas.height() as i32 {
        canvas.draw_pixel(x as u32, y as u32, color);
    }
}
