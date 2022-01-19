use minifb;

use cheval::cheval::Cheval;
use crate::window::Window;
use crate::window::WindowMode;
use cheval::render_buffer::RenderBuffer;

use std::cell::RefCell;
use std::rc::Rc;

type KeyVec = Rc<RefCell<Vec<u32>>>;

struct Input {
    keys: KeyVec,
}

impl Input {
    fn new(data: &KeyVec) -> Input {
        Input { keys: data.clone() }
    }
}

impl minifb::InputCallback for Input {
    fn add_char(&mut self, uni_char: u32) {
        self.keys.borrow_mut().push(uni_char);
    }
}

struct WindowWithFrame {
	pub name:	String,
	pub window: minifb::Window,
	pub frame:	Vec<u32>,
}

impl WindowWithFrame {
	pub fn new( name: &str, w: usize, h: usize ) -> Self {
		let window = minifb::Window::new(
				name,
				w,
				h,
				minifb::WindowOptions::default()
			).unwrap_or_else(|e| {
        		panic!("{}", e);
    		});
		Self {
			name:	name.to_owned(),
			window,
			frame:	vec![0u32; w * h],
		}
	}
}

pub struct WindowMinifb {
	render_buffer: RenderBuffer,
	downscale: usize, 
	frame: Vec<u32>,
//	window: minifb::Window,
	window_rgb:	Option< WindowWithFrame >,
	window_a:	Option< WindowWithFrame >,
	keybuffer: Rc<RefCell<Vec<u32>>>,
}

impl WindowMinifb {
	pub fn new( window_mode: &WindowMode ) -> Self {
		let w = 1920;
		let h = 1080;
		let ds = 2;
		let fw = w/ds;
		let fh = h/ds;
		let render_buffer = RenderBuffer::new( w, h );
		let keybuffer = KeyVec::new(RefCell::new(Vec::new()));
		let input = Box::new( Input::new( &keybuffer ) );

		let mut s = Self {
			render_buffer,
			downscale:	ds,
			frame:		vec![0u32; fw * fh],
			window_rgb:	None,
			window_a:	None,
    		keybuffer:	keybuffer,
		};

		// :TODO: loop for all windows

		let ( need_r, need_g, need_b, need_a, need_rgb, need_rgba ) = match window_mode {
				WindowMode::RGB		=> ( false,	false,	false,	false,	true,	false ),
				WindowMode::RGB_A	=> ( false,	false,	false,	true,	true,	false ),
				WindowMode::A		=> ( false,	false,	false,	true,	false,	false ),
				e					=> todo!("WindowMode {:?} not implemented", e),
		};

		if need_rgb {
			let name = "RGB";
			let mut w = WindowWithFrame::new( &name, fw, fh );
			w.window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
			w.window.set_input_callback(input);
			s.window_rgb = Some( w );
		}

		if need_a {
			let name = "A";
			let mut w = WindowWithFrame::new( &name, fw, fh );
			w.window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
// /			w.window.set_input_callback(input);
			s.window_a = Some( w );
		}
		s
	}
}

impl Window for WindowMinifb {

	fn done( &self ) -> bool {
		if let Some( w ) = &self.window_rgb {
			if w.window.is_open() && w.window.is_key_down( minifb::Key::Escape ) {
				return true;
			}
		}
		if let Some( w ) = &self.window_a {
			if w.window.is_open() && w.window.is_key_down( minifb::Key::Escape ) {
				return true;
			}
		}
		false
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

		// :TODO: handle all windows

		let mut have_rgb = false;
		let mut frame_rgb = vec![0u32;4];

		let mut have_a = false;
		let mut frame_a = vec![0u32;4];

		if let Some( window_rgb ) = &mut self.window_rgb {
			have_rgb = true;
//			frame_rgb = &mut window_rgb.frame;
			frame_rgb =	vec![0u32; fw * fh];
		}

		if let Some( window_a ) = &mut self.window_a {
			have_a = true;
//			frame_rgb = &mut window_rgb.frame;
			frame_a =	vec![0u32; fw * fh];
		}

//		let mut frame = &mut window.frame;

		for y in 0..fh {
			for x in 0..fw {
				argb[ 0 ] = 0;
				argb[ 1 ] = 0;
				argb[ 2 ] = 0;
				argb[ 3 ] = 0;

				let so = ( y * ds ) * self.render_buffer.width + ( x * ds );
				let pixel = self.render_buffer.buffer[ so ];
				argb[ 0 ] += ( ( pixel >> 24 ) & 0xff ) as u32;
				argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
				argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
				argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

				let pixel = self.render_buffer.buffer[ so + 1 ];
				argb[ 0 ] += ( ( pixel >> 24 ) & 0xff ) as u32;
				argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
				argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
				argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

				let pixel = self.render_buffer.buffer[ so + self.render_buffer.width ];
				argb[ 0 ] += ( ( pixel >> 24 ) & 0xff ) as u32;
				argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
				argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
				argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

				let pixel = self.render_buffer.buffer[ so + self.render_buffer.width + 1 ];
				argb[ 0 ] += ( ( pixel >> 24 ) & 0xff ) as u32;
				argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
				argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
				argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

				argb[ 0 ] /= 4;
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
				if have_rgb {
					frame_rgb[ fo ] = pixel;
				};
				if have_a {
					/* :TEST:
					let yyy = y.wrapping_rem( 256 ) as u32;
					let xxx = x.wrapping_rem( 256 ) as u32;
					let zzz = ( x + y).wrapping_rem( 256 ) as u32;
					let pixel_a = 
						( ( yyy & 0xff ) << 16 )
						| ( ( xxx & 0xff ) <<  8 )
						| ( ( zzz & 0xff ) <<  0 );
					*/
					let pixel_a = 
						( ( argb[ 0 ] & 0xff ) << 16 )
						| ( ( argb[ 0 ] & 0xff ) <<  8 )
						| ( ( argb[ 0 ] & 0xff ) <<  0 );

					frame_a[ fo ] = pixel_a;
				};
			}
		}
		if let Some( window_rgb ) = &mut self.window_rgb {
			window_rgb.window
				.update_with_buffer(&frame_rgb, fw, fh )
				.unwrap();
		}
		if let Some( window_a ) = &mut self.window_a {
			window_a.window
				.update_with_buffer(&frame_a, fw, fh )
				.unwrap();
		}
	}

	fn get_key( &mut self ) -> Option< u32 > {
		let mut keys = self.keybuffer.borrow_mut();

		if keys.is_empty() {
			None
		} else {
			Some( keys.remove( 0 ) )
		}
	}
}

