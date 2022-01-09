use std::collections::HashMap;

use std::path::{ Path, PathBuf };
use glob::Paths;

use serde::Deserialize;
use serde_yaml;

use crate::element_instance::ElementInstance;

use crate::block_element::BlockElementFactory;
use crate::lissajous_element::LissajousElementFactory;
use crate::loadtext_element::LoadTextElementFactory;
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

use chrono::{DateTime, Utc};
use hhmmss::Hhmmss;
use std::sync::mpsc;
use std::cell::RefCell;
use std::rc::Rc;

use actix_web::{web, App, HttpRequest, HttpServer, Responder, rt::System};

#[derive(Debug)]
enum Message {
	None,
	SetVariable( String, String ),
	SetElementVisibilityByName( String, bool ),
	ListElementInstances( mpsc::Sender< Response > ),
	GotoNextPage( mpsc::Sender< Response > ),
	GotoPrevPage( mpsc::Sender< Response > ),
	GotoPage( mpsc::Sender< Response >, usize ),
}

#[derive(Debug)]
enum Response {
	None,
	NotImplemented( String ),
	ElementInstanceList( String ),
	PageChanged( Option< usize >, Option< usize > ),	// new page #, old page #
}


#[derive(Debug)]
struct HttpState {
	id: String,
	http_sender: mpsc::Sender< Message >,
}

#[derive(Debug)]
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
	http_server: Option< actix_web::dev::Server >,
	http_receiver: Option< mpsc::Receiver< Message > >,
	done: bool,
	config_path: PathBuf,
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

#[derive(Debug, Deserialize)]
struct Config {
	default_page: Option< usize >,
	variable_filename: Option< String >,
	variable_defaults: Option< HashMap< String, String > >,
	pages: Option< Vec< ConfigPage > >,
	elements: Option< Vec< ConfigElement > >,
}

//	async fn set_variable( web::Path((name, value)): web::Path<(String, String)>, tx: mpsc::Receiver< Message > ) -> impl Responder {
	async fn set_variable(
		state: web::Data<HttpState>,		
		web::Path((name, value)): web::Path<(String, String)>
	) -> impl Responder {

		
		match state.http_sender.send( Message::SetVariable( name.clone(), value.clone() ) ) {
			_ => {},
		};

//		dbg!(&name, &value);
		format!("setVariable ({}) {} = {}", &state.id, &name, &value)
	}

	async fn show_by_name(
		state: web::Data<HttpState>,		
		web::Path((name)): web::Path<(String)>
	) -> impl Responder {
		match state.http_sender.send( Message::SetElementVisibilityByName( name.clone(), true ) ) {
			_ => {},
		};
		format!("show ({}) name == {}", &state.id, &name)
	}

	async fn hide_by_name(
		state: web::Data<HttpState>,		
		web::Path((name)): web::Path<(String)>
	) -> impl Responder {
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
				format!("elements ->")
			},
			_ => {
				format!("{{}}")
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

		format!("{{}}")
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

		format!("{{}}")
	}

	async fn goto_page_number(
		state: web::Data<HttpState>,
		web::Path((page_no)): web::Path<(usize)>
	) -> impl Responder {
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

		format!("{{}}")
	}

	async fn greet(req: HttpRequest) -> impl Responder {
	    let name = req.match_info().get("name").unwrap_or("World");
	    format!("Hello {}!", &name)
	}

impl Cheval {
	pub fn new() -> Self {
		Self {
//			element_instances: Vec::new(),
			page: None,
			pages: Vec::new(),
			active_page: 1,
			variable_filename: None,
			context: Context::new(),
			last_update_time: Utc::now(),
			start_time: Utc::now(),
			render_context: RenderContext::new(),
			http_enabled: false,
			http_server: None,
			http_receiver: None,
			done: false,
			config_path: PathBuf::new(),
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
				"loadtext"		=> Box::new( LoadTextElementFactory::create() ) as Box<dyn Element + Send>,
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
		let function_table = self.context.get_mut_machine().get_mut_function_table();

		function_table.register(
			"sin",
			|argc, variable_stack, _variable_storage| {
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
			let (tx, rx) = mpsc::channel();

			let (tx2, rx2) = mpsc::channel();

			self.http_receiver = Some( rx2 );


			let server = HttpServer::new(move || {
				let http_state = HttpState {
					id: "default".to_string(),
					http_sender: tx2.clone(),
				};
								App::new()
									.data( http_state )
									.route("/setVariable/{name}/{value}", web::get().to(set_variable))
									.route("/show/name/{name}", web::get().to(show_by_name))
									.route("/hide/name/{name}", web::get().to(hide_by_name))
									// :TODO: implement list_pages
									// :TODO: list element instances for specific page
									.route("/list_element_instances", web::get().to(list_element_instances))
									.route("/page/next", web::get().to(goto_next_page))
									.route("/page/prev", web::get().to(goto_prev_page))
									.route("/page/number/{number}", web::get().to(goto_page_number))
//									.route("/", web::get().to(greet))
							})
							.bind("0.0.0.0:8080")?
							.run();
			std::thread::spawn(move || {
				let mut sys = System::new("test");

				let _ = tx.send( server.clone() );

				sys.block_on( server );
    		});//.join().expect("Thread panicked");
    		dbg!(&self.http_enabled);

    		let server = rx.recv().unwrap();
    		self.http_server = Some( server );

    		dbg!(&self.http_server);

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
		if let soundbank = &mut self.context.get_soundbank_mut() {
			soundbank.update( ts );
		}

		if let Some( http_receiver ) = &self.http_receiver {
			match http_receiver.try_recv() {
				Ok( msg ) => {
//					dbg!("http_receiver got message", &msg);
					match msg {
						Message::SetVariable( name, value ) => {
							dbg!( "set variable", &name, &value );
							if let Ok( v ) = value.parse::<u32>() {
								self.context.set_f32( &name, v as f32 );
							} else if let Ok( v ) = value.parse::<f32>() {
								self.context.set_f32( &name, v );
							} else  {
								self.context.set_string( &name, &value );
							};
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
				Empty => {

				},
				_ => {},
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

	pub fn shutdown( &mut self ) {
		for p in self.pages.iter_mut() {
			p.shutdown();
		}
		if let Some( p ) = &mut self.page {
			p.shutdown();
		}
		if let Some( variable_filename ) = &self.variable_filename {
			self.context.get_mut_machine().save_variable_storage( &variable_filename );
		}		
	}
}
