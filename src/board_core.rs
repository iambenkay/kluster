#[path = "__board__/raspberrypi/cpu.rs"]
mod cpu;

#[path = "__board__/raspberrypi/screen.rs"]
pub mod screen;

#[path = "__board__/raspberrypi/console.rs"]
mod console;

pub use console::console;
