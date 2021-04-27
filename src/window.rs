
use cheval::cheval::Cheval;
use cheval::render_buffer::RenderBuffer;

pub struct WindowFactory {
}

impl WindowFactory {
	#[cfg(target_arch = "x86_64")]
	pub fn get_default_window_type( ) -> String {
		"minifb".to_string()
	}
	#[cfg(target_arch = "aarch64")]
	pub fn get_default_window_type( ) -> String {
		"minifb".to_string()
	}
	#[cfg(target_arch = "arm")]
	pub fn get_default_window_type( ) -> String {
		"framebuffer".to_string()
	}


	pub fn create( window_type: &str, scaling: f32 ) -> Box< dyn Window >{
		match window_type {
			"png" => Box::new( WindowPng::new( scaling ) ),
	#[cfg(target_arch = "x86_64")]
			"minifb" => Box::new( WindowMinifb::new() ),
	#[cfg(target_arch = "aarch64")]
			"minifb" => Box::new( WindowMinifb::new() ),
	#[cfg(target_arch = "arm")]
			"framebuffer" => Box::new( WindowFramebuffer::new() ),
			_ => panic!("window type not supported {:?}", &window_type ),
		}
	}
}

pub trait Window {
	fn done( &self ) -> bool;
	fn render_frame( &mut self, func: &mut dyn FnMut( &mut RenderBuffer, &mut Cheval ), cheval: &mut Cheval  );
	fn next_frame( &mut self );
	fn get_key( &mut self ) -> Option< u32 >;
}

#[cfg(target_arch = "x86_64")]
mod window_minifb;
#[cfg(target_arch = "x86_64")]
use window_minifb::WindowMinifb;

#[cfg(target_arch = "aarch64")]
mod window_minifb;
#[cfg(target_arch = "aarch64")]
use window_minifb::WindowMinifb;


#[cfg(target_arch = "arm")]
mod window_framebuffer;
#[cfg(target_arch = "arm")]
pub use window_framebuffer::WindowFramebuffer;

mod window_png;
pub use window_png::WindowPng;
