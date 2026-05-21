use crate::board_core::screen;

pub fn entrypoint() -> ! {
    if screen::init() {
        screen::draw_rect(100, 100, 500, 500, 0x0000_00FF);

        // Overlapping triangles in different colors.
        screen::draw_triangle(300, 150, 150, 550, 600, 450, 0x00FF_0000); // red
        screen::draw_triangle(450, 200, 750, 600, 250, 600, 0x0000_FF00); // green
        screen::draw_triangle(550, 100, 850, 500, 350, 400, 0x00FFFF_00); // yellow
        screen::draw_triangle(200, 400, 700, 250, 650, 700, 0x0000_FFFF); // cyan
    }

    loop {}
}
