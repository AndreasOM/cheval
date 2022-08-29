use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use minifb;
use tracing::*;

use crate::cheval::Cheval;
use crate::render_buffer::RenderBuffer;
use crate::window::Window;
use crate::window::WindowLayout;
use crate::window::WindowLayoutWindowConfig;
use crate::window::WindowMode;

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
		debug!("add_char {}", uni_char);
		self.keys.borrow_mut().push(uni_char);
	}
}

#[allow(dead_code)]
struct WindowWithFrame {
	pub name:   String,
	pub window: minifb::Window,
	pub frame:  Vec<u32>,
}

impl std::fmt::Debug for WindowWithFrame {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("WindowWithFrame")
			.field("name", &self.name)
			.finish()
	}
}

impl WindowWithFrame {
	pub fn new(name: &str, w: usize, h: usize) -> Self {
		let window = minifb::Window::new(
			name,
			w,
			h,
			minifb::WindowOptions {
				//					resize: true,
				//					scale: minifb::Scale::X2,
				//					scale_mode: ScaleMode::AspectRatioStretch,
				..minifb::WindowOptions::default()
			},
		)
		.unwrap_or_else(|e| {
			panic!("{}", e);
		});
		Self {
			name: name.to_owned(),
			window,
			frame: vec![0u32; w * h],
		}
	}
}

#[allow(dead_code)]
pub struct WindowMinifb {
	window_title:    String,
	render_buffer:   RenderBuffer,
	downscale:       usize,
	frame:           Vec<u32>,
	//	window: minifb::Window,
	window_rgb:      Option<WindowWithFrame>,
	window_a:        Option<WindowWithFrame>,
	keybuffer:       Rc<RefCell<Vec<u32>>>,
	original_layout: Option<WindowLayout>,
}

impl WindowMinifb {
	pub fn new(window_title: &str, window_mode: &WindowMode) -> Self {
		let w = 1920;
		let h = 1080;
		let ds = 2;
		let fw = w / ds;
		let fh = h / ds;
		let render_buffer = RenderBuffer::new(w, h);
		let keybuffer = KeyVec::new(RefCell::new(Vec::new()));

		let mut s = Self {
			window_title: window_title.to_string(),
			render_buffer,
			downscale: ds,
			frame: vec![0u32; fw * fh],
			window_rgb: None,
			window_a: None,
			keybuffer: keybuffer.clone(),
			original_layout: None,
		};

		// :TODO: loop for all windows

		let (_need_r, _need_g, _need_b, need_a, need_rgb, _need_rgba) = match window_mode {
			WindowMode::RGB => (false, false, false, false, true, false),
			WindowMode::RGB_A => (false, false, false, true, true, false),
			WindowMode::A => (false, false, false, true, false, false),
			e => todo!("WindowMode {:?} not implemented", e),
		};

		let (mut x, mut y) = (100, 100);
		if need_rgb {
			let name = format!("{}RGB", &s.window_title);
			let mut w = WindowWithFrame::new(&name, fw, fh);
			w.window
				.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
			let input = Box::new(Input::new(&keybuffer));
			w.window.set_input_callback(input);
			w.window.set_position(x, y);
			x += 50;
			y += 50;
			s.window_rgb = Some(w);
		}

		if need_a {
			let name = format!("{}A", &s.window_title);
			let mut w = WindowWithFrame::new(&name, fw, fh);
			w.window
				.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
			let input = Box::new(Input::new(&keybuffer));
			w.window.set_input_callback(input);
			w.window.set_position(x, y);
			x += 50;
			y += 50;
			s.window_a = Some(w);
		}
		dbg!(x, y);
		s
	}

	fn add_char(&mut self, uni_char: u32) {
		self.keybuffer.borrow_mut().push(uni_char);
	}

	#[inline(never)] // needed for clean flamegraphs ... :sigh:
	pub fn render_frame_rgb_a(
		source: &RenderBuffer,
		width: usize,
		height: usize,
		dest_rgb: &mut Vec<u32>,
		dest_a: &mut Vec<u32>,
	) {
		let ds = 2; // :TODO: 2x downscale is the only supported mode for now, fix once needed.
		let ds = 1;
		let mut argb = vec![0u32; 4];
		let range = source.buffer.as_ptr_range();
		let s_start = range.start;
		let s_end = range.end;

		for y in 0..height {
			for x in 0..width {
				argb[0] = 0;
				argb[1] = 0;
				argb[2] = 0;
				argb[3] = 0;

				let so = (y * ds) * source.width + (x * ds);

				let pixel = source.buffer[so];
				//					let pixel = unsafe { &*s_start.add( so ) };
				argb[0] += ((pixel >> 24) & 0xff) as u32;
				argb[1] += ((pixel >> 16) & 0xff) as u32;
				argb[2] += ((pixel >> 8) & 0xff) as u32;
				argb[3] += ((pixel >> 0) & 0xff) as u32;
				/*
									let pixel = source.buffer[ so + 1 ];
				//					let pixel = unsafe { &*s_start.add( so+1 ) };
									argb[ 0 ] += ( ( pixel >> 24 ) & 0xff ) as u32;
									argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
									argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
									argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

									let pixel = source.buffer[ so + source.width ];
				//					let pixel = unsafe { &*s_start.add( so + source.width ) };
									argb[ 0 ] += ( ( pixel >> 24 ) & 0xff ) as u32;
									argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
									argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
									argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

									let pixel = source.buffer[ so + source.width + 1 ];
				//					let pixel = unsafe { &*s_start.add( so + source.width + 1 ) };
									argb[ 0 ] += ( ( pixel >> 24 ) & 0xff ) as u32;
									argb[ 1 ] += ( ( pixel >> 16 ) & 0xff ) as u32;
									argb[ 2 ] += ( ( pixel >>  8 ) & 0xff ) as u32;
									argb[ 3 ] += ( ( pixel >>  0 ) & 0xff ) as u32;

									argb[ 0 ] /= 4;
									argb[ 1 ] /= 4;
									argb[ 2 ] /= 4;
									argb[ 3 ] /= 4;
				*/
				let pixel =
					((argb[1] & 0xff) << 16) | ((argb[2] & 0xff) << 8) | ((argb[3] & 0xff) << 0);

				//				let pixel = pixel + self.buffer[ so + 1 ];
				//				let pixel = pixel / 2;
				let fo = y * width + x;
				//				if y >= 270 { dbg!(&x, &fo); }
				//					if have_rgb {
				dest_rgb[fo] = pixel;
				//					};
				//					if have_a {
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
					((argb[0] & 0xff) << 16) | ((argb[0] & 0xff) << 8) | ((argb[0] & 0xff) << 0);

				dest_a[fo] = pixel_a;
				//					};
			}
		}
	}
}

impl Window for WindowMinifb {
	fn done(&self) -> bool {
		if let Some(w) = &self.window_rgb {
			if w.window.is_open() && w.window.is_key_down(minifb::Key::Escape) {
				return true;
			}
		}
		if let Some(w) = &self.window_a {
			if w.window.is_open() && w.window.is_key_down(minifb::Key::Escape) {
				return true;
			}
		}
		false
	}

	fn render_frame(
		&mut self,
		func: &mut dyn FnMut(&mut RenderBuffer, &mut Cheval),
		cheval: &mut Cheval,
	) {
		func(&mut self.render_buffer, cheval);
	}
	fn next_frame(&mut self) {
		// :TODO: handle multisampling for downscaling
		// :TODO: actually use downscaling factor for multisampling
		let ds = self.downscale;
		let ds = 1;
		let fw = self.render_buffer.width / ds;
		let fh = self.render_buffer.height / ds;

		let mut argb = vec![0u32; 4];

		// :TODO: handle all windows

		let mut have_rgb = false;
		let mut frame_rgb = vec![0u32; 4];

		let mut have_a = false;
		let mut frame_a = vec![0u32; 4];

		if self.window_rgb.is_some() {
			have_rgb = true;
			//			frame_rgb = &mut window_rgb.frame;
			frame_rgb = vec![0u32; fw * fh];
		}

		if self.window_a.is_some() {
			have_a = true;
			//			frame_rgb = &mut window_rgb.frame;
			frame_a = vec![0u32; fw * fh];
		}

		//		let mut frame = &mut window.frame;

		if have_rgb && have_a {
			WindowMinifb::render_frame_rgb_a(
				&self.render_buffer,
				fw,
				fh,
				&mut frame_rgb,
				&mut frame_a,
			);
		} else {
			for y in 0..fh {
				for x in 0..fw {
					argb[0] = 0;
					argb[1] = 0;
					argb[2] = 0;
					argb[3] = 0;

					let so = (y * ds) * self.render_buffer.width + (x * ds);
					let pixel = self.render_buffer.buffer[so];
					argb[0] += ((pixel >> 24) & 0xff) as u32;
					argb[1] += ((pixel >> 16) & 0xff) as u32;
					argb[2] += ((pixel >> 8) & 0xff) as u32;
					argb[3] += ((pixel >> 0) & 0xff) as u32;
					/* no downscaling for now, let minifb handle it
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
					*/
					let pixel = ((argb[1] & 0xff) << 16)
						| ((argb[2] & 0xff) << 8)
						| ((argb[3] & 0xff) << 0);

					//				let pixel = pixel + self.buffer[ so + 1 ];
					//				let pixel = pixel / 2;
					let fo = y * fw + x;
					//				if y >= 270 { dbg!(&x, &fo); }
					if have_rgb {
						frame_rgb[fo] = pixel;
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
						let pixel_a = ((argb[0] & 0xff) << 16)
							| ((argb[0] & 0xff) << 8) | ((argb[0] & 0xff) << 0);

						frame_a[fo] = pixel_a;
					};
				}
			}
		}
		let mut chars = Vec::new();
		if let Some(window_rgb) = &mut self.window_rgb {
			// :HACK:
			for k in window_rgb.window.get_keys_pressed(minifb::KeyRepeat::No) {
				//debug!("Key Pressed: {:?}", k);
				match k {
					minifb::Key::Left => chars.push(63234),
					minifb::Key::Right => chars.push(63235),
					_ => {},
				}
			}
			window_rgb
				.window
				.update_with_buffer(&frame_rgb, fw, fh)
				.unwrap();
		}
		if let Some(window_a) = &mut self.window_a {
			window_a
				.window
				.update_with_buffer(&frame_a, fw, fh)
				.unwrap();
		}

		for c in chars {
			self.add_char(c);
		}
	}

	fn get_key(&mut self) -> Option<u32> {
		let mut keys = self.keybuffer.borrow_mut();

		if keys.is_empty() {
			None
		} else {
			Some(keys.remove(0))
		}
	}
	fn restore_positions(&mut self, filename: &str) {
		let mut c = WindowLayout::default();

		if let Ok(_) = c.load(&Path::new(&filename)) {
			if let Some(ref wc) = c.window_rgb {
				if let Some(w) = &mut self.window_rgb {
					w.window.set_position(wc.pos_x as isize, wc.pos_y as isize);
				}
			}
			if let Some(ref wc) = c.window_a {
				if let Some(w) = &mut self.window_a {
					w.window.set_position(wc.pos_x as isize, wc.pos_y as isize);
				}
			}

			self.original_layout = Some(c);
		}
	}

	fn store_positions(&mut self, filename: &str) {
		let mut layout = if let Some(owl) = self.original_layout.take() {
			owl
		} else {
			WindowLayout::default()
		};

		if let Some(w) = &self.window_rgb {
			let (x, y) = w.window.get_position();
			let mut wc = WindowLayoutWindowConfig::default();
			wc.pos_x = x as u32;
			wc.pos_y = y as u32;
			layout.window_rgb = Some(wc);
		}
		if let Some(w) = &self.window_a {
			let (x, y) = w.window.get_position();
			let mut wc = WindowLayoutWindowConfig::default();
			wc.pos_x = x as u32;
			wc.pos_y = y as u32;
			layout.window_a = Some(wc);
		}

		match layout.save(&Path::new(&filename)) {
			// :TODO: handle errors
			_ => {},
		}

		self.original_layout = Some(layout);
	}
}
