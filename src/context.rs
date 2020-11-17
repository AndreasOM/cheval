use std::collections::HashMap;

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
}
