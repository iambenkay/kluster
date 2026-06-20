use crate::color;

pub trait DisplayBuffer {
    fn clear(&self, color: &color::Color);
    fn draw_pixel(&self, x: u32, y: u32, color: &color::Color);
    fn draw_rect(&self, x1: u32, y1: u32, x2: u32, y2: u32, color: &color::Color);
    fn draw_triangle(x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, color: &color::Color);
}
