use crate::cheval::Cheval;
use crate::render_buffer::RenderBuffer;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum WindowMode {
	RGB,
	RGB_A,
	R_G_B,
	R_G_B_A,
	A,
}

impl Into<WindowMode> for &str {
	fn into(self) -> WindowMode {
		match self {
			"RGB" => WindowMode::RGB,
			"RGB_A" => WindowMode::RGB_A,
			"R_G_B" => WindowMode::R_G_B,
			"R_G_B_A" => WindowMode::R_G_B_A,
			"A" => WindowMode::A,
			e => {
				panic!("Unsupported window mode {}", e);
			},
		}
	}
}

impl Into<&str> for WindowMode {
	fn into(self) -> &'static str {
		match self {
			WindowMode::RGB => "RGB",
			e => {
				panic!("Unsupported window mode {:?}", e);
			},
		}
	}
}

pub struct WindowFactory {}

impl WindowFactory {
	pub fn get_default_window_type() -> String {
		if cfg!(minifb) {
			"minifb".to_string()
		} else if cfg!(framebuffer) {
			"framebuffer".to_string()
		} else {
			"png".to_string()
		}
	}

	pub fn create(
		window_title: &str,
		window_type: &str,
		window_mode: &WindowMode,
		scaling: f32,
	) -> Box<dyn Window> {
		match window_type {
			"png" => Box::new(WindowPng::new(scaling)),
			#[cfg(minifb)]
			"minifb" => Box::new(WindowMinifb::new(&window_title, &window_mode)),
			#[cfg(framebuffer)]
			"framebuffer" => Box::new(WindowFramebuffer::new()),
			_ => panic!("window type not supported {:?}", &window_type),
		}
	}
}

pub trait Window {
	fn done(&self) -> bool;
	fn render_frame(
		&mut self,
		func: &mut dyn FnMut(&mut RenderBuffer, &mut Cheval),
		cheval: &mut Cheval,
	);
	fn next_frame(&mut self);
	fn get_key(&mut self) -> Option<u32>;
	fn restore_positions(&mut self, _filename: &str) {}
	fn store_positions(&self, _filename: &str) {}
}

#[cfg(minifb)]
pub mod window_minifb;
#[cfg(minifb)]
pub use window_minifb::WindowMinifb;

#[cfg(framebuffer)]
mod window_framebuffer;
#[cfg(framebuffer)]
pub use window_framebuffer::WindowFramebuffer;

mod window_png;
pub use window_png::WindowPng;

mod window_layout;
use window_layout::WindowLayout;
use window_layout::WindowLayoutWindowConfig;
