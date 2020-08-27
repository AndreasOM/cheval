use std::collections::HashMap;

#[derive(Debug)]
pub struct Context {
	variables: HashMap<String,String>,
}

impl Context {
	pub fn new() -> Self {
		Self {
			variables: HashMap::new(),
		}
	}

	pub fn set_string( &mut self, name: &str, value: &str ) {
		self.variables.insert( name.to_string(), value.to_string() );
	}

	pub fn get_string( &self, name: &str ) -> Option< &str > {
		self.variables.get( name ).map(|s| s.as_ref() )
	}
}
