use minifb;

pub struct Window {
	width: usize,
	height: usize,
	buffer: Vec<u32>,
	window: minifb::Window,
}

impl Window {
	pub fn new() -> Self {
		let mut s = Self {
			width: 640,
			height: 360,
			buffer: vec![0; 640 * 360], //Vec::new(),
			window: minifb::Window::new(
				"Test",
				640,
				360,
				minifb::WindowOptions::default()
			).unwrap_or_else(|e| {
        		panic!("{}", e);
    		}),
		};
		s.window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
		s
	}

	pub fn done( &self ) -> bool {
		!( self.window.is_open() && !self.window.is_key_down(minifb::Key::Escape) )
	}
	pub fn render_frame( &mut self, func: &mut dyn FnMut( &mut Vec<u32>, usize, usize )  ) {
		func( &mut self.buffer, self.width, self.height );
	}
	pub fn next_frame( &mut self ) {
        self.window
            .update_with_buffer(&self.buffer, self.width, self.height )
            .unwrap();
	}
}

