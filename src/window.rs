
use cheval::cheval::Cheval;

#[cfg(target_arch = "x86_64")]
pub struct Window {
	window: WindowMinifb,
}

#[cfg(target_arch = "arm")]
pub struct Window {
	window: WindowFramebuffer,
}

impl Window {
	#[cfg(target_arch = "x86_64")]
	pub fn new() -> Self {
		Self {
			window: WindowMinifb::new(),
		}
	}

	#[cfg(target_arch = "arm")]
	pub fn new() -> Self {
		Self {
			window: WindowFramebuffer::new(),
		}
	}

	pub fn done( &self ) -> bool {
		self.window.done()
	}

	pub fn render_frame( &mut self, func: &mut dyn FnMut( &mut Vec<u32>, usize, usize, &Cheval ), cheval: &Cheval  ) {
		self.window.render_frame( func, cheval )
	}

	pub fn next_frame( &mut self ) {
		self.window.next_frame()
	}
}

#[cfg(target_arch = "x86_64")]
mod window_minifb;
#[cfg(target_arch = "x86_64")]
use window_minifb::WindowMinifb;


#[cfg(target_arch = "arm")]
mod window_framebuffer;
#[cfg(target_arch = "arm")]
pub use window_framebuffer::WindowFramebuffer;
