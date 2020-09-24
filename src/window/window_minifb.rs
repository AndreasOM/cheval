use minifb;

use cheval::cheval::Cheval;
use crate::window::Window;
use cheval::render_buffer::RenderBuffer;

pub struct WindowMinifb {
	render_buffer: RenderBuffer,
	downscale: usize, 
	frame: Vec<u32>,
	window: minifb::Window,
}

impl WindowMinifb {
	pub fn new() -> Self {
		let w = 1920;
		let h = 1080;
		let ds = 2;
		let fw = w/ds;
		let fh = h/ds;
		let render_buffer = RenderBuffer::new( w, h );
		let mut s = Self {
			render_buffer,
			downscale: ds,
			frame: vec![0u32; fw * fh],
			window: minifb::Window::new(
				"Test",
				fw,
				fh,
				minifb::WindowOptions::default()
			).unwrap_or_else(|e| {
        		panic!("{}", e);
    		}),
		};
		s.window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
		s
	}	
}
impl Window for WindowMinifb {

	fn done( &self ) -> bool {
		!( self.window.is_open() && !self.window.is_key_down(minifb::Key::Escape) )
	}
	fn render_frame( &mut self, func: &mut dyn FnMut( &mut RenderBuffer, &mut Cheval ), cheval: &mut Cheval  ) {
		func( &mut self.render_buffer, cheval );
	}
	fn next_frame( &mut self ) {
		// :TODO: handle multisampling for downscaling
		// :TODO: actually use downscaling factor for multisampling
		let ds = self.downscale;
		let fw = self.render_buffer.width / ds;
		let fh = self.render_buffer.height / ds;

		let mut argb = vec![0u32;4];
		for y in 0..fh {
			for x in 0..fw {
				argb[ 0 ] = 0;
				argb[ 1 ] = 0;
				argb[ 2 ] = 0;
				argb[ 3 ] = 0;

				let so = ( y * ds ) * self.render_buffer.width + ( x * ds );
				let pixel = self.render_buffer.buffer[ so ];
				argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
				argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
				argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

				let pixel = self.render_buffer.buffer[ so + 1 ];
				argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
				argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
				argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

				let pixel = self.render_buffer.buffer[ so + self.render_buffer.width ];
				argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
				argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
				argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

				let pixel = self.render_buffer.buffer[ so + self.render_buffer.width + 1 ];
				argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
				argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
				argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

				argb[ 1 ] /= 4;
				argb[ 2 ] /= 4;
				argb[ 3 ] /= 4;

				let pixel = 
					( ( argb[ 1 ] & 0xff ) << 16 )
					| ( ( argb[ 2 ] & 0xff ) <<  8 )
					| ( ( argb[ 3 ] & 0xff ) <<  0 );

//				let pixel = pixel + self.buffer[ so + 1 ];
//				let pixel = pixel / 2;
				let fo = y * fw + x;
//				if y >= 270 { dbg!(&x, &fo); }
				self.frame[ fo ] = pixel;
			}
		}
        self.window
            .update_with_buffer(&self.frame, fw, fh )
            .unwrap();
	}
}

