use std::collections::HashMap;

use std::path::{ Path, PathBuf };

use std::convert::TryInto;

use serde::Deserialize;
use serde_yaml;

use crate::element_instance::ElementInstance;
use crate::file_cache::FileCache;

use crate::block_element::BlockElementFactory;
use crate::lissajous_element::LissajousElementFactory;
use crate::image_element::ImageElementFactory;
use crate::scrolltext_element::ScrollTextElementFactory;
use crate::soundbank_element::SoundbankElementFactory;
use crate::text_element::TextElementFactory;
use crate::element::{Element,ElementConfig};
use crate::context::Context;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;
use crate::timer_element::TimerElementFactory;
use crate::page::Page;

use derivative::Derivative;

use chrono::{DateTime, Utc};
use hhmmss::Hhmmss;
use std::sync::mpsc;

use actix_web::{
	web::{
		self,
		Data,
	},
	App,
//	HttpRequest,
	HttpServer,
	Responder,
	rt::System
};

#[allow(dead_code)]
#[derive(Debug)]
enum Message {
	None,
	SelectNextVariable( mpsc::Sender< Response >, Option< String > ), // optional prefix
	IncrementSelectedVariable( mpsc::Sender< Response >, i32 ),
	SetVariable( mpsc::Sender< Response >, String, String ),
	IncrementVariable( mpsc::Sender< Response >, String, i32 ),
	SetElementVisibilityByName( String, bool ),
	ListElementInstances( mpsc::Sender< Response > ),
	GotoNextPage( mpsc::Sender< Response > ),
	GotoPrevPage( mpsc::Sender< Response > ),
	GotoPage( mpsc::Sender< Response >, usize ),
}

#[allow(dead_code)]
#[derive(Debug)]
enum Response {
	None,
	NotImplemented( String ),
	ElementInstanceList( String ),
	PageChanged( Option< usize >, Option< usize > ),	// new page #, old page #
	VariableSelected( String ),
	VariableChanged( String, f32 ),
	VariableU32Changed( String, u32 ),
	VariableF32Changed( String, f32 ),
	VariableStringChanged( String, String ),
}


#[derive(Debug)]
struct HttpState {
	id: String,
	http_sender: mpsc::Sender< Message >,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Cheval {
//	element_instances: Vec< ElementInstance >,
	page: Option< Page >,
	pages: Vec< Page >,
	active_page: usize,
	variable_filename: Option< String >,
	context: Context,
	last_update_time: DateTime<Utc>,
	start_time: DateTime<Utc>,
	render_context: RenderContext,
	http_enabled: bool,
//	#[derivative(Debug="ignore")]
//	http_server: Option< actix_web::dev::Server >,
	http_receiver: Option< mpsc::Receiver< Message > >,
	done: bool,
	config_path: PathBuf,
	server_thread: Option< std::thread::JoinHandle< () > >,
	file_cache: std::sync::Arc< std::sync::Mutex< FileCache > >,
}


#[derive(Debug, Deserialize)]
struct ConfigElement {
	name: String,
	#[serde(rename = "type")]
	the_type: String,
	#[serde(default = "default_bool_false")]
	disabled: bool,
	#[serde(default = "default_bool_true")]
	visible: bool,
	parameters: HashMap< String, String >
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ConfigPage {
	name: String,
	elements: Vec< ConfigElement >,	
	parameters: Option< HashMap< String, String > >
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
	default_page: Option< usize >,
	variable_filename: Option< String >,
	variable_defaults: Option< HashMap< String, String > >,
	pages: Option< Vec< ConfigPage > >,
	elements: Option< Vec< ConfigElement > >,
}

	fn handle_response( rx: mpsc::Receiver< Response >) -> impl Responder {
		match rx.recv() {
			Ok( r ) => {
				match r {
					Response::VariableSelected( name ) => {
						format!("variable selected: {}", &name ) // :TODO: decide on formatting
					},
					Response::VariableChanged( name, v ) => {
						format!("{{\"variables\":[{{ \"{}\": {}}}]}}", &name, v)
					},
					Response::VariableU32Changed( name, v ) => {
						format!("{{\"variables\":[{{ \"{}\": {}}}]}}", &name, v)
					},
					Response::VariableF32Changed( name, v ) => {
						format!("{{\"variables\":[{{ \"{}\": {}}}]}}", &name, v)
					},
					Response::VariableStringChanged( name, v ) => {
						format!("{{\"variables\":[{{ \"{}\": \"{}\"}}]}}", &name, &v)
					},
					o => {
						format!("Unhandled response: {:?}", &o ) // :TODO: format as json			
					},
				}
			},
			Err( e ) => format!("Error: {:?}", &e ), // :TODO: format as json
		}
	}

	async fn select_next_variable(
		state: web::Data<HttpState>,
	) -> impl Responder {
		let (tx,rx) = std::sync::mpsc::channel();		
		match state.http_sender.send( Message::SelectNextVariable( tx, None ) ) {
			_ => {},
		};

		handle_response( rx )
	}

	async fn select_next_variable_with_prefix(
		state: web::Data<HttpState>,
		path: web::Path<String>
	) -> impl Responder {
		let prefix = path.into_inner();
		let (tx,rx) = std::sync::mpsc::channel();		
		match state.http_sender.send( Message::SelectNextVariable( tx, Some( prefix ) ) ) {
			_ => {},
		};

		handle_response( rx )
	}

	async fn set_variable(
		state: web::Data<HttpState>,		
		path : web::Path<(String, String)>
	) -> impl Responder {
		let (name, value) = path.into_inner();
		let (tx,rx) = std::sync::mpsc::channel();		
		match state.http_sender.send( Message::SetVariable( tx, name.clone(), value.clone() ) ) {
			_ => {},
		};

		handle_response( rx )
	}

	async fn inc_variable(
		state: web::Data<HttpState>,
		path: web::Path<(String, u32)>
	) -> impl Responder {
		let (name,delta) = path.into_inner();
		let (tx,rx) = std::sync::mpsc::channel();
		match state.http_sender.send( Message::IncrementVariable( tx, name.clone(), delta.try_into().unwrap() ) ) {
			_ => {},
		};
		handle_response( rx )
	}

	async fn dec_variable(
		state: web::Data<HttpState>,
		path: web::Path<(String, u32)>
	) -> impl Responder {
		let (name,delta) = path.into_inner();
		let v: i32 = delta.try_into().unwrap();
		let (tx,rx) = std::sync::mpsc::channel();
		match state.http_sender.send( Message::IncrementVariable( tx, name.clone(), -v ) ) {
			_ => {},
		};
		handle_response( rx )
	}

	async fn inc_selected_variable(
		state: web::Data<HttpState>,
		path: web::Path<u32>
	) -> impl Responder {
		let delta = path.into_inner();
		let (tx,rx) = std::sync::mpsc::channel();
		match state.http_sender.send( Message::IncrementSelectedVariable( tx, delta.try_into().unwrap() ) ) {
			_ => {},
		};
		handle_response( rx )
	}
	async fn dec_selected_variable(
		state: web::Data<HttpState>,
		path: web::Path<u32>
	) -> impl Responder {
		let delta = path.into_inner();
		let v: i32 = delta.try_into().unwrap();		
		let (tx,rx) = std::sync::mpsc::channel();
		match state.http_sender.send( Message::IncrementSelectedVariable( tx, -v ) ) {
			_ => {},
		};
		handle_response( rx )
	}

	async fn show_by_name(
		state: web::Data<HttpState>,		
		path: web::Path< String >
	) -> impl Responder {
		let name = path.into_inner();

		match state.http_sender.send( Message::SetElementVisibilityByName( name.clone(), true ) ) {
			_ => {},
		};
		format!("show ({}) name == {}", &state.id, &name)
	}

	async fn hide_by_name(
		state: web::Data<HttpState>,		
		path: web::Path< String >
	) -> impl Responder {
		let name = path.into_inner();

		match state.http_sender.send( Message::SetElementVisibilityByName( name.clone(), false ) ) {
			_ => {},
		};
		format!("hide ({}) name == {}", &state.id, &name)
	}

	async fn list_element_instances(
		state: web::Data<HttpState>,
	) -> impl Responder {
		let (sender, receiver) = mpsc::channel();

		match state.http_sender.send( Message::ListElementInstances( sender ) ) {
			Ok(_) => {
				match receiver.recv() {
					Ok( msg ) => {
						match msg {
							Response::ElementInstanceList( l ) => {
								return l;
							}
							_ => {
								dbg!(&msg);
							}
						}
					},
					Err( e ) => {
						dbg!( &e );
					}
				}
				//format!("elements ->")
				"elements ->".to_string()
			},
			_ => {
				//format!("{{}}")
				"{}".to_string()
			},
		}
	}

	async fn goto_next_page(
		state: web::Data<HttpState>,
	) -> impl Responder {
		let (sender, receiver) = mpsc::channel();
		match state.http_sender.send( Message::GotoNextPage( sender ) ) {
			Ok( _ ) => {
				match receiver.recv() {
					Ok( msg ) => {
						match msg {
							Response::PageChanged( new_page_no, old_page_no ) => {
								return format!("{}, {}", new_page_no.unwrap_or( usize::MAX ), old_page_no.unwrap_or( usize::MAX ) );
							},
							_ => {
								dbg!(&msg);
							}
						}
					},
					Err( e ) => {
						dbg!( &e );
					}
				}
			}
			_ => {},
		};

		"{}".to_string()
	}

	async fn goto_prev_page(
		state: web::Data<HttpState>,
	) -> impl Responder {
		let (sender, receiver) = mpsc::channel();
		match state.http_sender.send( Message::GotoPrevPage( sender ) ) {
			Ok( _ ) => {
				match receiver.recv() {
					Ok( msg ) => {
						match msg {
							Response::PageChanged( new_page_no, old_page_no ) => {
								return format!("{}, {}", new_page_no.unwrap_or( usize::MAX ), old_page_no.unwrap_or( usize::MAX ) );
							},
							_ => {
								dbg!(&msg);
							}
						}
					},
					Err( e ) => {
						dbg!( &e );
					}
				}
			}
			_ => {},
		};

		"{}".to_string()
	}

	async fn goto_page_number(
		state: web::Data<HttpState>,
		path: web::Path< usize >
	) -> impl Responder {
		let page_no = path.into_inner();
		let (sender, receiver) = mpsc::channel();
		match state.http_sender.send( Message::GotoPage( sender, page_no ) ) {
			Ok( _ ) => {
				match receiver.recv() {
					Ok( msg ) => {
						match msg {
							Response::PageChanged( new_page_no, old_page_no ) => {
								return format!("{}, {}", new_page_no.unwrap_or( usize::MAX ), old_page_no.unwrap_or( usize::MAX ) );
							},
							_ => {
								dbg!(&msg);
							}
						}
					},
					Err( e ) => {
						dbg!( &e );
					}
				}
			}
			_ => {},
		};

		"{}".to_string()
	}

impl Cheval {
	pub fn new() -> Self {
		let mut file_cache = std::sync::Arc::new( std::sync::Mutex::new( FileCache::new() ) );
		{	// :TODO: remove once ImageSequence is fully implemented
			let mut fc = file_cache.lock().unwrap();
			fc.enable_block_on_initial_load();
		}
		let mut context = Context::new();
		context.set_file_cache(file_cache.clone());
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
			server_thread: None,
			file_cache,
		}
	}

	pub fn done( &self ) -> bool {
		self.done
	}

	pub fn enable_http( &mut self ) {
		self.http_enabled = true;
	}

	async fn load_elements_for_page( &self, page: &mut Page, config_page_elements: &Vec< ConfigElement > ) -> anyhow::Result<()> {
		for e in config_page_elements {
			if e.disabled {
				continue;
			};
			let mut element: Box< dyn Element + Send > = match e.the_type.as_ref() {
				"block"			=> Box::new( BlockElementFactory::create() ) as Box<dyn Element + Send>,
				"timer"			=> Box::new( TimerElementFactory::create() ) as Box<dyn Element + Send>,
				"lissajous" 	=> Box::new( LissajousElementFactory::create() ),
				"image"			=> Box::new( ImageElementFactory::create() ),
				"text"			=> Box::new( TextElementFactory::create() ),
				"scrolltext"	=> Box::new( ScrollTextElementFactory::create() ),
				"soundbank"		=> Box::new( SoundbankElementFactory::create() ),
//				_ => panic!("Unsupported element type {}", e.the_type ),
				_ => {
					println!("Skipping unsupported element type {}", e.the_type);
					continue
				},
			};
			
			element.set_name( &e.name );

			let mut element_config = ElementConfig::new( &self.config_path.as_path() );

			for p in &e.parameters {
				element_config.set( &p.0, &p.1 );
			}

			dbg!(&element_config);

			element.configure( &element_config );

			let mut element_instance = ElementInstance::new( element );
			if e.visible {
				element_instance.show();
			} else {
				element_instance.hide();
			};
			
			page.add_element_instance( element_instance );
		}

		page.run().await?;
		/*
		for p in self.pages.iter_mut() {
			p.run().await?;
		}
		*/

		Ok(())
	}

	pub async fn load( &mut self, config_file_name: &str ) -> Result<(), Box< dyn std::error::Error > > {
		dbg!(&config_file_name);
		// make path absolute
		let cwd = std::env::current_dir().unwrap();
		let config_file_name = Path::new( &config_file_name );
		let config_file_name = cwd.join( config_file_name );
		let config_file_name = match config_file_name.canonicalize() {
			Ok( c ) => c,
			Err( e ) =>panic!( "Error canonicalizing config file {:?} -> {:?}", &config_file_name, &e ),
		};

		let config_file_name = if config_file_name.is_dir() {
			let mut cfn = config_file_name.clone();
			cfn.push( "config.yaml" );
			if cfn.is_file() {
				cfn
			} else {
				let mut glob = config_file_name.clone();
				glob.push("*config.yaml");
				let glob = glob.to_string_lossy().to_string();
				let mut result = glob::glob( &glob ).expect("Failed to read glob pattern");

				// :TODO: check if something like take(2).to_vec() result in more readable code

				match result.next() {
					None => todo!("no config found"),
					Some( fr ) => {
						match result.next() {
							None => fr.unwrap(),
							Some( _ ) => todo!("More than one config found")
						}
					},
				}
			}
		} else {
			config_file_name
		};

		if let Some( config_path ) = config_file_name.parent() {
			self.config_path = PathBuf::from( &config_path );

			// update file cache base
			{
				let mut fc = self.file_cache.lock().unwrap();
				fc.set_base_path( &self.config_path );

				fc.set_mode( crate::file_cache::FileCacheMode::Watch );
			}
		};


		let cf = match std::fs::File::open( &config_file_name ) {
			Ok( f ) => f,
			Err( e ) => panic!("Error opening config file {:?} -> {:?}", &config_file_name, &e ),
		};

		dbg!(&cf);

		let config: Config = match serde_yaml::from_reader( &cf ) {
			Ok( c ) => c,
			Err( e ) => panic!("Error deserializing config file {:?} -> {:?}", &cf, &e ),
		};

//		dbg!(&config);

		if let Some( variable_filename ) = &config.variable_filename {
			self.variable_filename = Some( variable_filename.clone() );
			match self.context.get_mut_machine().load_variable_storage( &variable_filename ) {
				Ok( _ ) => {},
				Err( _ ) => {

				},
//				r => todo!("{:?}", r),
			};

			println!( "Loaded variables from {}", &variable_filename );
			dbg!( self.context.get_mut_machine() );
		}

		// :HACK: load variable default
		if let Some( defaults ) = &config.variable_defaults {
			for (key, val) in defaults.iter() {
				dbg!(&key, &val);
				let vs = self.context.get_mut_machine().get_mut_variable_storage();
				if vs.get( &key ).is_none() {
					println!("Variable {} not found using default {}", &key, &val );
					// :TODO: handle more variable types
					if let Ok( v ) = val.parse::<i32>() {
						vs.set( &key, expresso::variables::Variable::I32( v ) );
					} else if let Ok( v ) = val.parse::<f32>() {
						vs.set( &key, expresso::variables::Variable::F32( v ) );
					} else {
						vs.set( &key, expresso::variables::Variable::String( val.clone() ) );
					}
				};
			};
		};

		// :HACK:
		{
		let function_table = self.context.get_mut_machine().get_mut_function_table();

		function_table.register(
			"sin",
			|_argc, variable_stack, _variable_storage| {
				// :TODO: handle wrong argc

				let fv = variable_stack.pop_as_f32();

				let r = fv.sin();

				variable_stack.push( expresso::variables::Variable::F32( r ) );
				true
			}
		);

		function_table.register(
			"printHHMMSS",
			|argc, variable_stack, _variable_storage| {
				if argc == 1 {
					let f = variable_stack.pop_as_f32();
					let duration = std::time::Duration::new( f as u64, 0);

					variable_stack.push( expresso::variables::Variable::String( duration.hhmmss() ) );
 					true
				} else {
					false
				}
			}
		);


		let file_cache = self.file_cache.clone();
		function_table.register(
			"text_from_file",
			move |argc, variable_stack, _variable_storage| {
				if argc == 1 {
					let filename = variable_stack.pop_as_string();
					// :TODO: check stack validity here?!

					let mut lock = file_cache.try_lock();
				    if let Ok(ref mut file_cache) = lock {
						match file_cache.load_string( &filename ) {
							Ok((v,s)) => {
								dbg!(&s);
								variable_stack.push( expresso::variables::Variable::String( s ) );
								true
							},
							Err( e ) => {
								let s = format!( "Error: {:?}", &e );
								variable_stack.push( expresso::variables::Variable::String( s ) );
								false
							}
						}
				    } else {
				        println!("try_lock failed");
				        false
				    }

				} else {
					false
				}
			}
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
						match file_cache.load_string( &filename ) {
							Ok((v,s)) => {
								let s = s.split("\n").skip( skip as usize );
								let s = if count > 0 {
									s.take( count as usize ).collect::<Vec<&str>>()
								} else { // zero -> take everything
									s.collect::<Vec<&str>>()
								};

								let s = s.join("\n");
								variable_stack.push( expresso::variables::Variable::String( s ) );
								true
							},
							Err( e ) => {
								let s = format!( "Error: {:?}", &e );
								variable_stack.push( expresso::variables::Variable::String( s ) );
								false
							}
						}
				    } else {
				        println!("try_lock failed");
				        false
				    }

				} else {
					false
				}
			}
		);
		}

		self.file_cache.lock().unwrap().run().await?;

		// -- :HACK:		

		if let Some( default_page ) = &config.default_page {
			self.active_page = *default_page;
		}

		dbg!(&config);
		if let Some( elements ) = &config.elements {
			let mut page = Page::new();	// global/top page

			self.load_elements_for_page( &mut page, &elements ).await?;

			page.show();
			self.page = Some( page );
		}

		// :TODO: allow start page to be configured

		if let Some( pages ) = &config.pages {
			for active_page_config in pages {
				let mut page = Page::new();	// sub page

				if let Some( parameters ) = &active_page_config.parameters {
					let mut page_config = ElementConfig::new( &self.config_path.as_path() );

					for p in parameters.iter() {
						page_config.set( &p.0, &p.1 );
					}

					dbg!(&page_config);

					page.configure( &page_config );

					dbg!(&page);

//					todo!("die");
				}

				self.load_elements_for_page( &mut page, &active_page_config.elements ).await?;

				dbg!(self.pages.len(), &self.active_page);
				if self.pages.len() == self.active_page {
					page.show();
				}
				self.pages.push( page );

			}
		} 
		println!("Running...");
		Ok(())
	}

	fn goto_page( &mut self, page_no: usize ) -> ( Option< usize >, Option< usize > ) {
		let mut old_page_no = None;
		let mut new_page_no = None;
		if self.active_page != page_no {
			if let Some( old_page ) = self.pages.get_mut( self.active_page ) {
				old_page_no = Some( self.active_page );
				old_page.hide();
			}
			self.active_page = page_no;
			if let Some( page ) = self.pages.get_mut( page_no ) {
				new_page_no = Some( page_no );
				page.show();
			}
		}

		let cheval_active_page_number = format!("{}", self.active_page);
		self.context.set_string( "cheval_active_page_number", &cheval_active_page_number.to_string() );

		( new_page_no, old_page_no )
	}

	fn goto_next_page( &mut self ) -> ( Option< usize >, Option< usize > ) {
		let page_no = ( self.active_page + 1 ) % self.pages.len();
		self.goto_page( page_no )
	}

	fn goto_prev_page( &mut self ) -> ( Option< usize >, Option< usize > ) {
		let page_no = if self.active_page > 0 {
			self.active_page - 1
		} else {
			self.pages.len() - 1
		};
		self.goto_page( page_no )
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
		func: Box< dyn Fn( &mut ElementInstance ) >,
	) {
		if let Some( p ) = &mut self.page {
			p.run_for_element_instance_with_name( name, &func );			
		}

		for p in &mut self.pages {
			p.run_for_element_instance_with_name( name, &func );
		}
	}

	pub fn initialize( &mut self ) -> anyhow::Result<()> {
		if self.http_enabled {
//			let (tx, rx) = mpsc::channel();

			let (tx2, rx2) = mpsc::channel();

			self.http_receiver = Some( rx2 );


			let server = HttpServer::new(move || {
				let http_state = HttpState {
					id: "default".to_string(),
					http_sender: tx2.clone(),
				};
				let http_state = Data::new( http_state );
								App::new()
//									.data( http_state )
									.app_data( http_state )
									.route("/selectNextVariable", web::get().to(select_next_variable))
									.route("/selectNextVariableWithPrefix/{prefix}", web::get().to(select_next_variable_with_prefix))
									.route("/incSelectedVariable/{value}", web::get().to(inc_selected_variable))
									.route("/decSelectedVariable/{value}", web::get().to(dec_selected_variable))
									.route("/setVariable/{name}/{value}", web::get().to(set_variable))
									.route("/incVariable/{name}/{delta}", web::get().to(inc_variable))
									.route("/decVariable/{name}/{delta}", web::get().to(dec_variable))
									.route("/show/name/{name}", web::get().to(show_by_name))
									.route("/hide/name/{name}", web::get().to(hide_by_name))
									// :TODO: implement list_pages
									// :TODO: list element instances for specific page
									.route("/list_element_instances", web::get().to(list_element_instances))
									.route("/page/next", web::get().to(goto_next_page))
									.route("/page/prev", web::get().to(goto_prev_page))
									.route("/page/number/{number}", web::get().to(goto_page_number))
							})
							.bind("0.0.0.0:8080")?
							.run();
			let server_thread = std::thread::spawn(move || {
				let sys = System::new(/*"test"*/);

//				let _ = tx.send( server.clone() );

				match sys.block_on( server ) {
					// :TODO: handle errors
					_ => {},
				}
    		});//.join().expect("Thread panicked");

			self.server_thread = Some( server_thread );
    		dbg!(&self.http_enabled);
/*
    		let server = rx.recv().unwrap();
    		self.http_server = Some( server );
*/
//    		dbg!(&self.http_server);

			// :TODO: cleanup server on shutdown
		}
		Ok(())
	}

	pub fn update( &mut self ) {
		// :TODO: create a date time element that provides info
		let now: DateTime<Utc> = Utc::now();
		let clock_string = now.format("%H:%M:%S");
		self.context.set_string( "clock_string", &clock_string.to_string() );

		let frametime_duration = now.signed_duration_since( self.last_update_time );
		let frametime = frametime_duration.num_milliseconds() as f64;
		let frametime_string = format!("{}", frametime );
		self.context.set_string( "frametime_string", &frametime_string );
		self.last_update_time = now;
		self.context.set_time_step( frametime/1000.0 );


		let time_since_start = now.signed_duration_since( self.start_time );
		let time_since_start = time_since_start.num_milliseconds() as f64 / 1000.0;

		self.context.get_mut_machine().get_mut_variable_storage().set( "time", expresso::variables::Variable::F32( time_since_start as f32 ) );

		if let Some( p ) = &mut self.page {
			p.update( &mut self.context );
		}
		for p in &mut self.pages {
			p.update( &mut self.context );
		}

		let ts = self.context.time_step();
		let soundbank = &mut self.context.get_soundbank_mut();
		soundbank.update( ts );

		match self.file_cache.try_lock() {
			Ok( ref mut file_cache ) => {
				file_cache.update();
			},
			_ => {},
//			if let Ok(ref mut file_cache) = lock {

		}

		if let Some( http_receiver ) = &self.http_receiver {
			match http_receiver.try_recv() {
				Ok( msg ) => {
//					dbg!("http_receiver got message", &msg);
					match msg {
						Message::SelectNextVariable( result_sender, maybe_prefix ) => {
							let name = self.context.select_next_variable( maybe_prefix.as_ref().map(String::as_str) );
							match result_sender.send( Response::VariableSelected( name.to_string() ) ) {
								_ => {},
							};
						}
						Message::SetVariable( result_sender, name, value ) => {
							dbg!( "set variable", &name, &value );
							if let Ok( v ) = value.parse::<u32>() {
								self.context.set_f32( &name, v as f32 );
								match result_sender.send( Response::VariableU32Changed( name.clone(), v) ) {
									_ => {},
								};
							} else if let Ok( v ) = value.parse::<f32>() {
								self.context.set_f32( &name, v );
								match result_sender.send( Response::VariableF32Changed( name.clone(), v) ) {
									_ => {},
								};
							} else  {
								self.context.set_string( &name, &value );
								match result_sender.send( Response::VariableStringChanged( name.clone(), value.clone()) ) {
									_ => {},
								};
							};
							dbg!(&self.context);
						}
						Message::IncrementVariable( result_sender, name, delta ) => {
							dbg!( "inc variable", &name, delta);
							if let Some( old ) = self.context.get_f32( &name ) {
								let new = old + delta as f32;
								self.context.set_f32( &name, new );
								match result_sender.send( Response::VariableChanged( name.clone(), new) ) {
									_ => {},
								};
							}

							dbg!(&self.context);
						}
						Message::IncrementSelectedVariable( result_sender, delta ) => {
							let name = self.context.selected_variable().to_string();
							dbg!( "inc selected variable", &name, delta);
							if let Some( old ) = self.context.get_f32( &name ) {
								let new = old + delta as f32;
								self.context.set_f32( &name, new );
								match result_sender.send( Response::VariableChanged( name, new) ) {
									_ => {},
								};
							}

							dbg!(&self.context);
						}
						Message::SetElementVisibilityByName( name, visible ) => {
							dbg!( "set visibility", &name, &visible );
							self.run_for_element_instance_with_name(
								&name,
								Box::new( move |element_instance| {
									dbg!("Found element_instance", &element_instance);
									if visible {
										element_instance.show();
									} else {
										element_instance.hide();										
									}
								} ),
							);
						}
						Message::ListElementInstances( sender ) => {
							match sender.send( Response::ElementInstanceList( "{\":TODO\": false}".to_string() ) ) {
								_ => {},
							};
						},
						Message::GotoNextPage( sender ) => {
							let ( new_page_no, old_page_no ) = self.goto_next_page();
							match sender.send( Response::PageChanged( new_page_no, old_page_no ) ) {
								_ => {},
							};
						},
						Message::GotoPrevPage( sender ) => {
							let ( new_page_no, old_page_no ) = self.goto_prev_page();
							match sender.send( Response::PageChanged( new_page_no, old_page_no ) ) {
								_ => {},
							};
						},
						Message::GotoPage( sender, page_no ) => {
							let ( new_page_no, old_page_no ) = self.goto_page( page_no );
							match sender.send( Response::PageChanged( new_page_no, old_page_no ) ) {
								_ => {},
							};
						},
						x => {
							dbg!("unhandled", &x);
						},
					}
				}
				Err( e ) => {
					match e {
						mpsc::TryRecvError::Empty => {
							// empty is fine
						},
						mpsc::TryRecvError::Disconnected => {
							// disconnected is also fine, for now
						},
					}
				}
			}
		}
	}

	pub fn render( &mut self, render_buffer: &mut RenderBuffer ) {
/*		
		for e in &self.elements {
//			dbg!(e);
			e.render( &mut render_buffer.buffer, render_buffer.width, render_buffer.height );
		};
*/		

//		let mut render_context = RenderContext::new();
		if let Some( p ) = &mut self.page {
			p.render( render_buffer, &mut self.render_context );
		}
		for p in &self.pages {
//			dbg!(e);
//			if e.is_visible() {
				p.render( render_buffer, &mut self.render_context );
//			}
		};
	}

	pub fn add_key( &mut self, key: u32 ) {
		// :TODO: add keys to queue, and handle in update
		match key {
			27 => {
				self.done = true;
			},
			63234 => {	// Cursor Left
				self.goto_prev_page();
			},
			63235 => {	// Cursor Right
				self.goto_next_page();
			},
			x if x >=48 && x<=57 => { // 0
				let p = x - 48;
				self.goto_page( p as usize );
			}
			_ => {
				dbg!("Got key", &key);
			}
		}
	}

	pub fn shutdown( &mut self ) -> anyhow::Result<()> {
		for p in self.pages.iter_mut() {
			p.shutdown();
		}
		if let Some( p ) = &mut self.page {
			p.shutdown();
		}
		if let Some( variable_filename ) = &self.variable_filename {
			match self.context.get_mut_machine().save_variable_storage( &variable_filename ) {
				Ok( _ ) => {},
				Err( e ) => return Err(anyhow::anyhow!("Error saving variables: {:?}", e )),
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
