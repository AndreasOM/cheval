use std::collections::HashMap;
use regex::Regex;

use expresso::expression::Expression;
use expresso::machine::Machine;

use oml_audio::SoundBank;

#[derive(Debug)]
pub struct Context {
	time_step: f64,
	soundbank: SoundBank,
	machine: Machine,
}

impl Context {
	pub fn new() -> Self {
		Self {
			time_step: 1.0/60.0,
			soundbank: SoundBank::new(),
			machine: Machine::new(),
		}
	}

	pub fn play_sound( &mut self, id: &str ) {
		println!("play_sound: {:?}", id);
		self.soundbank.enable_debug();
		self.soundbank.play( id );
	}

	pub fn get_soundbank_mut( &mut self ) -> &mut SoundBank {
		&mut self.soundbank
	}
	pub fn get_mut_machine( &mut self ) -> &mut Machine {
		&mut self.machine
	}

	pub fn set_time_step( &mut self, time_step: f64 ) {
		self.time_step = time_step;
	}

	pub fn time_step( &self ) -> f64 {
		self.time_step
	}

	pub fn set_string( &mut self, name: &str, value: &str ) {
//		dbg!(&name, &value);
		self.machine.get_mut_variable_storage().set( name, expresso::variables::Variable::String( value.to_string() ) );
	}

	pub fn set_f32( &mut self, name: &str, value: f32 ) {
//		dbg!(&name, &value);
		self.machine.get_mut_variable_storage().set( name, expresso::variables::Variable::F32( value ) );
	}

	pub fn get_string( &self, name: &str ) -> Option< &str > {
		match self.machine.get_variable_storage().get( name ) {
			Some( expresso::variables::Variable::String( s ) ) => Some( s ),
			o => todo!("{:?}", &o),
		}
	}

	pub fn get_f32( &self, name: &str ) -> Option< f32 > {
		match self.machine.get_variable_storage().get( name ) {
			Some( expresso::variables::Variable::F32( f ) ) => Some( *f ),
			None => None,	// Why not???
			o => {
//				todo!("{:?}", &o)
				println!("Error: Can not get as f32: {:?} using 0.0", &o);
				Some( 0.0 )
			},
		}
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
		let re2 = Regex::new(r"^\$\[([^:]+)(:(.+))?\]$").unwrap();	// :TODO: we could use non greedy matching here
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
		} else if let Some( caps ) = re2.captures( &s ) {
			let mut expression = Expression::new();
			expression.from_str( &caps[ 1 ] );
//			println!("{}", expression);
			let mut r = expression.run( &mut self.machine );
			match r.pop() {
				Some( expresso::variables::Variable::I32( i ) ) => {
					format!("{}", i )
				},
				Some( expresso::variables::Variable::F32( f ) ) => {
					format!("{}", f )
				},
				None => format!("No result" ),
				r => todo!("Result is not printable {:?}", r ),
			}

//			format!("Expression: {}", &caps[ 1 ])
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
/*
	pub fn 	expand_var_to_u32_or( &mut self, v: &Variable, default: u32 ) -> u32 {
		match v.original() {
			Original::U32( u ) => {
				u
			},
			Original::STRING( s ) => {
				let s = self.expand_string_or( &s, "" );
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
		match v.original() {
			Original::F32( u ) => {
				u
			},
			Original::U32( u ) => {
				u as f32
			},
			Original::STRING( s ) => {
				let s = self.expand_string_or( &s, "" );
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
*/	

}
