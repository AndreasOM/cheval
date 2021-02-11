use std::collections::HashMap;

use serde::Deserialize;
use serde_yaml;

use crate::element_instance::ElementInstance;

use crate::block_element::BlockElementFactory;
use crate::lissajous_element::LissajousElementFactory;
use crate::loadtext_element::LoadTextElementFactory;
use crate::image_element::ImageElementFactory;
use crate::scrolltext_element::ScrollTextElementFactory;
use crate::text_element::TextElementFactory;
use crate::element::{Element,ElementConfig};
use crate::context::Context;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;
use crate::timer_element::TimerElementFactory;
use crate::page::Page;

use chrono::{DateTime, Utc};
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
}

#[derive(Debug)]
enum Response {
	None,
	NotImplemented( String ),
	ElementInstanceList( String ),
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
	context: Context,
	last_update_time: DateTime<Utc>,
	render_context: RenderContext,
	http_enabled: bool,
	http_server: Option< actix_web::dev::Server >,
	http_receiver: Option< mpsc::Receiver< Message > >,
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
			context: Context::new(),
			last_update_time: Utc::now(),
			render_context: RenderContext::new(),
			http_enabled: false,
			http_server: None,
			http_receiver: None,
		}
	}

	pub fn enable_http( &mut self ) {
		self.http_enabled = true;
	}

	async fn load_elements_for_page( page: &mut Page, config_page_elements: &Vec< ConfigElement > ) -> anyhow::Result<()> {
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
//				_ => panic!("Unsupported element type {}", e.the_type ),
				_ => {
					println!("Skipping unsupported element type {}", e.the_type);
					continue
				},
			};
			
			element.set_name( &e.name );

			let mut element_config = ElementConfig::new();

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
		let cf = std::fs::File::open( config_file_name )?;

		let config: Config = serde_yaml::from_reader( cf )?;

		if let Some( default_page ) = &config.default_page {
			self.active_page = *default_page;
		}

		dbg!(&config);
		if let Some( elements ) = &config.elements {
			let mut page = Page::new();	// global/top page

			Cheval::load_elements_for_page( &mut page, &elements ).await?;

			page.show();
			self.page = Some( page );
		}

		// :TODO: allow start page to be configured

		if let Some( pages ) = &config.pages {
			for active_page_config in pages {
				let mut page = Page::new();	// sub page

				Cheval::load_elements_for_page( &mut page, &active_page_config.elements ).await?;

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
		if let Some( p ) = &mut self.page {
			p.update( &mut self.context );
		}
		for p in &mut self.pages {
			p.update( &mut self.context );
		}

		if let Some( http_receiver ) = &self.http_receiver {
			match http_receiver.try_recv() {
				Ok( msg ) => {
//					dbg!("http_receiver got message", &msg);
					match msg {
						Message::SetVariable( name, value ) => {
							dbg!( "set variable", &name, &value );
							self.context.set_string( &name, &value );
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
	pub fn shutdown( &mut self ) {
		for p in self.pages.iter_mut() {
			p.shutdown();
		}
		if let Some( p ) = &mut self.page {
			p.shutdown();
		}
	}
}
