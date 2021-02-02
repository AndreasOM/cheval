use std::collections::HashMap;

use crate::context::Context;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;

use crate::variable::Variable;

use async_trait::async_trait;

#[derive(Debug)]
pub enum ElementConfigEntry {
	U32( u32 ),
	STRING( String ),
	BOOL( bool ),
}

#[derive(Debug)]
pub struct ElementConfig {
	entries: HashMap<String, ElementConfigEntry>,
}

impl ElementConfig {
	pub fn new() -> Self {
		Self {
			entries: HashMap::new(),
		}
	}

	pub fn set( &mut self, name: &str, value: &str ) {
		if value == "true" {
			self.entries.insert( name.to_string(), ElementConfigEntry::BOOL( true ) );
		} else if value == "false" {
			self.entries.insert( name.to_string(), ElementConfigEntry::BOOL( false ) );
		} else if let Ok( v ) = value.parse::<u32>() {
			self.entries.insert( name.to_string(), ElementConfigEntry::U32( v ) );
		} else  {
			if let Some( v ) = value.strip_prefix( "0x" ) {
				match u32::from_str_radix( v, 16 ) {
					Ok( v ) => self.entries.insert( name.to_string(), ElementConfigEntry::U32( v ) ),
					_ => self.entries.insert( name.to_string(), ElementConfigEntry::STRING( value.to_string() ) ),
				};
			} else {
				self.entries.insert( name.to_string(), ElementConfigEntry::STRING( value.to_string() ) );	
			}
		}
	}

	pub fn set_u32( &mut self, name: &str, value: u32 ) {
		self.entries.insert( name.to_string(), ElementConfigEntry::U32( value ) );
	}

	pub fn get_u32_or( &self, name: &str, default: u32 ) -> u32 {
		match self.entries.get( name ) {
			Some( ElementConfigEntry::U32( v ) ) => *v,
			_ => default,
		}
	}

	pub fn get_variable_or( &self, name: &str, default: u32 ) -> Variable {
		match self.entries.get( name ) {
			Some( ElementConfigEntry::U32( v ) ) => Variable::U32( *v ),
			Some( ElementConfigEntry::STRING( v ) ) => Variable::STRING( v.clone() ),
			_ => Variable::U32( default ),
		}
	}

	pub fn get_string_or( &self, name: &str, default: &str ) -> String {
		match self.entries.get( name ) {
			Some( ElementConfigEntry::STRING( s ) ) => s.clone(),
			Some( ElementConfigEntry::U32( v ) ) => format!("{}", v),
			_ => default.to_string(),
		}
	}

	pub fn get_bool_or( &self, name: &str, default: bool ) -> bool {
		match self.entries.get( name ) {
			Some( ElementConfigEntry::BOOL( b ) ) => *b,
			_ => default,
		}
	}
}

#[async_trait]
pub trait Element {
	fn configure( &mut self, config: &ElementConfig );
	fn shutdown( &mut self ) {}
	fn update( &mut self, _context: &mut Context ) {}
	// fn render( &self, _buffer: &mut Vec<u32>, _width: usize, _height: usize ) {}
	fn render( &self, _render_buffer: &mut RenderBuffer, _render_context: &mut RenderContext ) {}
	async fn run( &mut self ) -> anyhow::Result<()>;
	fn name( &self ) -> &str;
	fn set_name( &mut self, name: &str );
	fn element_type( &self ) -> &str;
}

impl std::fmt::Debug for dyn Element {
	fn fmt( &self, f: &mut std::fmt::Formatter ) -> std::fmt::Result {
		writeln!( f,"[Trait] Element: {} [{}]", self.name(), self.element_type() )
	}
}


