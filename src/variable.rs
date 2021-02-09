use crate::context::Context;

use std::convert::From;

#[derive(Clone,Debug)]
pub enum Original {
	EMPTY,
	U32(u32),
	F32(f32),
	STRING(String)
}

#[derive(Clone,Debug)]
pub enum Baked {
	EMPTY,
	U32(u32),
	F32(f32),
	STRING(String)
}

#[derive(Clone,Debug)]
pub struct Variable {
	original: Original,
	baked: Baked
}

impl Variable {

	pub fn new() -> Self {
		Self {
			original: Original::EMPTY,
			baked: Baked::EMPTY,
		}
	}
	// :TODO: consider from trait instead of explicit call
	pub fn from_f32( v: f32 ) -> Self {
		Self {
			original: Original::F32( v ),
			baked: Baked::EMPTY,
		}
	}
	pub fn from_u32( v: u32 ) -> Self {
		Self {
			original: Original::U32( v ),
			baked: Baked::EMPTY,
		}
	}
	pub fn from_str( v: &str ) -> Self {
		Self {
			original: Original::STRING( v.to_owned() ),
			baked: Baked::EMPTY,
		}
	}

	// :HACK:
	pub fn original(&self) -> Original {
		self.original.clone()
	}

	// :TODO: maybe it could be better to have the context bake us, instead of us baking ourselves
	pub fn bake_u32_or( &mut self, context: &mut Context, default: u32 ) -> bool {
		self.baked =  Baked::U32( context.expand_var_to_u32_or( &self, default ) );
		true
	}
	pub fn bake_f32_or( &mut self, context: &mut Context, default: f32 ) -> bool {
		self.baked =  Baked::F32( context.expand_var_to_f32_or( &self, default ) );
		true
	}

	// :TODO: maybe we want to implement into here
	pub fn as_u32(&self) -> u32 {
		match self.baked {
			Baked::U32( v ) => v,
			_ => panic!("Tried to get Variable as u32 that is not an U32"),
		}
	}
	pub fn as_f32(&self) -> f32 {
		match self.baked {
			Baked::F32( v ) => v,
			_ => panic!("Tried to get Variable as f32 that is not an F32"),
		}
	}
}
/*
impl From<Variable> for f32 {
    fn from(v: Variable) -> Self {
		match v.baked {
			Baked::F32( v ) => v,
			_ => panic!("Tried to get Variable as f32 that is not an F32"),
		}
    }
}
*/
impl Into<f32> for Variable {
    fn into(self) -> f32 {
		match self.baked {
			Baked::F32( v ) => v,
			_ => panic!("Tried to get Variable as f32 that is not an F32"),
		}
    }
}

