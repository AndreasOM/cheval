use crate::render_buffer::RenderBuffer;
use rusttype::{point, Font, Scale};
use crate::pixel::Pixel;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

#[derive(Debug)]
pub struct RenderContext{
	fonts: HashMap<String, Option< Font<'static> >>,
	// :HACK:
	current_font: Option< String >,
}

impl RenderContext{
	pub fn new(
	) -> Self {
		Self {
			fonts: HashMap::new(),
			current_font: None,
		}
	}

	pub fn use_font(
		&mut self,
		fontfile: &str,
	) -> anyhow::Result<()> {
		// :TODO: actually cache multiple fonts

		if self.fonts.contains_key( fontfile ) {
			self.current_font = Some( fontfile.to_string() );
		} else {
			dbg!(&fontfile);
			let mut font_file = File::open( &fontfile ).unwrap_or_else(|e| {
				panic!("{}", e);
			});

			let mut buffer = Vec::new();
			font_file.read_to_end(&mut buffer).unwrap_or_else(|e| {
				panic!("{}", e);
			});

			// :TODO: handle error to avoid constant retries
			let font = Font::try_from_vec( buffer ).unwrap(); //.expect( panic!("error constructing a Font from vec" ) );

			self.fonts.insert( fontfile.to_string(), Some( font ) );
		}


		Ok(())
	}

	pub fn draw_text(
		&self,
		render_buffer: &mut RenderBuffer,
		text: &str,
		x: u32,
		y: u32,
		width: u32,
		height: u32,
		size: u32,
		color: u32,
	) -> anyhow::Result<()> {
		if let Some( fontfile ) = &self.current_font {
			if let Some( font ) = &self.fonts.get( fontfile ) {
				if let Some( font ) = &font {

					let scale = Scale::uniform( size as f32 );
					let start = point( x as f32, ( y + size ) as f32 );
					let glyphs: Vec<_> = font.layout( &text, scale, start).collect();
		//			dbg!(&glyphs);

					for g in glyphs {
						if let Some( bb ) = &g.pixel_bounding_box() {
							g.draw(|x, y, v| {
								if v > 0.0 {
									let x = ( bb.min.x as u32 + x ) as u32;
									let y = ( bb.min.y as u32 + y ) as u32;

									let o = ( y * render_buffer.width as u32 + x ) as usize;
									if o < render_buffer.buffer.len() {
										let old_pixel = Pixel::from_u32( render_buffer.buffer[ o ] );
										let new_pixel = Pixel::from_u32( color );
										let pixel = Pixel::blend_with_alpha_and_opacity( new_pixel, old_pixel, v );
										render_buffer.buffer[ o ] = pixel.to_u32();
									}
								}
							});
						}
					}
				}
			}
		}

		Ok(())
	}
}
