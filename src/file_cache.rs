use std::collections::HashMap;

use std::path::{ 
	Path,
	PathBuf,
};

#[derive(Debug)]
pub struct FileCache {
	internal:		std::sync::Arc< std::sync::Mutex< FileCacheInternal > >,
}

impl FileCache {
	pub fn new() -> Self {
		Self {
			internal:		std::sync::Arc::new(std::sync::Mutex::new( FileCacheInternal::new() )),
		}
	}

	pub async fn run(&mut self) -> anyhow::Result<()> {

		let mut internal = self.internal.clone();
		std::thread::spawn(move || {
			loop {
				let cache: Vec< ( String, Option< std::time::SystemTime > ) > = {
					let file_cache = internal.lock().unwrap();
					file_cache.cache().iter().map(|e| { ( e.0.clone(), e.1.modification_time().to_owned() ) }).collect()
				};
				let base_path = {
					let file_cache = internal.lock().unwrap();
					file_cache.base_path().to_owned()
				};

				for e in cache {
					let full_path = base_path.join( &e.0 );

					let old_modification_time = &e.1;
					let new_modification_time = match std::fs::metadata( &full_path ) {
						Ok( metadata ) => {
							if let Ok(time) = metadata.modified() {
								Some( time )
							} else {
								None
							}
						},
						Err( e ) => {
							dbg!(&e);
							// :TODO: decide what todo when a file was deleted
							None
						},
					};

					let reload_file = match ( old_modification_time, new_modification_time ) {
						( Some( o ), Some( n ) ) => n > *o,
						( Some( o ), None ) => false,	// no new time, file probably doesn't exist
						( None, Some( n ) ) => true,
						( None, None ) => false,
					};

					if reload_file {
//						println!("FC {} is outdated", &e.0 );
						match FileCacheInternal::load_entry( &full_path ) {
							Ok( mut entry ) => {
								entry.set_modification_time( new_modification_time );
//								println!("FC loaded entry {:?}", &entry );
								internal.lock().unwrap().update_entry( &e.0, entry );
							},
							Err( _ ) => {
								// :TODO: error handling
							},
						}
					}
				}
				std::thread::sleep( std::time::Duration::from_millis( 1000 ) );
			}
		});

		Ok(())
	}

	pub fn update( &mut self ) {
		self.internal.lock().unwrap().update( );
	}

	pub fn set_base_path(&mut self, base_path: &PathBuf ) {
		self.internal.lock().unwrap().set_base_path( base_path );
	}

	// :TODO: change String to &str
	pub fn load_string( &mut self, filename: &str ) -> anyhow::Result<(u32,String)> {
		self.internal.lock().unwrap().load_string( filename )
	}

	pub fn cache_misses( &self ) -> u32 {
		self.internal.lock().unwrap().cache_misses( )
	}

	pub fn cache_hits( &self ) -> u32 {
		self.internal.lock().unwrap().cache_hits( )
	}
}

#[derive(Debug)]
struct FileCacheEntry {
	content:			String,
	modification_time:	Option< std::time::SystemTime >,
}

impl Default for FileCacheEntry {
	fn default() -> Self {
		Self {
			content:			String::new(),
			modification_time:	None,
		}
	}
}

impl FileCacheEntry {
	pub fn content( &self ) -> &str {
		&self.content
	}

	pub fn set_content( &mut self, content: String ) {
		self.content = content;
	}

	pub fn modification_time( &self ) -> &Option< std::time::SystemTime > {
		&self.modification_time
	}

	pub fn set_modification_time( &mut self, modification_time: Option< std::time::SystemTime > ) {
		self.modification_time = modification_time;
	}
}

#[derive(Debug)]
struct FileCacheInternal {
	base_path:		PathBuf,
	cache_misses:	u32,
	cache_hits:		u32,
	cache:			HashMap< String, FileCacheEntry >,
}

impl FileCacheInternal {
	pub fn new() -> Self {
		Self {
			base_path: 		Path::new(".").to_path_buf(),
			cache_misses:	0,
			cache_hits:		0,
			cache:			HashMap::new(),
		}
	}

	pub fn update( &mut self ) {
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

	pub fn base_path(&self) -> &Path {
		&self.base_path
	}

	pub fn cache( &self ) -> &HashMap< String, FileCacheEntry > {
		&self.cache
	}

	pub fn update_entry( &mut self, filename: &str, entry: FileCacheEntry ) -> anyhow::Result<()> {
		self.cache.insert( filename.to_string(), entry );
		Ok(())
	}

	pub fn load_entry( path: &Path ) -> anyhow::Result< FileCacheEntry > {
		match std::fs::read_to_string( &path )
		{
			Ok( s ) => {
				let mut e = FileCacheEntry::default();
				e.set_content( s.clone() );
				Ok( e )
			},
			Err( e ) => {
				anyhow::bail!("Error opening file {:?} -> {:?}", &path, &e )
			},
		}
	}

	// :TODO: change String to &str
	pub fn load_string( &mut self, filename: &str ) -> anyhow::Result<(u32,String)> {
		// :TODO: caching

		if let Some( cached ) = &self.cache.get( filename ) {
//			anyhow::bail!(":TODO: cache hit")
			self.cache_hits += 1;
			Ok((0,cached.content().to_string()))
		} else {
//			dbg!(&self.base_path,&filename);
			self.cache_misses += 1;

			let full_filename = &self.base_path.join( Path::new( &filename ) ) ;
			dbg!(&full_filename);
			let block_on_initial_load = true;			// :TODO: expose via setter
			if block_on_initial_load {
				match FileCacheInternal::load_entry( &full_filename ) {
					Ok( mut entry ) => {
						let s = entry.content().to_string();
						self.update_entry( &filename, entry );
						Ok((0,s))
					},
					Err( e ) => {
						// :TODO: error handling
						anyhow::bail!("TODO {:?}", e )
					},
				}		
			} else {
				self.update_entry( filename, FileCacheEntry::default() );
				Ok((0,String::new()))
			}
/*
			match std::fs::read_to_string( &full_filename )
			{
				Ok( s ) => {
					self.cache_misses += 1;
					let mut e = FileCacheEntry::default();
					e.set_content( s.clone() );
					self.cache.insert( filename.to_string(), e );
					Ok((0,s))
				},
				Err( e ) => {
					anyhow::bail!("Error opening file {:?} -> {:?}", &full_filename, &e )
				},
			}
*/			
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
