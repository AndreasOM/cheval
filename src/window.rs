
use cheval::cheval::Cheval;
use cheval::render_buffer::RenderBuffer;

pub struct WindowFactory {
}

impl WindowFactory {
	pub fn get_default_window_type( ) -> String {
		if cfg!( minifb ) {
			"minifb".to_string()
		} else if cfg!( framebuffer ) {
			"framebuffer".to_string()
		} else {
			"png".to_string()
		}
	}

	pub fn create( window_type: &str, scaling: f32 ) -> Box< dyn Window >{
		match window_type {
			"png" => Box::new( WindowPng::new( scaling ) ),
	#[cfg( minifb )]
			"minifb" => Box::new( WindowMinifb::new() ),
	#[cfg( framebuffer )]
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

#[cfg( minifb )]
mod window_minifb;
#[cfg( minifb )]
use window_minifb::WindowMinifb;

#[cfg( framebuffer )]
mod window_framebuffer;
#[cfg( framebuffer )]
pub use window_framebuffer::WindowFramebuffer;

mod window_png;
pub use window_png::WindowPng;
