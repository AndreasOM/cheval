use crate::cheval::Cheval;
use crate::render_buffer::RenderBuffer;
use crate::window::Window;

pub struct WindowPng {
	render_buffer:     RenderBuffer,
	downscale:         usize,
	frame:             Vec<u32>,
	filename:          Option<String>,
	filename_template: Option<String>,
	frame_count:       usize,
}

impl WindowPng {
	pub fn new(scaling: f32) -> Self {
		let w = 1920;
		let h = 1080;
		let ds = if scaling == 0.5 {
			2
		} else if scaling == 1.0 {
			1
		} else {
			panic!("Unsupported scaling")
		};

		let fw = w / ds;
		let fh = h / ds;
		let render_buffer = RenderBuffer::new(w, h);

		Self {
			render_buffer,
			downscale: ds,
			frame: vec![0u32; fw * fh],
			filename: Some("window.png".to_string()), //None,
			//	filename_template: None,
			filename_template: Some("window_%04d.png".to_string()),
			frame_count: 0,
		}
	}
}
impl Window for WindowPng {
	fn done(&self) -> bool {
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
		let fw = self.render_buffer.width / ds;
		let fh = self.render_buffer.height / ds;

		for y in 0..fh {
			for x in 0..fw {
				let so = (y * ds) * self.render_buffer.width + (x * ds);
				let mut argb = vec![0u32; 4];
				let pixel = self.render_buffer.buffer[so];
				argb[0] += ((pixel >> 24) & 0xff) as u32;
				argb[1] += ((pixel >> 16) & 0xff) as u32;
				argb[2] += ((pixel >> 8) & 0xff) as u32;
				argb[3] += ((pixel >> 0) & 0xff) as u32;

				if ds == 2 {
					let pixel = self.render_buffer.buffer[so + 1];
					argb[0] += ((pixel >> 24) & 0xff) as u32;
					argb[1] += ((pixel >> 16) & 0xff) as u32;
					argb[2] += ((pixel >> 8) & 0xff) as u32;
					argb[3] += ((pixel >> 0) & 0xff) as u32;

					let pixel = self.render_buffer.buffer[so + self.render_buffer.width];
					argb[0] += ((pixel >> 24) & 0xff) as u32;
					argb[1] += ((pixel >> 16) & 0xff) as u32;
					argb[2] += ((pixel >> 8) & 0xff) as u32;
					argb[3] += ((pixel >> 0) & 0xff) as u32;

					let pixel = self.render_buffer.buffer[so + self.render_buffer.width + 1];
					argb[0] += ((pixel >> 24) & 0xff) as u32;
					argb[1] += ((pixel >> 16) & 0xff) as u32;
					argb[2] += ((pixel >> 8) & 0xff) as u32;
					argb[3] += ((pixel >> 0) & 0xff) as u32;

					argb[0] /= 4;
					argb[1] /= 4;
					argb[2] /= 4;
					argb[3] /= 4;
				}

				let pixel = ((argb[0] & 0xff) << 24)
					| ((argb[1] & 0xff) << 16)
					| ((argb[2] & 0xff) << 8)
					| ((argb[3] & 0xff) << 0);

				//				let pixel = pixel + self.buffer[ so + 1 ];
				//				let pixel = pixel / 2;
				let fo = y * fw + x;
				//				if y >= 270 { dbg!(&x, &fo); }
				self.frame[fo] = pixel;
			}
		}

		if let Some(filename_template) = &self.filename_template {
			let frame_4d = format!("{:0>4}", self.frame_count);
			dbg!(&frame_4d);
			let f = filename_template.replace(&"%04d", &frame_4d);
			dbg!(&f);
			self.filename = Some(f);
			//			let f = format!(&filename_template, frame);
		};

		if let Some(filename) = &self.filename {
			// :TODO: write to png
			let mut imgbuf = image::ImageBuffer::new(fw as u32, fh as u32);

			for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
				let fo = y * fw as u32 + x;
				let mut rgba = [0u8; 4];
				let pix = self.frame[fo as usize];
				rgba[0] += ((pix >> 16) & 0xff) as u8;
				rgba[1] += ((pix >> 8) & 0xff) as u8;
				rgba[2] += ((pix >> 0) & 0xff) as u8;
				rgba[3] += ((pix >> 24) & 0xff) as u8;
				//				rgba[ 3 ] = 0xff;

				*pixel = image::Rgba(rgba);
			}

			match imgbuf.save(filename) {
				Err(e) => {
					println!("Error saving PNG: {:?}", e);
				},
				_ => {},
			}
		}
		self.frame_count += 1;
	}

	fn get_key(&mut self) -> Option<u32> {
		None
	}
}
