use std::collections::HashMap;
use regex::Regex;

use crate::variable::Variable;

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

	// :TODO: maybe return str instead String to avoid potentially unneeded copies
	pub fn expand_string_or( &mut self, s: &str, default: &str ) -> String {
		let re = Regex::new(r"^\$\{([^:]+)(:(.+))?\}$").unwrap();	// :TODO: we could use non greedy matching here
		if let Some( caps ) = re.captures( &s ) {
//			dbg!(&caps);
			let name = &caps[ 1 ];
//			dbg!(&name);
			if let Some( value ) = self.get_string( &name ) {
				value.to_string()
			} else {
//				dbg!("Variable not found", &name);
//				dbg!("Returning default for", &s, &default);
				match caps.get( 3 ) {
					Some( c ) => {
						self.set_string( &name, c.as_str() );
						c.as_str().to_string()
					},
					None => {
						default.to_string()
					},
				}
			}
		} else {
			s.to_string()
		}
	}

	pub fn expand_u32_or( &mut self, s: &str, default: u32 ) -> u32 {
		let s = self.expand_string_or( s, "" );
		if let Ok( u ) = s.parse::<u32>() {
			u
		} else {
			default
		}
	}

	pub fn 	expand_var_to_u32_or( &mut self, v: &Variable, default: u32 ) -> u32 {
		match v {
			Variable::U32( u ) => {
				*u
			},
			Variable::STRING( s ) => {
				let s = self.expand_string_or( s, "" );
				if let Ok( u ) = s.parse::<u32>() {
					u
				} else if let Ok( f ) = s.parse::<f32>() {
					f as u32
				} else {
					default
				}
			},
			_ => default,
		}
	}

	pub fn 	expand_var_to_f32_or( &mut self, v: &Variable, default: f32 ) -> f32 {
		match v {
			Variable::F32( u ) => {
				*u
			},
			Variable::U32( u ) => {
				*u as f32
			},
			Variable::STRING( s ) => {
				let s = self.expand_string_or( s, "" );
				if let Ok( u ) = s.parse::<u32>() {
					u as f32
				} else if let Ok( f ) = s.parse::<f32>() {
					f
				} else {
					default
				}
			},
			_ => default,
		}
	}

}
