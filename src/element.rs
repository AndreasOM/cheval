use std::collections::HashMap;

pub enum ElementConfigEntry {
	U32( u32 ),
}

pub struct ElementConfig {
	entries: HashMap<String, ElementConfigEntry>,
}

impl ElementConfig {
	pub fn new() -> Self {
		Self {
			entries: HashMap::new(),
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
}

pub trait Element {
	fn configure( &mut self, config: &ElementConfig );
	fn update( &mut self );
	fn render( &self, buffer: &mut Vec<u32>, width: usize, height: usize );
	fn name( &self ) -> &str;
}

impl std::fmt::Debug for Element {
	fn fmt( &self, f: &mut std::fmt::Formatter ) -> std::fmt::Result {
		writeln!( f,"[Trait] Element: {}", self.name() )
	}
}
