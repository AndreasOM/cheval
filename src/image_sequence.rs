use std::io::Cursor;

use image::DynamicImage;
use image::io::Reader;

use glob::glob;

use crate::file_cache::FileCache;

#[derive(Debug)]
struct ImageSequenceEntry {
	filename:	String,
	version: 	u32,
	image:		DynamicImage,
}

impl ImageSequenceEntry {
	pub fn filename( &self ) -> &str {
		&self.filename
	}

	pub fn image( &self ) -> &DynamicImage {
		&self.image
	}

	pub fn version( &self ) -> u32 {
		self.version
	}

	pub fn without_image( filename: &str, version: u32 ) -> Self {
		Self {
			filename: filename.to_string(),
			version,
			image: image::DynamicImage::ImageRgb8(image::RgbImage::new(0, 0)),
		}
	}

	pub fn from_data( filename: &str, version: u32, data: &Vec< u8 > ) -> anyhow::Result< Self > {
		let reader = Reader::new(Cursor::new(data))
							.with_guessed_format()
							.unwrap();

		match reader.decode() {
		    Ok( image ) => {
				Ok(
					Self {
						filename: filename.to_string(),
						version,
						image,
					}
				)
			},
			Err( e ) => {
				eprintln!("Couldn't load image {} {:?}", &filename, &e);
				anyhow::bail!( "Couldn't load image {} {:?}", &filename, e );
			}
		}
	}	
}

#[derive(Debug)]
pub struct ImageSequence {
	filename: String,
	entries: Vec< ImageSequenceEntry >,
}

impl ImageSequence {
	pub fn new() -> Self {
		Self {
			filename:	String::new(),
			entries:		Vec::new(),
		}
	}

	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn set_filename(&mut self, filename: &str ) {
		self.filename = filename.to_string();
	}
	pub fn load( &mut self, file_cache: &mut std::sync::Arc< std::sync::Mutex< FileCache > > ) -> anyhow::Result<()> {
		if self.entries.len() == 0 { // :HACK: to allow multiple calls for now
			if self.filename != "" {
				self.entries = Vec::new();

				let fileglob = self.filename.clone();

				for entry in glob( &fileglob ).expect("Failed to read glob pattern") {
					let mut fc = file_cache.lock().unwrap();
				    match entry {
				        Ok(path) => {
				        	self.add_image( &mut fc, &path.to_string_lossy() )?;
				        },
				        Err(e) => println!("{:?}", e),
				    }
				}
			}
		} else {
			// :TODO: maybe don't do this on the main thread
			for entry in self.entries.iter_mut() {
				let mut fc = file_cache.lock().unwrap();
				ImageSequence::update_entry( &mut fc, entry )?;
			}	
		}
		Ok(())
	}

	pub fn get( &self, index: usize ) -> Option< &DynamicImage > {
		self.entries.get( index ).map( |e| e.image() )
	}

	fn update_entry( file_cache: &mut FileCache, entry: &mut ImageSequenceEntry ) -> anyhow::Result<()> {
		let filename = entry.filename();
		let old_version = entry.version();
//		println!( "Trying to update image {:?}", &filename );

		let (version,data) = file_cache.load( &filename ).unwrap();

		if old_version != version {
//			eprintln!("Updating image from data version {}", version );
			let entry_new = ImageSequenceEntry::from_data( &filename, version, &data )?;

			*entry = entry_new;
		}

		Ok(())
	}

	fn add_image( &mut self, file_cache: &mut FileCache, filename: &str ) -> anyhow::Result<()> {
//		println!( "Trying to load image {:?}", &filename );
		let (version,data) = file_cache.load( filename ).unwrap();

		if data.len() > 0 { // only true if cache blocks on initial load, or somebody else already requested the same file earlier
			let entry = ImageSequenceEntry::from_data( &filename, version, &data )?;
			self.entries.push( entry );
		} else {
			let entry = ImageSequenceEntry::without_image( &filename, version );
			self.entries.push( entry );
		}

		Ok(())
	}

}
