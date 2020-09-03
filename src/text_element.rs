use crate::element::{Element, ElementConfig};
use crate::pixel::Pixel;
use crate::context::Context;
use async_trait::async_trait;

use std::fs::File;
use std::io::{BufReader, Read};
use rusttype::{point, Font, Scale};
use regex::Regex;

#[derive(Debug)]
pub struct TextElement {
	name: String,
	x: u32,
	y: u32,
	width: u32,
	height: u32,
	color: u32,
	text: String,
	fontfile: String,
	size: u32,
	font: Option< Font<'static> >,
	display_text: String,
}

impl TextElement {
	fn fill_box( buffer: &mut Vec<u32>, width: usize, height: usize, x: u32, y: u32, w: u32, h: u32, color: u32 ) {
		for iy in 0..h {
			let py = iy + y;
			if py >= height as u32 { continue; }
			for ix in 0..w {
				let px = ix + x;
				if px >= width as u32 { continue; }

				let o = ( py * width as u32 + px ) as usize;
				buffer[ o ] = color;
			}
		}
	}
}

#[async_trait]
impl Element for TextElement {
	fn configure( &mut self, config: &ElementConfig ) {
		self.x      = config.get_u32_or( "pos_x", 0 );
		self.y      = config.get_u32_or( "pos_y", 0 );
		self.width  = config.get_u32_or( "width", 0 );
		self.height = config.get_u32_or( "height", 0 );
		self.color  = config.get_u32_or( "color", 0xffff00ff );
		self.text	= config.get_string_or( "text", "" );
		self.fontfile	= config.get_string_or( "font", "" );
		self.size	= config.get_u32_or( "size", 20 );
		self.display_text	= config.get_string_or( "text", "" );

		// load font
		/*
let mut f = match File::open(input[ 0 ]) {
			Ok( f ) => f,
			Err( _ ) => return Err(OmError::Generic("io".to_string())),
		};

		let mut buffer = Vec::new();

	    // read the whole file
	    f.read_to_end(&mut buffer).unwrap();//_or_else( return Err(OmError::Generic( "Error reading font file".to_string() )));

		let collection = FontCollection::from_bytes(&buffer[..] as &[u8]).unwrap_or_else(|e| {
	        panic!("error constructing a FontCollection from bytes: {}", e);
    	});

		let font = collection
		        .into_font() // only succeeds if collection consists of one font
		        .unwrap_or_else(|e| {
		            panic!("error turning FontCollection into a Font: {}", e);
		        });
		       */

		let mut font_file = File::open( &self.fontfile ).unwrap_or_else(|e| {
			panic!("{}", e);
		});

		let mut buffer = Vec::new();
		font_file.read_to_end(&mut buffer).unwrap_or_else(|e| {
			panic!("{}", e);
		});

		let font = Font::try_from_vec( buffer ).unwrap(); //.expect( panic!("error constructing a Font from vec" ) );

		self.font = Some( font );
	}

	fn shutdown( &mut self ) {
		
	}
	
	async fn run( &mut self ) -> anyhow::Result<()> {
		Ok(())
	}


	fn update( &mut self, context: &mut Context ) {
		let re = Regex::new(r"^\$\{(.+)\}$").unwrap();
		if let Some( caps ) = re.captures( &self.text ) {
			let name = &caps[ 1 ];
			if let Some( value ) = context.get_string( &name ) {
				self.display_text = value.to_string();
//				context.set_string( "clock_string", "USED" );
			}
		}
	}

	fn render( &self, buffer: &mut Vec<u32>, width: usize, height: usize ) {
//		dbg!(&self);
		if let Some( font ) = &self.font {

			let scale = Scale::uniform( self.size as f32 );
			let start = point( self.x as f32, ( self.y + self.size ) as f32 );
			let glyphs: Vec<_> = font.layout( &self.display_text, scale, start).collect();
//			dbg!(&glyphs);

			for g in glyphs {
				if let Some( bb ) = &g.pixel_bounding_box() {
					g.draw(|x, y, v| {
						if v > 0.0 {
							let x = ( bb.min.x as u32 + x ) as u32;
							let y = ( bb.min.y as u32 + y ) as u32;

							let o = ( y * width as u32 + x ) as usize;
							if o < buffer.len() {
								let old_pixel = Pixel::from_u32( buffer[ o ] );
								let new_pixel = Pixel::from_u32( self.color );
								let pixel = Pixel::blend_with_alpha_and_opacity( new_pixel, old_pixel, v );
								buffer[ o ] = pixel.to_u32();
							}
						}
					});
				/*
					let x = bb.min.x;
					let y = bb.min.y;
					let w = bb.max.x - x;
					let h = bb.max.y - y;
					TextElement::fill_box( buffer, width, height, x as u32, y as u32, w as u32, h as u32, self.color );
				*/
				}

			}
//			TextElement::fill_box( buffer, width, height, self.x, self.y, self.width, self.height, self.color );
			/*
			for y in 0..self.height {
				let py = y + self.y;
				if py >= height as u32 { continue; }
				for x in 0..self.width {
					let px = x + self.x;
					if px >= width as u32 { continue; }

	//				dbg!(&px, &py);

					let o = ( py * width as u32 + px ) as usize;
					buffer[ o ] = self.color;
				}
			}
			*/

		}

	}
	fn name( &self ) -> &str {
		&self.name
	}
	fn set_name(&mut self, name: &str ) {
		self.name = name.to_string();
	}

	fn element_type( &self ) -> &str {
		"text"
	}
}

pub struct TextElementFactory {

}

impl TextElementFactory {
	pub fn create() -> TextElement {
		TextElement {
			name: "".to_string(),
			x: 0,
			y: 0,
			width: 0,
			height: 0,
			color: 0xff00ffff,
			text: "".to_string(),
			fontfile: "".to_string(),
			size: 20,
			font: None,
			display_text: "".to_string(),
		}
	}
}

