use framebuffer::Framebuffer;

use cheval::cheval::Cheval;
use crate::window::Window;
use cheval::render_buffer::RenderBuffer;

pub struct WindowFramebuffer {
	render_buffer: RenderBuffer,
	frame: Vec<u8>,
	framebuffer: Framebuffer,
}

impl WindowFramebuffer {
	pub fn new() -> Self {
		let framebuffer = Framebuffer::new("/dev/fb0").unwrap_or_else(|e| {
			panic!("{}", e)
		});
		let width = framebuffer.var_screen_info.xres as usize;
		let height = framebuffer.var_screen_info.yres as usize;
		let line_length = framebuffer.fix_screen_info.line_length as usize;
		let frame = vec![0u8; line_length * height];
		let render_buffer = RenderBuffer::new( width, height );
		let s = Self {
			render_buffer: render_buffer,
			frame: frame,
			framebuffer: framebuffer,
		};
//		dbg!(&s);
		s
	}	
}

impl Window for WindowFramebuffer {

	fn done( &self ) -> bool {
		false
	}
	fn render_frame( &mut self, func: &mut dyn FnMut( &mut RenderBuffer, &mut Cheval ), cheval: &mut Cheval  ) {
		func( &mut self.render_buffer, cheval );
	}
	fn next_frame( &mut self ) {
		for y in 0..self.render_buffer.height {
			for x in 0..self.render_buffer.width {
				let o = y * self.render_buffer.width + x;
				let argb = self.render_buffer.buffer[ o ];

//				let a = ( argb >> 24 ) as u8;
				let r = ( argb >> 16 ) as u8;
				let g = ( argb >>  8 ) as u8;
				let b = ( argb >>  0 ) as u8;

				let o = y * ( self.framebuffer.fix_screen_info.line_length as usize / 2 ) + x;
//				dbg!(o, y, x);

				// rrrr rrrr
				// 1111 1000
				//    f    8

				// gggg gggg
				// 1111 1100
				//    f    c

				// bbbb bbbb
				// 1111 1000
				//    f    8

				// rrrr rggg gggb bbbb


				let rgb565: u16 =
					( ( ( r as u16 ) & 0xf8 ) << 8 )
					| ( ( ( g as u16 ) & 0xfc ) << 3 )
					| ( ( ( b as u16 ) & 0xf8 ) >> 3 )
				;

				let hb = ( ( rgb565 >> 8 ) & 0xff ) as u8;
				let lb = ( rgb565 & 0xff ) as u8;
				self.frame[ o * 2 + 0 ] = lb;
				self.frame[ o * 2 + 1 ] = hb;
			}
		}
		self.framebuffer.write_frame( &self.frame );
	}

	fn get_key( &mut self ) -> Option< u32 > {
		// :TODO:
		None
	}

}

