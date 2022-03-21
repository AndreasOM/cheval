
use image::DynamicImage;
use image::GenericImageView;
use crate::pixel::Pixel;

use glob::glob;

#[derive(Debug)]
pub struct ImageSequence {
	filename: String,
	images: Vec< DynamicImage >,
}

impl ImageSequence {
	pub fn new() -> Self {
		Self {
			filename:	String::new(),
			images:		Vec::new(),
		}
	}

	pub fn len(&self) -> usize {
		self.images.len()
	}

	pub fn set_filename(&mut self, filename: &str ) {
		self.filename = filename.to_string();
	}
	pub fn load( &mut self ) -> anyhow::Result<()> {
		if self.filename != "" {
			self.images = Vec::new();

			let fileglob = self.filename.clone();

			for entry in glob( &fileglob ).expect("Failed to read glob pattern") {
			    match entry {
			        Ok(path) => {
			        	dbg!(&path);
			        	self.add_image( &path.to_string_lossy() );
			        },
			        Err(e) => println!("{:?}", e),
			    }
			}
		}
		Ok(())
	}

	pub fn get( &self, index: usize ) -> Option< &DynamicImage > {
		self.images.get( index )
	}

	fn add_image( &mut self, filename: &str ) -> bool {
		println!( "Trying to load image {:?}", &filename );
		match image::open(&filename) {
		    Ok( img ) => {
//		    	self.width = img.dimensions().0;
//			    self.height = img.dimensions().1;
			    self.images.push( img );
			    true
			},
			Err( e ) => {
				println!( "Couldn't load image {} {:?}", &filename, e );
//				self.images = Vec::new();
				false
			}
		}
	}

}
