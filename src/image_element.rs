use async_trait::async_trait;
use image::GenericImageView;

use crate::bakedexpression::BakedExpression;
use crate::context::Context;
use crate::element::{Element, ElementConfig};
use crate::image_sequence::ImageSequence;
use crate::pixel::Pixel;
use crate::render_buffer::RenderBuffer;
use crate::render_context::RenderContext;

pub struct ImageElement {
	name:           String,
	x:              BakedExpression,
	y:              BakedExpression,
	width:          u32,
	height:         u32,
	color:          u32,
	filename:       String,
	fps:            BakedExpression,
	current_image:  f64,
	image_sequence: ImageSequence,
}

impl std::fmt::Debug for ImageElement {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		writeln!(f, "ImageElement: :TODO:")
	}
}

impl ImageElement {}

#[async_trait]
impl Element for ImageElement {
	fn configure(&mut self, config: &ElementConfig) {
		self.x = config.get_bakedexpression_u32("pos_x", 0);
		self.y = config.get_bakedexpression_u32("pos_y", 0);
		self.width = config.get_u32_or("width", 800);
		self.height = config.get_u32_or("height", 200);
		self.color = config.get_u32_or("color", 0xff00ffff);
		self.filename = config.get_path_or("filename", "");
		self.image_sequence.set_filename(&self.filename);
		self.fps = config.get_bakedexpression_f32("fps", 0.0);
	}

	fn shutdown(&mut self) {}

	async fn run(&mut self) -> anyhow::Result<()> {
		Ok(())
	}

	fn update(&mut self, context: &mut Context) {
		self.image_sequence.load(context.file_cache()); //?;
		self.x.bake_u32_or(context, 0);
		self.y.bake_u32_or(context, 0);
		self.fps.bake_f32_or(context, 0.0);
		//dbg!(&self.fps);
		let fps = self.fps.as_f32() as f64;
		if fps > 0.0 {
			let time_step = context.time_step();
			self.current_image += fps * time_step;
			self.current_image = self.current_image % (self.image_sequence.len() as f64);
			//			dbg!(&self.current_image);
			/*
			if self.current_image >= self.images.len() as f64 {
				self.current_image -= self.images.len() as f64;
			};
			*/
		}
	}

	fn render(&self, render_buffer: &mut RenderBuffer, _render_context: &mut RenderContext) {
		//		dbg!(&self);
		match &self.image_sequence.get(self.current_image.trunc() as usize) {
			None => {
				render_buffer.for_pixel_in_block(
					self.x.as_u32(),
					self.y.as_u32(),
					self.width,
					self.height,
					|_, _, _, _, p: &mut u32| {
						*p = self.color;
					},
				);
			},
			Some(img) => {
				let width = img.dimensions().0;
				let height = img.dimensions().1;

				render_buffer.for_pixel_in_block(
					self.x.as_u32(),
					self.y.as_u32(),
					width,
					height,
					|_sx, _sy, x, y, p: &mut u32| {
						let old_pixel = *p;

						let pixel = img.get_pixel(x, y);

						let pixel: u32 = (((pixel[3] & 0xff) as u32) << 24)
							| (((pixel[0] & 0xff) as u32) << 16)
							| (((pixel[1] & 0xff) as u32) << 8)
							| (((pixel[2] & 0xff) as u32) << 0);

						let pixel = Pixel::from_u32(pixel);
						let old_pixel = Pixel::from_u32(old_pixel);
						let pixel = Pixel::blend_with_alpha(&pixel, &old_pixel);

						*p = pixel.to_u32();
					},
				);
				/*
								for y in 0..self.height {
									let py = y + self.y;
									if py >= render_buffer.height as u32 { continue; }
									for x in 0..self.width {
										let px = x + self.x;
										if px >= render_buffer.width as u32 { continue; }

										let o = ( py * render_buffer.width as u32 + px ) as usize;

										let old_pixel = render_buffer.buffer[ o ];

										let pixel = img.get_pixel(x, y);

										let pixel: u32 =
											( ( ( pixel[ 3 ] & 0xff ) as u32 ) << 24 )
											| ( ( ( pixel[ 0 ] & 0xff ) as u32 ) << 16 )
											| ( ( ( pixel[ 1 ] & 0xff ) as u32 ) <<  8 )
											| ( ( ( pixel[ 2 ] & 0xff ) as u32 ) <<  0 );

										let pixel = Pixel::from_u32( pixel );
										let old_pixel = Pixel::from_u32( old_pixel );
										let pixel = Pixel::blend_with_alpha( pixel, old_pixel );
				//						let pixel = ImageElement::mix_argb_with_alpha( pixel, old_pixel, 1.0 );
										render_buffer.buffer[ o ] = pixel.to_u32();
									}
								}
				*/
			},
		}
	}
	fn name(&self) -> &str {
		&self.name
	}
	fn set_name(&mut self, name: &str) {
		self.name = name.to_string();
	}

	fn element_type(&self) -> &str {
		"image"
	}
}

pub struct ImageElementFactory {}

impl ImageElementFactory {
	pub fn create() -> ImageElement {
		ImageElement {
			name:           "".to_string(),
			x:              BakedExpression::from_u32(0),
			y:              BakedExpression::from_u32(0),
			width:          0,
			height:         0,
			color:          0xff00ffff,
			filename:       "".to_string(),
			fps:            BakedExpression::from_f32(0.0),
			current_image:  0.0,
			image_sequence: ImageSequence::new(),
		}
	}
}
