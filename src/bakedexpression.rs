use crate::context::Context;

use expresso::expression::Expression;
use expresso::variables::Variable;

#[derive(Debug)]
pub struct BakedExpression {
	original: String,
	expression: Option< Expression >,
	baked: Baked,
}

#[derive(Debug)]
pub enum Baked {
	EMPTY,
	F32(f32),
	U32(u32),
	STRING(String),
}

impl BakedExpression {
	pub fn new() -> Self {
		Self {
			original: String::new(),
			expression: None,
			baked: Baked::EMPTY,
		}
	}


	pub fn from_str( v: &str ) -> Self {
		let mut expression = Expression::new();
		expression.enable_upgrade_of_literals_to_strings();
		expression.from_str( v );

		Self {
			original: v.to_string(),
			expression: Some( expression ),
			baked: Baked::EMPTY,
		}
	}

	pub fn from_f32( v: f32 ) -> Self {
		Self {
			original: String::new(),
			expression: None,
			baked: Baked::F32( v ),
		}
	}

	pub fn from_u32( v: u32 ) -> Self {
		Self {
			original: String::new(),
			expression: None,
			baked: Baked::U32( v ),
		}
	}


	pub fn bake_f32_or( &mut self, context: &mut Context, default: f32 ) {
		if let Some( e ) = &self.expression {
			let r = e.run( context.get_mut_machine() );
			match r.top() {
				Some( Variable::F32( f ) ) => {
					self.baked = Baked::F32( *f );
				},
				Some( Variable::ERROR( e ) ) => {
					println!("Error baking {:?} in {:?}", self, context );
					self.baked = Baked::F32( default );
				},
				t => todo!("Result type not handled {:?} {:?} {:?}", t, r, e ),
			}
		} else {
			match self.baked {
				Baked::F32( _ ) => {}, // just keep the baked value
				_ => self.baked = Baked::F32( default ),
			}
		}
	}

	pub fn bake_u32_or( &mut self, context: &mut Context, default: u32 ) {
		if let Some( e ) = &self.expression {
			let r = e.run( context.get_mut_machine() );
			match r.top() {
				Some( Variable::F32( f ) ) => {
					self.baked = Baked::U32( *f as u32 );
				},
				Some( Variable::I32( i ) ) => {	// :HACK: :TODO: at least add a range check
					self.baked = Baked::U32( *i as u32 );
				},
				Some( Variable::ERROR( e ) ) => {
					// :TODO: make error visible to caller/user
					println!("Error baking {:?} got stack {:?} using default {}", &self, &r, &default );
					self.baked = Baked::U32( default );
				},
				t => {
					dbg!( &self );
					todo!("Result type not handled {:?} {:?} {:?}", t, r, e )
				},
			}
		} else {
			match self.baked {
				Baked::U32( _ ) => {}, // just keep the baked value
				_ => self.baked = Baked::U32( default ),
			}
		}
	}

	pub fn bake_string_or( &mut self, context: &mut Context, default: &str ) {
		if let Some( e ) = &self.expression {
			let r = e.run( context.get_mut_machine() );
			match r.top() {
				Some( Variable::F32( f ) ) => {
					self.baked = Baked::STRING( format!("{}", f).to_string() );
				},
				Some( Variable::String( s ) ) => {
					self.baked = Baked::STRING( s.to_string() );
				},
				Some( Variable::ERROR( e ) ) => {
					println!("Error baking {:?} in {:?}", self, context );
					self.baked = Baked::STRING( default.to_string() );
				},
				None => {
					self.baked = Baked::STRING( default.to_string() );	
				},
				t => todo!("Result type not handled {:?} {:?} {:?}", t, r, e ),
			}
		} else {
			match self.baked {
				Baked::F32( _ ) => {}, // just keep the baked value
				_ => self.baked = Baked::STRING( default.to_string() ),
			}
		}
	}

	pub fn as_f32( &self ) -> f32 {
		match self.baked {
			Baked::F32( f ) => f,
			Baked::U32( u ) => u as f32,
			_ => 0.0,	// :TODO: report error in "trace" mode
		}
	}

	pub fn as_u32( &self ) -> u32 {
		match self.baked {
			Baked::U32( u ) => u,
			Baked::F32( f ) => f as u32,
			_ => 0,	// :TODO: report error in "trace" mode
		}
	}

	pub fn as_string( &self ) -> String {
		match &self.baked {
			Baked::STRING( s ) => s.clone(),
			Baked::U32( u ) => format!( "{}", u ).to_string(),
			Baked::F32( f ) => format!( "{}", f ).to_string(),
			_ => String::new(),	// :TODO: report error in "trace" mode
		}
	}
}

