use std::collections::HashMap;
use std::collections::VecDeque;

use std::path::{ 
	Path,
	PathBuf,
};

use notify::{
	DebouncedEvent,
	RecommendedWatcher,
	Watcher,
	RecursiveMode
};

use derivative::Derivative;
use path_calculate::*;

use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct FileCache {
	internal:		std::sync::Arc< std::sync::Mutex< FileCacheInternal > >,
	mode:			FileCacheMode,
	#[derivative(Debug="ignore")]
	watcher:		Option<RecommendedWatcher>,
}


#[derive(Debug)]
pub enum FileCacheMode {
	Poll,
	Watch,
}

impl FileCache {
	pub fn new() -> Self {
		Self {
			internal:		std::sync::Arc::new(std::sync::Mutex::new( FileCacheInternal::new() )),
			mode:			FileCacheMode::Poll,
			watcher:		None,
		}
	}

	pub fn set_mode( &mut self, mode: FileCacheMode ) {
		self.mode = mode;
	}

	pub async fn run_poll(&mut self) -> anyhow::Result<()> {
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

	pub async fn run_watch( &mut self ) -> anyhow::Result<()> {
	    // Create a channel to receive the events.
	    let (tx, rx) = channel();

	    // Automatically select the best implementation for your platform.
	    // You can also access each implementation directly e.g. INotifyWatcher.
	    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;

	    // Add a path to be watched. All files and directories at that path and
	    // below will be monitored for changes.
	    let base_path = {
	    	self.internal.lock().unwrap().base_path( ).to_owned()
	    };

	    watcher.watch( base_path.clone(), RecursiveMode::Recursive)?;

	    self.watcher = Some( watcher );
	    // This is a simple loop, but you may want to use more complex logic here,
	    // for example to handle I/O.
		let mut internal = self.internal.clone();

		std::thread::spawn(move || {
		    loop {
		        match rx.recv() {
		            Ok(event) => {
//		            	println!("{:?}", event);
		            	match event {
		            		DebouncedEvent::Write( full_path ) => {
								match full_path.related_to( &base_path ) {
									Ok( filename ) => {
										let filename = filename.to_string_lossy();
										internal.lock().unwrap().loading_queue_push_back( filename.to_string() );
									},
									Err( e ) => {
										dbg!(&e);
									},
								}
		            		},
		            		// :TODO: handle other cases
		            		_ => {},
		            	}
		            },
		            Err(e) => {
		            	match e {
		            		RecvError => {
		            			return;
		            		},
		            		e => {
				            	println!("watch error: {:?}", e);
		            		},

		            	}
		            },
		        }
		    }
		});
    	Ok(())
	}

	pub async fn run(&mut self) -> anyhow::Result<()> {
		match self.mode {
			FileCacheMode::Poll => self.run_poll().await?,
			FileCacheMode::Watch => self.run_watch().await?,
//			_ => todo!("Unsupported mode {:?}", self.mode ),
		}

		let mut internal = self.internal.clone();
	    let base_path = {
	    	internal.lock().unwrap().base_path( ).to_owned()
	    };
		std::thread::spawn(move || {
			loop {
			    let front = {
			    	internal.lock().unwrap().loading_queue_pop_front( )
			    };
			    if let Some( filename ) = front {
					let full_path = base_path.join( &filename);
					dbg!(&full_path);
					match FileCacheInternal::load_entry( &full_path ) {
						Ok( mut entry ) => {
							// :TODO: add mtime to entry
							internal.lock().unwrap().update_entry( &filename, entry );
						},
						Err( _ ) => {
							// :TODO: error handling
						},
					}
			    }
				std::thread::sleep( std::time::Duration::from_millis( 100 ) );
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
	loading_queue:	VecDeque< String >,
}

impl FileCacheInternal {
	pub fn new() -> Self {
		Self {
			base_path: 		Path::new(".").to_path_buf(),
			cache_misses:	0,
			cache_hits:		0,
			cache:			HashMap::new(),
			loading_queue:	VecDeque::new(),
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
			let block_on_initial_load = false;			// :TODO: expose via setter
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
				self.loading_queue_push_back( filename.to_string() );
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

	pub fn loading_queue_pop_front( &mut self ) -> Option< String > {
		self.loading_queue.pop_front()
	}

	pub fn loading_queue_push_back( &mut self, entry: String ) {
		self.loading_queue.push_back( entry );
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
