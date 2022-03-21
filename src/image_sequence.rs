use std::io::Cursor;

use image::DynamicImage;
use image::GenericImageView;
use image::io::Reader;
use crate::pixel::Pixel;

use glob::glob;

use crate::file_cache::FileCache;

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
	pub fn load( &mut self, file_cache: &mut std::sync::Arc< std::sync::Mutex< FileCache > > ) -> anyhow::Result<()> {
		if self.images.len() == 0 { // :HACK: to allow multiple calls for now
			if self.filename != "" {
				self.images = Vec::new();

				let fileglob = self.filename.clone();

				for entry in glob( &fileglob ).expect("Failed to read glob pattern") {
					let mut fc = file_cache.lock().unwrap();
				    match entry {
				        Ok(path) => {
				        	dbg!(&path);
				        	self.add_image( &mut fc, &path.to_string_lossy() );
				        },
				        Err(e) => println!("{:?}", e),
				    }
				}
			}
		}
		Ok(())
	}

	pub fn get( &self, index: usize ) -> Option< &DynamicImage > {
		self.images.get( index )
	}

	fn add_image( &mut self, file_cache: &mut FileCache, filename: &str ) -> bool {
		println!( "Trying to load image {:?}", &filename );
		let (version,data) = file_cache.load( filename ).unwrap();

		let mut reader = Reader::new(Cursor::new(data))
							.with_guessed_format()
							.unwrap();

		match reader.decode() {
		    Ok( img ) => {
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
