use crate::element::{Element, ElementConfig};

use image::DynamicImage;
use image::GenericImageView;
use crate::pixel::Pixel;

pub struct ImageElement {
	name: String,
	x: u32,
	y: u32,
	width: u32,
	height: u32,
	color: u32,
	filename: String,
	image: Option< DynamicImage >,
}

impl std::fmt::Debug for ImageElement {
	fn fmt( &self, f: &mut std::fmt::Formatter ) -> std::fmt::Result {
		writeln!( f,"ImageElement: :TODO:" )
	}
}

impl ImageElement {
}

impl Element for ImageElement {
	fn configure( &mut self, config: &ElementConfig ) {
		self.x      = config.get_u32_or( "pos_x", 0 );
		self.y      = config.get_u32_or( "pos_y", 0 );
		self.width  = config.get_u32_or( "width", 800 );
		self.height = config.get_u32_or( "height", 200 );
		self.color  = config.get_u32_or( "color", 0xff00ffff );
		self.filename  = config.get_string_or( "filename", "" );
		if self.filename != "" {
			    let img = image::open(&self.filename).unwrap();
			    self.width = img.dimensions().0;
			    self.height = img.dimensions().1;
			    self.image = Some( img );
		}
	}
	fn update( &mut self ) {
	}

	fn render( &self, buffer: &mut Vec<u32>, width: usize, height: usize ) {
//		dbg!(&self);
		match &self.image {
			None => {
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
			},
			Some( img ) => {
				for y in 0..self.height {
					let py = y + self.y;
					if py >= height as u32 { continue; }
					for x in 0..self.width {
						let px = x + self.x;
						if px >= width as u32 { continue; }

						let o = ( py * width as u32 + px ) as usize;

						let old_pixel = buffer[ o ];

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
						buffer[ o ] = pixel.to_u32();
					}
				}
			},
		}

	}
	fn name( &self ) -> &str {
		&self.name
	}
	fn set_name(&mut self, name: &str ) {
		self.name = name.to_string();
	}

	fn element_type( &self ) -> &str {
		"image"
	}

}

pub struct ImageElementFactory {

}

impl ImageElementFactory {
	pub fn create() -> ImageElement {
		ImageElement {
			name: "".to_string(),
			x: 0,
			y: 0,
			width: 0,
			height: 0,
			color: 0xff00ffff,
			filename: "".to_string(),
			image: None,
		}
	}
}

