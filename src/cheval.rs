use std::collections::HashMap;
use std::convert::TryInto;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use chrono::{DateTime, Utc};
use derivative::Derivative;
use hhmmss::Hhmmss;
use serde::Deserialize;
use serde_yaml;
use tracing::*;
use tokio::runtime::Runtime;

use crate::block_element::BlockElementFactory;
use crate::context::Context;
use crate::element::{Element, ElementConfig};
use crate::element_instance::ElementInstance;
use crate::file_cache::FileCache;
use crate::image_element::ImageElementFactory;
use crate::lissajous_element::LissajousElementFactory;
use crate::page::Page;
use crate::render_buffer::RenderBuffer;
use crate::render_context::RenderContext;
use crate::scrolltext_element::ScrollTextElementFactory;
use crate::soundbank_element::SoundbankElementFactory;
use crate::text_element::TextElementFactory;
use crate::timer_element::TimerElementFactory;
use crate::control::{
	Message,
	Response
};
use crate::HttpApi;


#[derive(Derivative)]
#[derivative(Debug)]
pub struct Cheval {
	//	element_instances: Vec< ElementInstance >,
	page:              Option<Page>,
	pages:             Vec<Page>,
	active_page:       usize,
	variable_filename: Option<String>,
	context:           Context,
	last_update_time:  DateTime<Utc>,
	start_time:        DateTime<Utc>,
	render_context:    RenderContext,
	http_enabled:      bool,
	http_receiver:     Option<mpsc::Receiver<Message>>,
	done:              bool,
	config_path:       PathBuf,
	http_api:			Option< HttpApi >,
	file_cache:        std::sync::Arc<std::sync::Mutex<FileCache>>,
}

#[derive(Debug, Deserialize)]
struct ConfigElement {
	name:       String,
	#[serde(rename = "type")]
	the_type:   String,
	#[serde(default = "default_bool_false")]
	disabled:   bool,
	#[serde(default = "default_bool_true")]
	visible:    bool,
	parameters: HashMap<String, String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ConfigPage {
	name:       String,
	elements:   Vec<ConfigElement>,
	parameters: Option<HashMap<String, String>>,
}
fn default_bool_false() -> bool {
	false
}

fn default_bool_true() -> bool {
	true
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
	default_page:      Option<usize>,
	variable_filename: Option<String>,
	variable_defaults: Option<HashMap<String, String>>,
	pages:             Option<Vec<ConfigPage>>,
	elements:          Option<Vec<ConfigElement>>,
}


impl Cheval {
	pub fn new() -> Self {
		let file_cache = std::sync::Arc::new(std::sync::Mutex::new(FileCache::new()));
		let mut context = Context::new();
		context.set_file_cache(file_cache.clone());
		// :HACK:
		{
			let cheval_active_page_number = format!("{}", 0);
			context.set_string(
				"cheval_active_page_number",
				&cheval_active_page_number.to_string(),
			);
		}
		Self {
			//			element_instances: Vec::new(),
			page: None,
			pages: Vec::new(),
			active_page: 1,
			variable_filename: None,
			context,
			last_update_time: Utc::now(),
			start_time: Utc::now(),
			render_context: RenderContext::new(),
			http_enabled: false,
			//			http_server: None,
			http_receiver: None,
			done: false,
			config_path: PathBuf::new(),
			http_api:		None,
			file_cache,
		}
	}

	pub fn done(&self) -> bool {
		self.done
	}

	pub fn enable_http(&mut self) {
		self.http_enabled = true;
	}

	async fn load_elements_for_page(
		&self,
		page: &mut Page,
		config_page_elements: &Vec<ConfigElement>,
	) -> anyhow::Result<()> {
		for e in config_page_elements {
			if e.disabled {
				continue;
			};
			let mut element: Box<dyn Element + Send> = match e.the_type.as_ref() {
				"block" => Box::new(BlockElementFactory::create()) as Box<dyn Element + Send>,
				"timer" => Box::new(TimerElementFactory::create()) as Box<dyn Element + Send>,
				"lissajous" => Box::new(LissajousElementFactory::create()),
				"image" => Box::new(ImageElementFactory::create()),
				"text" => Box::new(TextElementFactory::create()),
				"scrolltext" => Box::new(ScrollTextElementFactory::create()),
				"soundbank" => Box::new(SoundbankElementFactory::create()),
				//				_ => panic!("Unsupported element type {}", e.the_type ),
				_ => {
					println!("Skipping unsupported element type {}", e.the_type);
					continue;
				},
			};

			element.set_name(&e.name);

			let mut element_config = ElementConfig::new(&self.config_path.as_path());

			for p in &e.parameters {
				element_config.set(&p.0, &p.1);
			}

			debug!("element_config: {:?}", &element_config);

			element.configure(&element_config);

			let mut element_instance = ElementInstance::new(element);
			if e.visible {
				element_instance.show();
			} else {
				element_instance.hide();
			};

			page.add_element_instance(element_instance);
		}

		page.run().await?;
		/*
		for p in self.pages.iter_mut() {
			p.run().await?;
		}
		*/

		Ok(())
	}

	pub async fn load(&mut self, config_file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
//		dbg!(&config_file_name);
		debug!("Loading config from {}", &config_file_name);
		// make path absolute
		let cwd = std::env::current_dir().unwrap();
		let config_file_name = Path::new(&config_file_name);
		let config_file_name = cwd.join(config_file_name);
		debug!("Loading config from {:?}", &config_file_name);
		let config_file_name = match FileCache::canonicalize( config_file_name.as_path() ) {
			Ok(c) => c,
			Err(e) => panic!(
				"Error canonicalizing config file {:?} -> {:?}",
				&config_file_name, &e
			),
		};
		debug!("Loading config from {:?}", &config_file_name);

		let config_file_name = if config_file_name.is_dir() {
			let mut cfn = config_file_name.clone();
			cfn.push("config.yaml");
			if cfn.is_file() {
				cfn
			} else {
				let mut glob = config_file_name.clone();
				glob.push("*config.yaml");
				let glob = glob.to_string_lossy().to_string();
				let mut result = glob::glob(&glob).expect("Failed to read glob pattern");

				// :TODO: check if something like take(2).to_vec() result in more readable code

				match result.next() {
					None => todo!("no config found"),
					Some(fr) => match result.next() {
						None => fr.unwrap(),
						Some(_) => todo!("More than one config found"),
					},
				}
			}
		} else {
			config_file_name
		};

		if let Some(config_path) = config_file_name.parent() {
			self.config_path = PathBuf::from(&config_path);

			// update file cache base
			{
				let mut fc = self.file_cache.lock().unwrap();
				fc.set_base_path(&self.config_path);

				fc.set_mode(crate::file_cache::FileCacheMode::Watch);
			}
		};

		let cf = match std::fs::File::open(&config_file_name) {
			Ok(f) => f,
			Err(e) => panic!(
				"Error opening config file {:?} -> {:?}",
				&config_file_name, &e
			),
		};

		debug!("config: {:?}", &cf);

		let config: Config = match serde_yaml::from_reader(&cf) {
			Ok(c) => c,
			Err(e) => panic!("Error deserializing config file {:?} -> {:?}", &cf, &e),
		};

		//		dbg!(&config);

		if let Some(variable_filename) = &config.variable_filename {
			self.variable_filename = Some(variable_filename.clone());
			match self
				.context
				.get_mut_machine()
				.load_variable_storage(&variable_filename)
			{
				Ok(_) => {},
				Err(_) => {},
				//				r => todo!("{:?}", r),
			};

			println!("Loaded variables from {}", &variable_filename);
			debug!("{:?}", self.context.get_mut_machine());
		}

		// :HACK: load variable default
		if let Some(defaults) = &config.variable_defaults {
			for (key, val) in defaults.iter() {
				debug!("{:?} = {:?}", &key, &val);
				let vs = self.context.get_mut_machine().get_mut_variable_storage();
				if vs.get(&key).is_none() {
					println!("Variable {} not found using default {}", &key, &val);
					// :TODO: handle more variable types
					if let Ok(v) = val.parse::<i32>() {
						vs.set(&key, expresso::variables::Variable::I32(v));
					} else if let Ok(v) = val.parse::<f32>() {
						vs.set(&key, expresso::variables::Variable::F32(v));
					} else {
						vs.set(&key, expresso::variables::Variable::String(val.clone()));
					}
				};
			}
		};

		// :HACK:
		{
			let function_table = self.context.get_mut_machine().get_mut_function_table();

			function_table.register("sin", |_argc, variable_stack, _variable_storage| {
				// :TODO: handle wrong argc

				let fv = variable_stack.pop_as_f32();

				let r = fv.sin();

				variable_stack.push(expresso::variables::Variable::F32(r));
				true
			});

			function_table.register("printHHMMSS", |argc, variable_stack, _variable_storage| {
				if argc == 1 {
					let f = variable_stack.pop_as_f32();
					let duration = std::time::Duration::new(f as u64, 0);

					variable_stack.push(expresso::variables::Variable::String(duration.hhmmss()));
					true
				} else {
					false
				}
			});

			let file_cache = self.file_cache.clone();
			function_table.register(
				"text_from_file",
				move |argc, variable_stack, _variable_storage| {
					if argc == 1 {
						let filename = variable_stack.pop_as_string();
						// :TODO: check stack validity here?!

						let mut lock = file_cache.try_lock();
						if let Ok(ref mut file_cache) = lock {
							match file_cache.load_string(&filename) {
								Ok((_v, s)) => {
									debug!("{:?}", &s);
									variable_stack.push(expresso::variables::Variable::String(s));
									true
								},
								Err(e) => {
									let s = format!("Error: {:?}", &e);
									variable_stack.push(expresso::variables::Variable::String(s));
									false
								},
							}
						} else {
							warn!("try_lock failed");
							false
						}
					} else {
						false
					}
				},
			);
			let file_cache = self.file_cache.clone();
			function_table.register(
				"text_lines_from_file",
				move |argc, variable_stack, _variable_storage| {
					if argc == 3 {
						let filename = variable_stack.pop_as_string();
						let count = variable_stack.pop_as_i32();
						let skip = variable_stack.pop_as_i32();
						// :TODO: check stack validity here?!

						let mut lock = file_cache.try_lock();
						if let Ok(ref mut file_cache) = lock {
							match file_cache.load_string(&filename) {
								Ok((_v, s)) => {
									let s = s.split("\n").skip(skip as usize);
									let s = if count > 0 {
										s.take(count as usize).collect::<Vec<&str>>()
									} else {
										// zero -> take everything
										s.collect::<Vec<&str>>()
									};

									let s = s.join("\n");
									variable_stack.push(expresso::variables::Variable::String(s));
									true
								},
								Err(e) => {
									let s = format!("Error: {:?}", &e);
									variable_stack.push(expresso::variables::Variable::String(s));
									false
								},
							}
						} else {
							println!("try_lock failed");
							false
						}
					} else {
						false
					}
				},
			);
		}

		self.file_cache.lock().unwrap().run().await?;

		// -- :HACK:

		if let Some(default_page) = &config.default_page {
			self.active_page = *default_page;
		}

		debug!("{:?}", &config);
		if let Some(elements) = &config.elements {
			let mut page = Page::new(); // global/top page

			self.load_elements_for_page(&mut page, &elements).await?;

			page.show();
			self.page = Some(page);
		}

		// :TODO: allow start page to be configured

		if let Some(pages) = &config.pages {
			for active_page_config in pages {
				let mut page = Page::new(); // sub page
				page.set_name(&active_page_config.name);

				if let Some(parameters) = &active_page_config.parameters {
					let mut page_config = ElementConfig::new(&self.config_path.as_path());

					for p in parameters.iter() {
						page_config.set(&p.0, &p.1);
					}

					debug!("{:?}", &page_config);

					page.configure(&page_config);

					debug!("{:?}", &page);
				}

				self.load_elements_for_page(&mut page, &active_page_config.elements)
					.await?;

				dbg!(self.pages.len(), &self.active_page);
				if self.pages.len() == self.active_page {
					page.show();
				}
				self.pages.push(page);
			}
		}
		println!("Running...");
		Ok(())
	}

	fn goto_page(&mut self, page_no: usize) -> (Option<usize>, Option<usize>) {
		let mut old_page_no = None;
		let mut new_page_no = None;
		old_page_no = Some(self.active_page);
		new_page_no = Some(page_no);

		if self.active_page != page_no {
			if let Some(old_page) = self.pages.get_mut(self.active_page) {
				old_page_no = Some(self.active_page);
				old_page.hide();
			}
			self.active_page = page_no;
			if let Some(page) = self.pages.get_mut(page_no) {
				new_page_no = Some(page_no);
				page.show();
			} else {
				new_page_no = None;
			}
		}

		let cheval_active_page_number = format!("{}", self.active_page);
		self.context.set_string(
			"cheval_active_page_number",
			&cheval_active_page_number.to_string(),
		);

		(new_page_no, old_page_no)
	}

	fn goto_page_name(&mut self, page_name: &str) -> (Option<usize>, Option<usize>) {
		let mut old_page_no = None;
		let mut new_page_no = None;
		old_page_no = Some(self.active_page);

		let page_no = if let Some(pos) = self.pages.iter().position(|p| p.name() == page_name) {
			pos
		} else {
			return (new_page_no, old_page_no);
		};

		new_page_no = Some(page_no);

		if self.active_page != page_no {
			if let Some(old_page) = self.pages.get_mut(self.active_page) {
				old_page_no = Some(self.active_page);
				old_page.hide();
			}
			self.active_page = page_no;
			if let Some(page) = self.pages.get_mut(page_no) {
				new_page_no = Some(page_no);
				page.show();
			} else {
				new_page_no = None;
			}
		}

		let cheval_active_page_number = format!("{}", self.active_page);
		self.context.set_string(
			"cheval_active_page_number",
			&cheval_active_page_number.to_string(),
		);

		(new_page_no, old_page_no)
	}

	fn goto_next_page(&mut self) -> (Option<usize>, Option<usize>) {
		let page_no = if self.pages.len() > 0 {
			(self.active_page + 1) % self.pages.len()
		} else {
			0
		};
		self.goto_page(page_no)
	}

	fn goto_prev_page(&mut self) -> (Option<usize>, Option<usize>) {
		let page_no = if self.active_page > 0 {
			self.active_page - 1
		} else {
			self.pages.len() - 1
		};
		self.goto_page(page_no)
	}

	/*
		pub fn add_element_instance( &mut self, element_instance: ElementInstance ) {
			self.element_instances.push( element_instance );
		}
	*/
	// :TODO: decide how to handle name collisions
	pub fn run_for_element_instance_with_name(
		&mut self,
		name: &str,
		func: Box<dyn Fn(&mut ElementInstance)>,
	) {
		if let Some(p) = &mut self.page {
			p.run_for_element_instance_with_name(name, &func);
		}

		for p in &mut self.pages {
			p.run_for_element_instance_with_name(name, &func);
		}
	}

	pub fn initialize(&mut self) -> anyhow::Result<()> {

		let (tx2, rx2) = mpsc::channel();

		self.http_receiver = Some(rx2);

		if self.http_enabled {
			let mut http_api = HttpApi::new( tx2.clone() );

//			http_api.run();

			self.http_api = Some( http_api );
		}

		debug!("http_enabled: {:?}", &self.http_enabled);
		Ok(())
	}

	pub fn run( &mut self ) -> anyhow::Result<()> {
		if let Some( http_api ) = self.http_api.take() {
			
			std::thread::spawn( move || -> anyhow::Result<()>  {
				let mut rt  = Runtime::new().unwrap();
				rt.block_on( async { http_api.run().await } );
				//let sys = actix_web::rt::System::new();
				//sys.block_on( http_api.run() )?;
				//anyhow::bail!("run ended");
				debug!("run ended");
				Ok(())
			} );
			
		}

		Ok(())
	}

	pub fn update(&mut self) {
		// :TODO: create a date time element that provides info
		let now: DateTime<Utc> = Utc::now();
		let clock_string = now.format("%H:%M:%S");
		self.context
			.set_string("clock_string", &clock_string.to_string());

		let frametime_duration = now.signed_duration_since(self.last_update_time);
		let frametime = frametime_duration.num_milliseconds() as f64;
		let frametime_string = format!("{}", frametime);
		self.context
			.set_string("frametime_string", &frametime_string);
		self.last_update_time = now;
		self.context.set_time_step(frametime / 1000.0);

		let time_since_start = now.signed_duration_since(self.start_time);
		let time_since_start = time_since_start.num_milliseconds() as f64 / 1000.0;

		self.context
			.get_mut_machine()
			.get_mut_variable_storage()
			.set(
				"time",
				expresso::variables::Variable::F32(time_since_start as f32),
			);

		if let Some(p) = &mut self.page {
			p.update(&mut self.context);
		}
		for p in &mut self.pages {
			p.update(&mut self.context);
		}

		let ts = self.context.time_step();
		let soundbank = &mut self.context.get_soundbank_mut();
		soundbank.update(ts);

		match self.file_cache.try_lock() {
			Ok(ref mut file_cache) => {
				file_cache.update();
			},
			_ => {},
			//			if let Ok(ref mut file_cache) = lock {
		}

		if let Some(http_receiver) = &self.http_receiver {
			match http_receiver.try_recv() {
				Ok(msg) => {
					//					dbg!("http_receiver got message", &msg);
					match msg {
						Message::SelectNextVariable(result_sender, maybe_prefix) => {
							let name = self
								.context
								.select_next_variable(maybe_prefix.as_ref().map(String::as_str));
							match result_sender.send(Response::VariableSelected(name.to_string())) {
								_ => {},
							};
						},
						Message::SetVariable(result_sender, name, value) => {
							debug!("set variable {} => {}", &name, &value);
							if let Ok(v) = value.parse::<u32>() {
								self.context.set_f32(&name, v as f32);
								match result_sender
									.send(Response::VariableU32Changed(name.clone(), v))
								{
									_ => {},
								};
							} else if let Ok(v) = value.parse::<f32>() {
								self.context.set_f32(&name, v);
								match result_sender
									.send(Response::VariableF32Changed(name.clone(), v))
								{
									_ => {},
								};
							} else {
								self.context.set_string(&name, &value);
								match result_sender.send(Response::VariableStringChanged(
									name.clone(),
									value.clone(),
								)) {
									_ => {},
								};
							};
							debug!("{:?}", &self.context);
						},
						Message::IncrementVariable(result_sender, name, delta) => {
							debug!("inc variable {} by {}", &name, delta);
							if let Some(old) = self.context.get_f32(&name) {
								let new = old + delta as f32;
								self.context.set_f32(&name, new);
								match result_sender
									.send(Response::VariableChanged(name.clone(), new))
								{
									_ => {},
								};
							}

							debug!("{:?}", &self.context);
						},
						Message::IncrementSelectedVariable(result_sender, delta) => {
							let name = self.context.selected_variable().to_string();
							debug!("inc selected variable {} by {}", &name, delta);
							if let Some(old) = self.context.get_f32(&name) {
								let new = old + delta as f32;
								self.context.set_f32(&name, new);
								match result_sender.send(Response::VariableChanged(name, new)) {
									_ => {},
								};
							}

							debug!("{:?}", &self.context);
						},
						Message::SetElementVisibilityByName(name, visible) => {
							debug!("set visibility {} to {}", &name, &visible);
							self.run_for_element_instance_with_name(
								&name,
								Box::new(move |element_instance| {
									debug!("Found element_instance: {:?}", &element_instance);
									if visible {
										element_instance.show();
									} else {
										element_instance.hide();
									}
								}),
							);
						},
						Message::ListElementInstances(sender) => {
							match sender.send(Response::ElementInstanceList(
								"{\":TODO\": false}".to_string(),
							)) {
								_ => {},
							};
						},
						Message::GotoNextPage(sender) => {
							let (new_page_no, old_page_no) = self.goto_next_page();
							match sender.send(Response::PageChanged(new_page_no, old_page_no)) {
								_ => {},
							};
						},
						Message::GotoPrevPage(sender) => {
							let (new_page_no, old_page_no) = self.goto_prev_page();
							match sender.send(Response::PageChanged(new_page_no, old_page_no)) {
								_ => {},
							};
						},
						Message::GotoPage(sender, page_no) => {
							let (new_page_no, old_page_no) = self.goto_page(page_no);
							match sender.send(Response::PageChanged(new_page_no, old_page_no)) {
								_ => {},
							};
						},
						Message::GotoPageName(sender, page_name) => {
							let (new_page_no, old_page_no) = self.goto_page_name(&page_name);
							match sender.send(Response::PageChanged(new_page_no, old_page_no)) {
								_ => {},
							};
						},
						x => {
							debug!("unhandled {:?}", &x);
						},
					}
				},
				Err(e) => {
					match e {
						mpsc::TryRecvError::Empty => {
							// empty is fine
						},
						mpsc::TryRecvError::Disconnected => {
							// disconnected is also fine, for now
						},
					}
				},
			}
		}
	}

	pub fn render(&mut self, render_buffer: &mut RenderBuffer) {
		/*
				for e in &self.elements {
		//			dbg!(e);
					e.render( &mut render_buffer.buffer, render_buffer.width, render_buffer.height );
				};
		*/

		//		let mut render_context = RenderContext::new();
		if let Some(p) = &mut self.page {
			p.render(render_buffer, &mut self.render_context);
		}
		for p in &self.pages {
			//			dbg!(e);
			//			if e.is_visible() {
			p.render(render_buffer, &mut self.render_context);
			//			}
		}
	}

	pub fn add_key(&mut self, key: u32) {
		// :TODO: add keys to queue, and handle in update
		match key {
			27 => {
				self.done = true;
			},
			63234 => {
				// Cursor Left
				self.goto_prev_page();
			},
			63235 => {
				// Cursor Right
				self.goto_next_page();
			},
			x if x >= 48 && x <= 57 => {
				// 0
				let p = x - 48;
				self.goto_page(p as usize);
			},
			_ => {
				debug!("Got key {:?}", &key);
			},
		}
	}

	pub fn shutdown(&mut self) -> anyhow::Result<()> {
		for p in self.pages.iter_mut() {
			p.shutdown();
		}
		if let Some(p) = &mut self.page {
			p.shutdown();
		}
		if let Some(variable_filename) = &self.variable_filename {
			match self
				.context
				.get_mut_machine()
				.save_variable_storage(&variable_filename)
			{
				Ok(_) => {},
				Err(e) => return Err(anyhow::anyhow!("Error saving variables: {:?}", e)),
			}
		}
		// :TODO:
		/*
		if let Some( server_thread ) = self.server_thread.take() {
			match server_thread.join() {
				Ok( _ ) => {},
				Err( e ) => return Err(anyhow::anyhow!("Error joining server thread: {:?}", e )),
			}
		}
		*/

		//		dbg!(&self.file_cache);
		Ok(())
	}
}
