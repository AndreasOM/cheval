use cheval::cheval::Cheval;
use crate::window::Window;

pub struct WindowPng {
	width: usize,
	height: usize,
	downscale: usize, 
	buffer: Vec<u32>,
	frame: Vec<u32>,
	filename: Option<String>,
}

impl WindowPng {
	pub fn new() -> Self {
		let w = 1920;
		let h = 1080;
		let ds = 2;
		let fw = w/ds;
		let fh = h/ds;
		Self {
			width: w,
			height: h,
			downscale: ds,
			buffer: vec![0u32; w * h],
			frame: vec![0u32; fw * fh],
			filename: Some( "window.png".to_string() ), //None,
		}
	}	
}
impl Window for WindowPng {

	fn done( &self ) -> bool {
		false
	}
	fn render_frame( &mut self, func: &mut dyn FnMut( &mut Vec<u32>, usize, usize, &Cheval ), cheval: &Cheval  ) {
		func( &mut self.buffer, self.width, self.height, cheval );
	}
	fn next_frame( &mut self ) {
		// :TODO: handle multisampling for downscaling
		// :TODO: actually use downscaling factor for multisampling
		let ds = self.downscale;
		let fw = self.width / ds;
		let fh = self.height / ds;

		for y in 0..fh {
			for x in 0..fw {
				let so = ( y * ds ) * self.width + ( x * ds );
				let mut argb = vec![0u32;4];
				let pixel = self.buffer[ so ];
				argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
				argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
				argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

				let pixel = self.buffer[ so + 1 ];
				argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
				argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
				argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

				let pixel = self.buffer[ so + self.width ];
				argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
				argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
				argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

				let pixel = self.buffer[ so + self.width + 1 ];
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
		if let Some( filename ) = &self.filename { 
			// :TODO: write to png
			let mut imgbuf = image::ImageBuffer::new( fw as u32, fh as u32 );

			for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
				let fo = y * fw as u32 + x;
				let mut rgba = [0u8;4];
				let pix = self.frame[ fo as usize ];
				rgba[ 0 ] += ( ( pix >> 16 ) & 0xff ) as u8;
				rgba[ 1 ] += ( ( pix >>  8 ) & 0xff ) as u8;
				rgba[ 2 ] += ( ( pix >>  0 ) & 0xff ) as u8;
				rgba[ 3 ] += ( ( pix >> 24 ) & 0xff ) as u8;
				rgba[ 3 ] = 0xff;

				*pixel = image::Rgba(rgba);
			};

			match imgbuf.save(filename) {
				Err(e) => {
					println!("Error saving PNG: {:?}", e );
				},
				_ => {},
			}
		}
	}
}

