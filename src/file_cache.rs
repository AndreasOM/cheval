use std::collections::HashMap;

use std::path::{ 
	Path,
	PathBuf,
};

#[derive(Debug)]
pub struct FileCache {
	base_path:		PathBuf,
	cache_misses:	u32,
	cache_hits:		u32,
	cache:			HashMap< String, String >,
}

impl FileCache {
	pub fn new() -> Self {
		Self {
			base_path: 		Path::new(".").to_path_buf(),
			cache_misses:	0,
			cache_hits:		0,
			cache:			HashMap::new(),
		}
	}

	pub fn set_base_path(&mut self, base_path: &PathBuf ) {
		let cwd = std::env::current_dir().unwrap();
		let full_path = Path::new( &base_path );
		let full_path = cwd.join( full_path );
		let full_path = match full_path.canonicalize() {
			Ok( c ) => c,
			Err( e ) =>panic!( "Error canonicalizing base path {:?} -> {:?}", &base_path, &e ),
		};

		self.base_path = full_path.to_path_buf();
	}

	// :TODO: change String to &str
	pub fn load_string( &mut self, filename: &str ) -> anyhow::Result<(u32,String)> {
		// :TODO: caching

		if let Some( cached ) = &self.cache.get( filename ) {
//			anyhow::bail!(":TODO: cache hit")
			self.cache_hits += 1;
			Ok((0,cached.to_string()))
		} else {
//			dbg!(&self.base_path,&filename);
			let full_filename = &self.base_path.join( Path::new( &filename ) ) ;
			dbg!(&full_filename);

			match std::fs::read_to_string( &full_filename )
			{
				Ok( s ) => {
					self.cache_misses += 1;
					self.cache.insert( filename.to_string(), s.clone() );
					Ok((0,s))
				},
				Err( e ) => {
					anyhow::bail!("Error opening file {:?} -> {:?}", &full_filename, &e )
				},
			}
		}
	}

	pub fn cache_misses( &self ) -> u32 {
		self.cache_misses
	}

	pub fn cache_hits( &self ) -> u32 {
		self.cache_hits
	}
}

mod test {

	use std::path::Path;

	use crate::file_cache::FileCache;

	#[test]
	pub fn file_cache_can_load_file() {
		let mut fc = FileCache::new();
		fc.set_base_path( &Path::new( "./test" ).to_path_buf() );
		let f = fc.load_string( "test_text_01.txt" );
//		assert!(f.is_ok());
		assert_eq!( "01", f.unwrap().1.to_string() );
		assert_eq!( 1, fc.cache_misses() );

		let f = fc.load_string( "test_text_02.txt" );
//		assert!(f.is_ok());
		assert_eq!( "02", f.unwrap().1.to_string() );
		assert_eq!( 2, fc.cache_misses() );

		let f = fc.load_string( "test_text_01.txt" );
//		assert!(f.is_ok());
		assert_eq!( "01", f.unwrap().1.to_string() );
		assert_eq!( 2, fc.cache_misses() );
		assert_eq!( 1, fc.cache_hits() );
	}
}
