use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_yaml;

use crate::block_element::{BlockElement,BlockElementFactory};
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
	parameters: HashMap< String, String >
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
			let mut block_element = BlockElementFactory::create();
			block_element.set_name( &e.name );

			let mut element_config = ElementConfig::new();

			for p in e.parameters {
				element_config.set( &p.0, &p.1 );
			}

			dbg!(&element_config);

			block_element.configure( &element_config );

			self.add_element( Box::new( block_element ) );
		}

		Ok(())
	}
	pub fn add_element( &mut self, element: Box< dyn Element > ) {
		self.elements.push( element );
	}

	pub fn render( &self, buffer: &mut Vec<u32>, width: usize, height: usize ) {
		for e in &self.elements {
//			dbg!(e);
			e.render( buffer, width, height );
		}
	}
}
