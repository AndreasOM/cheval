use std::collections::HashMap;

use serde::Deserialize;
use serde_yaml;

use crate::block_element::BlockElementFactory;
use crate::countdown_element::CountdownElementFactory;
use crate::lissajous_element::LissajousElementFactory;
use crate::loadtext_element::LoadTextElementFactory;
use crate::image_element::ImageElementFactory;
use crate::text_element::TextElementFactory;
use crate::element::{Element,ElementConfig};
use crate::context::Context;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;

use chrono::{DateTime, Utc};
use std::sync::mpsc;
use std::cell::RefCell;
use std::rc::Rc;

use actix_web::{web, App, HttpRequest, HttpServer, Responder, rt::System};

#[derive(Debug)]
enum Message {
	None,
	SetVariable( String, String ),
}

#[derive(Debug)]
struct HttpState {
	id: String,
	http_sender: mpsc::Sender< Message >,
}

#[derive(Debug)]
pub struct Cheval {
	elements: Vec< Box< dyn Element > >,
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
	parameters: HashMap< String, String >
}

fn default_bool_false() -> bool {
    false
}

#[derive(Debug, Deserialize)]
struct Config {
	elements: Vec< ConfigElement >
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

	async fn greet(req: HttpRequest) -> impl Responder {
	    let name = req.match_info().get("name").unwrap_or("World");
	    format!("Hello {}!", &name)
	}

impl Cheval {
	pub fn new() -> Self {
		Self {
			elements: Vec::new(),
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


	pub async fn load( &mut self, config_file_name: &str ) -> Result<(), Box< dyn std::error::Error > > {
		let cf = std::fs::File::open( config_file_name )?;

		let config: Config = serde_yaml::from_reader( cf )?;

		dbg!(&config);
		for e in config.elements {
			if e.disabled {
				continue;
			};
			let mut element: Box< dyn Element + Send > = match e.the_type.as_ref() {
				"block" => Box::new( BlockElementFactory::create() ) as Box<dyn Element + Send>,
				"countdown" => Box::new( CountdownElementFactory::create() ) as Box<dyn Element + Send>,
				"loadtext" => Box::new( LoadTextElementFactory::create() ) as Box<dyn Element + Send>,
				"lissajous" => Box::new( LissajousElementFactory::create() ),
				"image" => Box::new( ImageElementFactory::create() ),
				"text" => Box::new( TextElementFactory::create() ),
//				_ => panic!("Unsupported element type {}", e.the_type ),
				_ => {
					println!("Skipping unsupported element type {}", e.the_type);
					continue
				},
			};
			
			element.set_name( &e.name );

			let mut element_config = ElementConfig::new();

			for p in e.parameters {
				element_config.set( &p.0, &p.1 );
			}

			dbg!(&element_config);

			element.configure( &element_config );

			self.add_element( element );
		}

		for e in self.elements.iter_mut() {
			e.run().await?;
		}

		println!("Running...");
		Ok(())
	}
	pub fn add_element( &mut self, element: Box< dyn Element > ) {
		self.elements.push( element );
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
									.route("/", web::get().to(greet))
									.route("/{name}", web::get().to(greet))
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
		for e in &mut self.elements {
			e.update( &mut self.context );
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
						_ => {},
					}
				}
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
		for e in &self.elements {
//			dbg!(e);
			e.render( render_buffer, &mut self.render_context );
		};
	}
	pub fn shutdown( &mut self ) {
		for e in self.elements.iter_mut() {
			e.shutdown();
		}
	}
}
