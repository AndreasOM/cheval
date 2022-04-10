use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

use derivative::Derivative;
use futures::select;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::*;
//use glob::glob;

#[derive(Debug)]
enum WatchChange {
	Add(PathBuf),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct FileCache {
	internal: std::sync::Arc<std::sync::Mutex<FileCacheInternal>>,
	mode:     FileCacheMode,
	//	#[derivative(Debug="ignore")]
}

#[derive(Debug)]
pub enum FileCacheMode {
	Poll,
	Watch,
}

impl FileCache {
	pub fn canonicalize( path: &Path ) -> anyhow::Result< PathBuf > {
		let c = match path.canonicalize() {
			Ok(c) => c,
			Err(e) => anyhow::bail!(
				"Error canonicalizing filename {:?} -> {:?}",
				&path, &e
			),
		};
/*
		let c = c.components().map( |c|{
//			debug!("{:?}", &c);
			match c {
				std::path::Component::Prefix( p ) => {
					debug!("Prefix {:?}", &p);
					let p = match p.kind() {
						std::path::Prefix::VerbatimDisk( disk ) => {
							debug!("Disk {}", &disk);
//							p
							std::path::Prefix::Disk( disk )
						},
						o => o,
					};
					/*
					let p = match p.kind() {
						std::path::Prefix::VerbatimDisk( disk ) => {
							std::path::Prefix::Disk( disk )
						},
						p => p,
					};
					*/
					std::path::Component::Prefix( p )
					//p
				},
				c => c,
			}
		}).collect();
		*/
		let mut result = PathBuf::new();
		for c in c.components() {
			match c {
				std::path::Component::Prefix( p ) => {
					match p.kind() {
						std::path::Prefix::VerbatimDisk( disk ) => {
//							debug!("Disk {}", &disk);
							result.push( format!("{}:", disk as char) );
						},
						_ => result.push( c ),
					}
				},
				o => result.push( o ),
			};
		}

//		debug!("canonicalized -> {:?}", &result);
		Ok( result )
	}
	pub fn glob(pattern: &str) -> anyhow::Result<Vec<PathBuf>> {
		let mut paths = Vec::new();
		let path = std::path::Path::new(pattern);
		/*
		if path.exists() { // Note: This should not be needed if globbing worked correctly
			paths.push(path.to_path_buf());
		} else {
		*/
		{
			let glob = match glob::glob(pattern) {
				Ok(o) => o,
				Err(e) => {
					anyhow::bail!("glob error for {} -> {:?}", &pattern, e);
				},
			};
			for entry in glob {
				match entry {
					Ok(path) => {
						paths.push(path);
					},
					Err(e) => debug!("Error globbing {:?}", e),
				}
			}
		}
		Ok(paths)
	}

	pub fn new() -> Self {
		Self {
			internal: std::sync::Arc::new(std::sync::Mutex::new(FileCacheInternal::new())),
			mode:     FileCacheMode::Poll,
		}
	}

	pub fn set_mode(&mut self, mode: FileCacheMode) {
		self.mode = mode;
	}

	pub fn enable_block_on_initial_load(&mut self) {
		self.internal.lock().unwrap().enable_block_on_initial_load();
	}

	pub fn disable_block_on_initial_load(&mut self) {
		self.internal
			.lock()
			.unwrap()
			.disable_block_on_initial_load();
	}

	pub async fn run_poll(&mut self) -> anyhow::Result<()> {
		let internal = self.internal.clone();
		std::thread::spawn(move || {
			loop {
				//dbg!("run_pool::loop");
				let cache: Vec<(PathBuf, Option<std::time::SystemTime>)> = {
					let file_cache = internal.lock().unwrap();
					file_cache
						.cache()
						.iter()
						.map(|e| (e.0.clone(), e.1.modification_time().to_owned()))
						.collect()
				};
				/*
								let base_path = {
									let file_cache = internal.lock().unwrap();
									file_cache.base_path().to_owned()
								};
				*/
				for e in cache {
					let full_path = &e.0;

					let old_modification_time = &e.1;
					let new_modification_time = match std::fs::metadata(&full_path) {
						Ok(metadata) => {
							if let Ok(time) = metadata.modified() {
								Some(time)
							} else {
								None
							}
						},
						Err(e) => {
							dbg!(&e);
							// :TODO: decide what todo when a file was deleted
							None
						},
					};

					let reload_file = match (old_modification_time, new_modification_time) {
						(Some(o), Some(n)) => n > *o,
						_ => false,
						/*
						( Some( _o ), None ) => false,	// no new time, file probably doesn't exist
						( None, Some( _n ) ) => false,	// the initial creator of the entry is responsible for queuing the entry once
						( None, None ) => false,
						*/
					};

					if reload_file {
						println!(
							"FC {:?} is outdated {:?} {:?}",
							&e.0, old_modification_time, new_modification_time
						);
						internal
							.lock()
							.unwrap()
							.loading_queue_push_back(e.0.to_path_buf());
						std::thread::sleep(std::time::Duration::from_millis(16));
					} else {
						std::thread::sleep(std::time::Duration::from_millis(1));
					}
				}
				std::thread::sleep(std::time::Duration::from_millis(100));
			}
		});

		Ok(())
	}

	fn run_watch_wait_for_file_changes(
		rx: &std::sync::mpsc::Receiver<DebouncedEvent>,
		internal: &std::sync::Arc<std::sync::Mutex<FileCacheInternal>>,
	) -> anyhow::Result<bool> {
		match rx.try_recv() {
			Ok(event) => {
				//				debug!("event: {:?}", &event);
				match event {
					DebouncedEvent::Write(full_path) | DebouncedEvent::Create(full_path) => {
						let filename = full_path;
						//								match full_path.related_to( &base_path ) {
						//									Ok( filename ) => {
						//										let filename = full_path.to_string_lossy();
						//										let filename = filename.to_string();
						if internal.lock().unwrap().cache.contains_key(&filename) {
							// check we are actually interested in this file
							//											dbg!("Watcher putting file in queue");
							//											dbg!(&filename);
							internal.lock().unwrap().loading_queue_push_back(filename);
						} else {
							//											dbg!("Not interested in ...");
							//											dbg!(&filename);
							//											dbg!(&internal.lock().unwrap().cache);
						};
						//									},
						//									Err( e ) => {
						//										dbg!(&e);
						//									},
						//								}
					},
					DebouncedEvent::NoticeWrite(_) => {
						debug!("Ignored {:?}", &event);
					},
					// :TODO: handle other cases
					o => {
						debug!("Unhandled: {:?}", &o);
					},
				}
			},
			Err(e) => {
				match e {
					/*
					std::sync::mpsc::RecvError => {
						eprintln!("Closing FileCache run_watch!");
						return Ok(true);
					},
					*/
					std::sync::mpsc::TryRecvError::Empty => {
						return Ok(false);
					},
					std::sync::mpsc::TryRecvError::Disconnected => {
						return Ok(true);
					},
					/*
										e => {
											println!("watch error: {:?}", e);
										},
					*/
				}
			},
		}
		Ok(false)
	}

	fn run_watch_wait_for_watch_changes(
		watch_change_rx: &std::sync::mpsc::Receiver<WatchChange>,
		watcher: &mut RecommendedWatcher,
	) -> anyhow::Result<bool> {
		match watch_change_rx.try_recv() {
			Ok(m) => {
				match m {
					WatchChange::Add(added) => {
						//						eprintln!("Added {:?}", &added );
						watcher.watch(added.clone(), RecursiveMode::Recursive)?;
					},
				};
			},
			Err(e) => match e {
				std::sync::mpsc::TryRecvError::Empty => {
					return Ok(false);
				},
				std::sync::mpsc::TryRecvError::Disconnected => {
					return Ok(true);
				},
			},
		};

		Ok(false)
	}

	pub async fn run_watch(&mut self) -> anyhow::Result<()> {
		let (watch_change_tx, watch_change_rx) = channel();
		{
			self.internal
				.lock()
				.unwrap()
				.set_watch_change_tx(Some(watch_change_tx));
		};
		let internal = self.internal.clone();

		std::thread::spawn(move || -> anyhow::Result<()> {
			let (tx, rx) = channel();
			let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;
			/*
			let base_path = {
				internal.lock().unwrap().base_path( ).to_owned()
			};
			*/

			//		    watcher.watch( base_path.clone(), RecursiveMode::Recursive)?;

			loop {
				let watch_changes_done =
					FileCache::run_watch_wait_for_watch_changes(&watch_change_rx, &mut watcher)?;
				let file_changes_done = FileCache::run_watch_wait_for_file_changes(&rx, &internal)?;
				//		    	select! {	// :TODO: make async
				//			    }
				if watch_changes_done || file_changes_done {
					return Ok(());
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

		let internal = self.internal.clone();
		/*
		let base_path = {
			internal.lock().unwrap().base_path( ).to_owned()
		};
		*/
		std::thread::spawn(move || {
			loop {
				let front = { internal.lock().unwrap().loading_queue_pop_front() };
				if let Some(full_path) = front {
					match FileCacheInternal::load_entry(&full_path) {
						Ok(mut entry) => {
							let new_modification_time = match std::fs::metadata(&full_path) {
								Ok(metadata) => {
									if let Ok(time) = metadata.modified() {
										Some(time)
									} else {
										None
									}
								},
								Err(e) => {
									dbg!(&e);
									// :TODO: decide what todo when a file was deleted
									None
								},
							};

							entry.set_modification_time(new_modification_time);

							internal.lock().unwrap().update_entry(&full_path, entry); //?;
						},
						Err(_) => {
							// :TODO: error handling
						},
					}
				}
				std::thread::sleep(std::time::Duration::from_millis(10));
			}
			//			Ok(())
		});

		Ok(())
	}

	pub fn update(&mut self) {
		self.internal.lock().unwrap().update();
	}

	pub fn set_base_path(&mut self, base_path: &PathBuf) {
		self.internal.lock().unwrap().set_base_path(base_path);
	}

	pub fn load(&mut self, filename: &str) -> anyhow::Result<(u32, Vec<u8>)> {
		self.internal.lock().unwrap().load(filename)
	}

	// :TODO: change String to &str
	pub fn load_string(&mut self, filename: &str) -> anyhow::Result<(u32, String)> {
		self.internal.lock().unwrap().load_string(filename)
	}

	pub fn wait_for_change_with_timeout(&mut self, timeout: Duration) {
		// :TODO: this could select on multiple futures instead of polling
		let mut timeout = timeout;
		let step = Duration::from_millis(100);
		let old_updates = self.entry_updates();
		while timeout != Duration::ZERO {
			timeout = timeout.saturating_sub(step);
			std::thread::sleep(step);
			//			eprintln!(".");
			//			dbg!(&timeout);
			let new_updates = self.entry_updates();
			if new_updates > old_updates {
				//				eprintln!("!");
				break;
			}
		}

		if timeout == Duration::ZERO {
			eprintln!("Timeout while waiting for changes!");
		}
	}

	pub fn cache_misses(&self) -> u32 {
		self.internal.lock().unwrap().cache_misses()
	}

	pub fn cache_hits(&self) -> u32 {
		self.internal.lock().unwrap().cache_hits()
	}

	pub fn entry_updates(&self) -> u32 {
		self.internal.lock().unwrap().entry_updates()
	}
}

#[derive(Derivative)]
#[derivative(Debug)]
struct FileCacheEntry {
	version:           u32,
	#[derivative(Debug = "ignore")]
	content:           Vec<u8>,
	modification_time: Option<std::time::SystemTime>,
}

impl Default for FileCacheEntry {
	fn default() -> Self {
		Self {
			version:           0,
			content:           Vec::new(),
			modification_time: None,
		}
	}
}

impl FileCacheEntry {
	pub fn version(&self) -> u32 {
		self.version
	}

	pub fn set_version(&mut self, version: u32) {
		self.version = version;
	}

	pub fn content(&self) -> &Vec<u8> {
		&self.content
	}

	pub fn set_content(&mut self, content: Vec<u8>) {
		self.content = content;
	}

	pub fn modification_time(&self) -> &Option<std::time::SystemTime> {
		&self.modification_time
	}

	pub fn set_modification_time(&mut self, modification_time: Option<std::time::SystemTime>) {
		self.modification_time = modification_time;
	}
}

#[derive(Debug)]
struct FileCacheInternal {
	base_path:             PathBuf,
	cache_misses:          u32,
	cache_hits:            u32,
	entry_updates:         u32,
	cache:                 HashMap<PathBuf, FileCacheEntry>,
	loading_queue:         VecDeque<PathBuf>,
	block_on_initial_load: bool,
	watch_change_tx:       Option<std::sync::mpsc::Sender<WatchChange>>,
}

impl FileCacheInternal {
	pub fn new() -> Self {
		Self {
			base_path:             Path::new(".").to_path_buf(),
			cache_misses:          0,
			cache_hits:            0,
			entry_updates:         0,
			cache:                 HashMap::new(),
			loading_queue:         VecDeque::new(),
			block_on_initial_load: false,
			watch_change_tx:       None,
		}
	}

	pub fn update(&mut self) {}

	pub fn enable_block_on_initial_load(&mut self) {
		self.block_on_initial_load = true;
	}

	pub fn disable_block_on_initial_load(&mut self) {
		self.block_on_initial_load = false;
	}

	pub fn set_watch_change_tx(
		&mut self,
		watch_change_tx: Option<std::sync::mpsc::Sender<WatchChange>>,
	) {
		self.watch_change_tx = watch_change_tx;
	}

	pub fn set_base_path(&mut self, base_path: &PathBuf) {
		let cwd = std::env::current_dir().unwrap();
		let full_path = Path::new(&base_path);
		let full_path = cwd.join(full_path);
		let full_path = match full_path.canonicalize() {
			Ok(c) => c,
			Err(e) => panic!(
				"Error canonicalizing base path {:?} -> {:?}",
				&base_path, &e
			),
		};

		self.base_path = full_path.to_path_buf();
	}

	pub fn base_path(&self) -> &Path {
		&self.base_path
	}

	pub fn cache(&self) -> &HashMap<PathBuf, FileCacheEntry> {
		&self.cache
	}

	pub fn update_entry(
		&mut self,
		full_path: &Path,
		mut entry: FileCacheEntry,
	) -> anyhow::Result<()> {
		self.entry_updates += 1;
		let version = if let Some(old_entry) = self.cache.get(full_path) {
			if old_entry.modification_time == entry.modification_time {
				dbg!(&old_entry);
				dbg!(&entry);
				panic!("Entry not modified! ???");
			}
			old_entry.version + 1
		} else {
			0
		};
		entry.set_version(version);
		//		dbg!(&entry);
		self.cache.insert(full_path.to_path_buf(), entry);
		Ok(())
	}

	pub fn load_entry(path: &Path) -> anyhow::Result<FileCacheEntry> {
		// :TODO: handle very large files
		match std::fs::read(&path) {
			Ok(s) => {
				let mut e = FileCacheEntry::default();
				e.set_content(s);
				Ok(e)
			},
			Err(e) => {
				anyhow::bail!("Error opening file {:?} -> {:?}", &path, &e)
			},
		}
	}

	pub fn load(&mut self, filename: &str) -> anyhow::Result<(u32, Vec<u8>)> {
		let full_filename = &self.base_path.join(Path::new(&filename));
		//		dbg!(&full_filename);
		let full_filename = full_filename.to_path_buf();
		let full_filename = full_filename.canonicalize()?;

		if let Some(cached) = &self.cache.get(&full_filename) {
			self.cache_hits += 1;
			Ok((cached.version(), cached.content().clone()))
		} else {
			self.cache_misses += 1;
			if let Some(tx) = &self.watch_change_tx {
				let _ = tx.send(WatchChange::Add(full_filename.clone()));
			}

			if self.block_on_initial_load {
				match FileCacheInternal::load_entry(&full_filename) {
					Ok(entry) => {
						let s = entry.content().clone();
						let v = entry.version();
						self.update_entry(&full_filename, entry)?;
						Ok((v, s))
					},
					Err(e) => {
						// :TODO: error handling
						anyhow::bail!("TODO {:?}", e)
					},
				}
			} else {
				let entry = FileCacheEntry::default();
				let s = entry.content().clone();
				let v = entry.version();

				self.update_entry(&full_filename, entry)?;
				//				dbg!("load_string putting default entry on loading queue");
				//				dbg!(&filename);
				self.loading_queue_push_back(full_filename);
				Ok((v, s))
			}
		}
	}

	// :TODO: change String to &str
	pub fn load_string(&mut self, filename: &str) -> anyhow::Result<(u32, String)> {
		let (version, data) = self.load(filename)?;
		Ok((version, String::from_utf8_lossy(&data).to_string()))
	}

	pub fn loading_queue_pop_front(&mut self) -> Option<PathBuf> {
		self.loading_queue.pop_front()
	}

	pub fn loading_queue_push_back(&mut self, entry: PathBuf) {
		self.loading_queue.push_back(entry);
	}

	pub fn cache_misses(&self) -> u32 {
		self.cache_misses
	}

	pub fn cache_hits(&self) -> u32 {
		self.cache_hits
	}

	pub fn entry_updates(&self) -> u32 {
		self.entry_updates
	}
}

#[cfg(test)]
mod test {

	use std::fs::File;
	use std::io::Write;
	use std::path::Path;

	use tracing_test::traced_test;

	use crate::file_cache::{FileCache, FileCacheMode};

	//	#[test]
	#[actix_rt::test]
	async fn file_cache_can_load_file_with_block_on_initial_load_enabled() -> anyhow::Result<()> {
		let mut fc = FileCache::new();
		fc.enable_block_on_initial_load();
		fc.set_base_path(&Path::new("./test").to_path_buf());

		fc.run().await?;

		let f = fc.load_string("test_text_01.txt");
		assert_eq!("01", f.unwrap().1.to_string());
		assert_eq!(1, fc.cache_misses());

		let f = fc.load_string("test_text_02.txt");
		assert_eq!("02", f.unwrap().1.to_string());
		assert_eq!(2, fc.cache_misses());

		let f = fc.load_string("test_text_01.txt");
		assert_eq!("01", f.unwrap().1.to_string());
		assert_eq!(2, fc.cache_misses());
		assert_eq!(1, fc.cache_hits());

		Ok(())
	}

	#[actix_rt::test]
	async fn file_cache_can_load_file_with_block_on_initial_load_disabled() -> anyhow::Result<()> {
		let mut fc = FileCache::new();
		fc.disable_block_on_initial_load();
		fc.set_base_path(&Path::new("./test").to_path_buf());

		fc.run().await?;

		let f = fc.load_string("test_text_01.txt");
		assert_eq!("", f.unwrap().1.to_string());
		assert_eq!(1, fc.cache_misses());

		std::thread::sleep(std::time::Duration::from_millis(200));

		let f = fc.load_string("test_text_01.txt"); // cache hit
		assert_eq!("01", f.unwrap().1.to_string());
		assert_eq!(1, fc.cache_misses());
		assert_eq!(1, fc.cache_hits());

		let f = fc.load_string("test_text_02.txt");
		assert_eq!("", f.unwrap().1.to_string());
		assert_eq!(2, fc.cache_misses());

		std::thread::sleep(std::time::Duration::from_millis(200));

		let f = fc.load_string("test_text_02.txt"); // cache hit
		assert_eq!("02", f.unwrap().1.to_string());
		assert_eq!(2, fc.cache_misses());
		assert_eq!(2, fc.cache_hits());

		let f = fc.load_string("test_text_01.txt"); // cache hit
		assert_eq!("01", f.unwrap().1.to_string());
		assert_eq!(2, fc.cache_misses());
		assert_eq!(3, fc.cache_hits());

		Ok(())
	}

	#[actix_rt::test]
	async fn file_cache_can_load_file_in_poll_mode_with_block_on_initial_load_enabled(
	) -> anyhow::Result<()> {
		let mut fc = FileCache::new();
		fc.set_mode(FileCacheMode::Poll);
		fc.enable_block_on_initial_load();
		fc.set_base_path(&Path::new("./test").to_path_buf());

		fc.run().await?;

		let f = fc.load_string("test_text_01.txt");
		assert_eq!("01", f.unwrap().1.to_string());
		assert_eq!(1, fc.cache_misses());

		let f = fc.load_string("test_text_02.txt");
		assert_eq!("02", f.unwrap().1.to_string());
		assert_eq!(2, fc.cache_misses());

		let f = fc.load_string("test_text_01.txt");
		assert_eq!("01", f.unwrap().1.to_string());
		assert_eq!(2, fc.cache_misses());
		assert_eq!(1, fc.cache_hits());

		Ok(())
	}

	#[actix_rt::test]
	async fn file_cache_can_load_file_in_poll_mode_with_block_on_initial_load_disabled(
	) -> anyhow::Result<()> {
		let mut fc = FileCache::new();
		fc.set_mode(FileCacheMode::Poll);
		fc.disable_block_on_initial_load();
		fc.set_base_path(&Path::new("./test").to_path_buf());

		fc.run().await?;

		let f = fc.load_string("test_text_01.txt");
		assert_eq!("", f.unwrap().1.to_string());
		assert_eq!(1, fc.cache_misses());

		std::thread::sleep(std::time::Duration::from_millis(200));

		let f = fc.load_string("test_text_01.txt"); // cache hit
		assert_eq!("01", f.unwrap().1.to_string());
		assert_eq!(1, fc.cache_misses());
		assert_eq!(1, fc.cache_hits());

		let f = fc.load_string("test_text_02.txt");
		assert_eq!("", f.unwrap().1.to_string());
		assert_eq!(2, fc.cache_misses());

		std::thread::sleep(std::time::Duration::from_millis(200));

		let f = fc.load_string("test_text_02.txt"); // cache hit
		assert_eq!("02", f.unwrap().1.to_string());
		assert_eq!(2, fc.cache_misses());
		assert_eq!(2, fc.cache_hits());

		let f = fc.load_string("test_text_01.txt"); // cache hit
		assert_eq!("01", f.unwrap().1.to_string());
		assert_eq!(2, fc.cache_misses());
		assert_eq!(3, fc.cache_hits());

		Ok(())
	}

	#[actix_rt::test]
	async fn file_cache_can_load_and_update_file_in_poll_mode_with_block_on_initial_load_disabled(
	) -> anyhow::Result<()> {
		let mut fc = FileCache::new();
		fc.set_mode(FileCacheMode::Poll);
		fc.disable_block_on_initial_load();
		fc.set_base_path(&Path::new("./test").to_path_buf());

		fc.run().await?;

		let test_file = "auto_test_poll.txt";
		let test_file_with_dir = "./test/auto_test_poll.txt";
		// write "01" to test_file
		{
			let mut file = File::create(&test_file_with_dir)?;
			file.write_all(b"01")?;
		}

		std::thread::sleep(std::time::Duration::from_millis(2000));

		let f = fc.load_string(&test_file);
		assert_eq!("", f.unwrap().1.to_string());
		assert_eq!(1, fc.entry_updates());

		std::thread::sleep(std::time::Duration::from_millis(2000));

		let f = fc.load_string(&test_file); // cache hit
		assert_eq!("01", f.unwrap().1.to_string());
		assert_eq!(2, fc.entry_updates());

		// write "02" to test_file
		{
			let mut file = File::create(&test_file_with_dir)?;
			file.write_all(b"02")?;
			dbg!("Wrote 02 to test file");
		}

		std::thread::sleep(std::time::Duration::from_millis(2500));

		let f = fc.load_string(&test_file); // cache hit
		assert_eq!("02", f.unwrap().1.to_string());
		assert_eq!(3, fc.entry_updates());

		std::thread::sleep(std::time::Duration::from_millis(2500));

		assert_eq!(3, fc.entry_updates());

		Ok(())
	}

	#[actix_rt::test]
	async fn file_cache_can_load_and_update_file_in_watch_mode_with_block_on_initial_load_disabled(
	) -> anyhow::Result<()> {
		let mut fc = FileCache::new();
		fc.set_mode(FileCacheMode::Watch);
		fc.disable_block_on_initial_load();
		fc.set_base_path(&Path::new("./test").to_path_buf());

		fc.run().await?;

		let test_file = "auto_test_watch.txt";
		let test_file_with_dir = "./test/auto_test_watch.txt";
		// write "01" to test_file
		{
			let mut file = File::create(&test_file_with_dir)?;
			file.write_all(b"01")?;
		}

		let f = fc.load_string(&test_file);
		assert_eq!("", f.unwrap().1.to_string());
		assert_eq!(1, fc.entry_updates());

		fc.wait_for_change_with_timeout(std::time::Duration::from_millis(60000));

		let f = fc.load_string(&test_file); // cache hit
		assert_eq!("01", f.unwrap().1.to_string());
		assert_eq!(2, fc.entry_updates());

		// write "02" to test_file
		{
			let mut file = File::create(&test_file_with_dir)?;
			file.write_all(b"02")?;
			dbg!("Wrote 02 to test file");
		}

		fc.wait_for_change_with_timeout(std::time::Duration::from_millis(60000));

		let f = fc.load_string(&test_file); // cache hit
		assert_eq!("02", f.unwrap().1.to_string());
		assert_eq!(3, fc.entry_updates());

		Ok(())
	}

	#[actix_rt::test]
	#[traced_test]
	async fn file_cache_can_update_vector_clock_in_watch_mode_with_block_on_initial_load_disabled(
	) -> anyhow::Result<()> {
		let mut fc = FileCache::new();
		fc.set_mode(FileCacheMode::Watch);
		fc.disable_block_on_initial_load();
		fc.set_base_path(&Path::new("./test").to_path_buf());

		fc.run().await?;

		let test_file = "auto_test_vector_watch.txt";
		let test_file_with_dir = "./test/auto_test_vector_watch.txt";
		// write "01" to test_file
		{
			let mut file = File::create(&test_file_with_dir)?;
			file.write_all(b"01")?;
			dbg!("Wrote 01 to test file");
		}

		let f = fc.load_string(&test_file);
		let f = f.unwrap();
		assert_eq!("", f.1.to_string());
		assert_eq!(0, f.0);
		assert_eq!(1, fc.entry_updates());

		fc.wait_for_change_with_timeout(std::time::Duration::from_millis(60000));

		let f = fc.load_string(&test_file); // cache hit
		let f = f.unwrap();
		assert_eq!("01", f.1.to_string());
		assert_eq!(1, f.0);
		assert_eq!(2, fc.entry_updates());

		// write "02" to test_file
		{
			let mut file = File::create(&test_file_with_dir)?;
			file.write_all(b"02")?;
			dbg!("Wrote 02 to test file");
		}

		fc.wait_for_change_with_timeout(std::time::Duration::from_millis(60000));

		let f = fc.load_string(&test_file); // cache hit
		let f = f.unwrap();
		assert_eq!("02", f.1.to_string());
		assert_eq!(2, f.0);
		assert_eq!(3, fc.entry_updates());

		Ok(())
	}

	#[actix_rt::test]
	async fn file_cache_can_update_vector_clock_for_binary_file_in_watch_mode_with_block_on_initial_load_disabled(
	) -> anyhow::Result<()> {
		let mut fc = FileCache::new();
		fc.set_mode(FileCacheMode::Watch);
		fc.disable_block_on_initial_load();
		fc.set_base_path(&Path::new("./test").to_path_buf());

		fc.run().await?;

		let test_file = "auto_test_vector_binary_watch.txt";
		let test_file_with_dir = "./test/auto_test_vector_binary_watch.txt";
		// write "01" to test_file
		{
			let mut file = File::create(&test_file_with_dir)?;
			file.write_all(b"01")?;
		}

		let f = fc.load(&test_file);
		let f = f.unwrap();
		assert_eq!(b"", f.1.as_slice());
		assert_eq!(0, f.0);
		assert_eq!(1, fc.entry_updates());

		fc.wait_for_change_with_timeout(std::time::Duration::from_millis(60000));

		let f = fc.load(&test_file); // cache hit
		let f = f.unwrap();
		assert_eq!(b"01", f.1.as_slice());
		assert_eq!(1, f.0);
		assert_eq!(2, fc.entry_updates());

		// write "02" to test_file
		{
			let mut file = File::create(&test_file_with_dir)?;
			file.write_all(b"02")?;
			dbg!("Wrote 02 to test file");
		}

		fc.wait_for_change_with_timeout(std::time::Duration::from_millis(60000));

		let f = fc.load(&test_file); // cache hit
		let f = f.unwrap();
		assert_eq!(b"02", f.1.as_slice());
		assert_eq!(2, f.0);
		assert_eq!(3, fc.entry_updates());

		Ok(())
	}

	#[actix_rt::test]
	async fn file_cache_can_update_vector_clock_for_binary_file_out_of_tree_in_watch_mode_with_block_on_initial_load_disabled(
	) -> anyhow::Result<()> {
		/*
			test/aaa/1.txt
			test/bbb/2.txt
			test/aaa

		*/
		let mut fc = FileCache::new();
		fc.set_mode(FileCacheMode::Watch);
		fc.disable_block_on_initial_load();
		fc.set_base_path(&Path::new("./test/aaa").to_path_buf());

		fc.run().await?;

		{
			let test_file = "auto_test_vector_binary_watch.txt";
			let test_file_with_dir = "./test/aaa/auto_test_vector_binary_watch.txt";
			// write "01" to test_file
			{
				let mut file = File::create(&test_file_with_dir)?;
				file.write_all(b"01")?;
			}

			let f = fc.load(&test_file);
			let f = f.unwrap();
			assert_eq!(b"", f.1.as_slice());
			assert_eq!(0, f.0);
			assert_eq!(1, fc.entry_updates());

			fc.wait_for_change_with_timeout(std::time::Duration::from_millis(60000));

			let f = fc.load(&test_file); // cache hit
			let f = f.unwrap();
			assert_eq!(b"01", f.1.as_slice());
			assert_eq!(1, f.0);
			assert_eq!(2, fc.entry_updates());

			// write "02" to test_file
			{
				let mut file = File::create(&test_file_with_dir)?;
				file.write_all(b"02")?;
				dbg!("Wrote 02 to test file");
			}

			fc.wait_for_change_with_timeout(std::time::Duration::from_millis(60000));

			let f = fc.load(&test_file); // cache hit
			let f = f.unwrap();
			assert_eq!(b"02", f.1.as_slice());
			assert_eq!(2, f.0);
			assert_eq!(3, fc.entry_updates());
		}

		{
			let test_file = "../bbb/auto_test_vector_binary_watch.txt";
			let test_file_with_dir = "./test/bbb/auto_test_vector_binary_watch.txt";
			// write "01" to test_file
			{
				let mut file = File::create(&test_file_with_dir)?;
				file.write_all(b"01")?;
			}

			let f = fc.load(&test_file);
			let f = f.unwrap();
			assert_eq!(b"", f.1.as_slice());
			assert_eq!(0, f.0);
			assert_eq!(4, fc.entry_updates());

			fc.wait_for_change_with_timeout(std::time::Duration::from_millis(60000));

			let f = fc.load(&test_file); // cache hit
			let f = f.unwrap();
			assert_eq!(b"01", f.1.as_slice());
			assert_eq!(1, f.0);
			assert_eq!(5, fc.entry_updates());

			// write "02" to test_file
			{
				let mut file = File::create(&test_file_with_dir)?;
				file.write_all(b"02")?;
				dbg!("Wrote 02 to test file");
			}

			fc.wait_for_change_with_timeout(std::time::Duration::from_millis(60000));

			let f = fc.load(&test_file); // cache hit
			let f = f.unwrap();
			assert_eq!(b"02", f.1.as_slice());
			assert_eq!(2, f.0);
			assert_eq!(6, fc.entry_updates());
		}

		Ok(())
	}
}
