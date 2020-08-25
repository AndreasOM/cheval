use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_yaml;

use crate::block_element::BlockElementFactory;
use crate::lissajous_element::LissajousElementFactory;
use crate::image_element::ImageElementFactory;
use crate::element::{Element,ElementConfig};

#[derive(Debug)]
pub struct Cheval {
	elements: Vec< Box< dyn Element > >,
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
		}
	}

	pub fn load( &mut self, config_file_name: &str ) -> Result<(), Box< dyn std::error::Error > > {
		let cf = std::fs::File::open( config_file_name )?;

		let config: Config = serde_yaml::from_reader( cf )?;

		dbg!(&config);
		for e in config.elements {
			if e.disabled {
				continue;
			};
			let mut element: Box< dyn Element > = match e.the_type.as_ref() {
				"block" => Box::new( BlockElementFactory::create() ),
				"lissajous" => Box::new( LissajousElementFactory::create() ),
				"image" => Box::new( ImageElementFactory::create() ),
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

		Ok(())
	}
	pub fn add_element( &mut self, element: Box< dyn Element > ) {
		self.elements.push( element );
	}

	pub fn update( &mut self ) {
		for e in &mut self.elements {
			e.update();
		}	
	}

	pub fn render( &self, buffer: &mut Vec<u32>, width: usize, height: usize ) {
		for e in &self.elements {
//			dbg!(e);
			e.render( buffer, width, height );
		}
	}
}
