use std::collections::HashMap;
use regex::Regex;

#[derive(Debug)]
pub struct Context {
	time_step: f64,
	variables: HashMap<String,String>,
}

impl Context {
	pub fn new() -> Self {
		Self {
			time_step: 1.0/60.0,
			variables: HashMap::new(),
		}
	}

	pub fn set_time_step( &mut self, time_step: f64 ) {
		self.time_step = time_step;
	}

	pub fn time_step( &self ) -> f64 {
		self.time_step
	}

	pub fn set_string( &mut self, name: &str, value: &str ) {
		self.variables.insert( name.to_string(), value.to_string() );
	}

	pub fn get_string( &self, name: &str ) -> Option< &str > {
		self.variables.get( name ).map(|s| s.as_ref() )
	}

	pub fn get_expanded_string( &self, name: &str ) -> Option< &str > {
		match self.get_string( name ) {
			None => None,
			Some( s ) => {
				Some( s )
			},
		}
	}

	pub fn expand_string_or( &self, s: &str, default: &str ) -> String {
		let re = Regex::new(r"^\$\{(.+)\}$").unwrap();
		if let Some( caps ) = re.captures( &s ) {
			let name = &caps[ 1 ];
			if let Some( value ) = self.get_string( &name ) {
				value.to_string()
			} else {
//				dbg!("Variable not found", &name);
//				dbg!("Returning default for", &s, &default);

				default.to_string()
			}
		} else {
			s.to_string()
		}
	}

	pub fn expand_u32_or( &self, s: &str, default: u32 ) -> u32 {
		let s = self.expand_string_or( s, "" );
		if let Ok( u ) = s.parse::<u32>() {
			u
		} else {
			default
		}
	}
}
