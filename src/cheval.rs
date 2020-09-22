use std::collections::HashMap;

use serde::Deserialize;
use serde_yaml;

use crate::block_element::BlockElementFactory;
use crate::lissajous_element::LissajousElementFactory;
use crate::loadtext_element::LoadTextElementFactory;
use crate::image_element::ImageElementFactory;
use crate::text_element::TextElementFactory;
use crate::element::{Element,ElementConfig};
use crate::context::Context;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;

use chrono::{DateTime, Utc};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Cheval {
	elements: Vec< Box< dyn Element > >,
	context: Context,
	last_update_time: DateTime<Utc>,
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

impl Cheval {
	pub fn new() -> Self {
		Self {
			elements: Vec::new(),
			context: Context::new(),
			last_update_time: Utc::now(),
		}
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
		for e in &mut self.elements {
			e.update( &mut self.context );
		}	
	}

	pub fn render( &self, render_buffer: &mut RenderBuffer ) {
/*		
		for e in &self.elements {
//			dbg!(e);
			e.render( &mut render_buffer.buffer, render_buffer.width, render_buffer.height );
		};
*/		

		let mut render_context = RenderContext::new();
		for e in &self.elements {
//			dbg!(e);
			e.render( render_buffer, &mut render_context );
		};
	}
	pub fn shutdown( &mut self ) {
		for e in self.elements.iter_mut() {
			e.shutdown();
		}
	}
}
