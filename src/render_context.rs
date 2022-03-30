use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use rusttype::{point, Font, Scale};
use tracing::*;

use crate::axisalignedrectangle::AxisAlignedRectangle;
use crate::pixel::Pixel;
use crate::render_buffer::RenderBuffer;

#[derive(Debug)]
pub struct RenderContext {
	fonts:        HashMap<String, Option<Font<'static>>>,
	current_font: Option<String>,
}

impl RenderContext {
	pub fn new() -> Self {
		Self {
			fonts:        HashMap::new(),
			current_font: None,
		}
	}

	pub fn use_font(&mut self, fontfile: &str) -> anyhow::Result<()> {
		if self.fonts.contains_key(fontfile) {
			self.current_font = Some(fontfile.to_string());
		} else {
			debug!("Using font {}", &fontfile);
			let mut font_file = File::open(&fontfile).unwrap_or_else(|e| {
				panic!("{}", e);
			});

			let mut buffer = Vec::new();
			font_file.read_to_end(&mut buffer).unwrap_or_else(|e| {
				panic!("{}", e);
			});

			if let Some(font) = Font::try_from_vec(buffer) {
				self.fonts.insert(fontfile.to_string(), Some(font));
			} else {
				self.fonts.insert(fontfile.to_string(), None);
			};
		}

		Ok(())
	}

	pub fn draw_text(
		&self,
		render_buffer: &mut RenderBuffer,
		text: &str,
		pos_x: u32,
		pos_y: u32,
		_width: u32,
		_height: u32,
		bounding_box: &AxisAlignedRectangle,
		size: u32,
		color: u32,
	) -> anyhow::Result<()> {
		/*
		// :TODO: enable via command line
		self.draw_frame( render_buffer, pos_x, pos_y, width, height, 0xff44ee44 );
		self.draw_frame( render_buffer, bounding_box.x.as_u32(), bounding_box.y.as_u32(), bounding_box.width.as_u32(), bounding_box.height.as_u32(), 0xffff4444 );
		*/
		if let Some(fontfile) = &self.current_font {
			if let Some(font) = &self.fonts.get(fontfile) {
				if let Some(font) = &font {
					let scale = Scale::uniform(size as f32);

					let text_lines: Vec<&str> = text.split('\n').collect();

					let mut line = 0;
					for text in text_lines {
						let start = point(pos_x as f32, (pos_y + (line + 1) * size) as f32);
						let glyphs: Vec<_> = font.layout(&text, scale, start).collect();
						//			dbg!(&glyphs);

						let start_x = bounding_box.x.as_u32();
						let start_y = bounding_box.y.as_u32();
						let end_x = bounding_box.x.as_u32() + bounding_box.width.as_u32(); // pos_x + width;
						let end_y = bounding_box.y.as_u32() + bounding_box.height.as_u32(); // pos_y + height;

						let mut visible_glyphs = Vec::new();

						for g in glyphs {
							let bb = g.pixel_bounding_box();
							match bb {
								Some(r) => {
									if r.max.x >= bounding_box.x.as_u32() as i32
										&& r.min.x
											< (bounding_box.x.as_u32()
												+ bounding_box.width.as_u32()) as i32
									{
										//									self.draw_frame( render_buffer, r.min.x as u32, r.min.y as u32, ( r.max.x - r.min.x ) as u32, ( r.max.y - r.min.y ) as u32, 0xffaaaaee );
										visible_glyphs.push(g);
									}
								},
								None => {},
							}
						}

						for g in visible_glyphs {
							if let Some(bb) = &g.pixel_bounding_box() {
								/* :TODO: use nested loops instead of closure
									// pseudo code from `rusttype` crate
									let bb = glyph.pixel_bounding_box();
									for y in 0..bb.height() {
										for x in 0..bb.width() {
											o(x, y, calc_coverage(&glyph, x, y));
										}
									}
								*/
								let debug_overflow = false; //true;

								g.draw(|x, y, v| {
									if v > 0.0 {
										let mut color = color;
										let x = (bb.min.x as u32 + x) as u32;

										if x >= render_buffer.width as u32 {
											return;
										}

										if x >= end_x || x < start_x {
											if debug_overflow {
												color = 0xff44ee44;
											} else {
												return;
											}
										}

										let y = (bb.min.y as u32 + y) as u32;
										if y >= end_y || y < start_y {
											if debug_overflow {
												color = 0xff44ee44;
											} else {
												return;
											}
										}

										let o = (y * render_buffer.width as u32 + x) as usize;
										if o < render_buffer.buffer.len() {
											let old_pixel =
												Pixel::from_u32(render_buffer.buffer[o]);
											let new_pixel = Pixel::from_u32(color);
											let pixel = Pixel::blend_with_alpha_and_opacity(
												&new_pixel, &old_pixel, v,
											);
											render_buffer.buffer[o] = pixel.to_u32();
										}
									}
								});
							}
						}
						line += 1;
					}
				}
			}
		}

		Ok(())
	}
	pub fn draw_frame(
		&self,
		render_buffer: &mut RenderBuffer,
		pos_x: u32,
		pos_y: u32,
		width: u32,
		height: u32,
		color: u32,
	) -> anyhow::Result<()> {
		render_buffer.for_pixel_in_block(
			pos_x,
			pos_y,
			width,
			height,
			|x, y, _bx, _by, p: &mut u32| {
				if x == pos_x || x == pos_x + width - 1 || y == pos_y || y == pos_y + height - 1 {
					*p = color;
				}
			},
		);
		Ok(())
	}
}
