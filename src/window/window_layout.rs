use serde::{
	Deserialize,
	Serialize,
};
use serde_yaml;

use std::path::Path;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct WindowLayoutWindowConfig {
	pub pos_x: u32,
	pub pos_y: u32,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct WindowLayout {
	pub window_rgb: Option< WindowLayoutWindowConfig >,
	pub window_a: Option< WindowLayoutWindowConfig >,
}

impl WindowLayout {
/*
	pub fn new() -> Self {
		Self {
			window_rgb: None,
		}
	}
*/
	pub fn load( &mut self, filename: &Path ) -> anyhow::Result<()> {
//		let b = fs::read("address.txt")?;
//		let s = String::from_utf8_lossy( &b );
//		let c = serde_yaml::from_str(&s)?;
//		let s = String::from_utf8_lossy( &fs::read("address.txt")? );
//		let c = serde_yaml::from_str(&s)?;
		let f = std::fs::File::open( &filename )?;
		let c = serde_yaml::from_reader( &f )?;
		*self = c;

//		dbg!(&self);
		Ok(())
	}

	pub fn save( &self, filename: &Path ) -> anyhow::Result<()> {
		let s = serde_yaml::to_string(&self)?;
//		dbg!(&s);
//		let mut buffer = File::create( filename )?;
//		buffer.write_all( &s.as_bytes() );
		std::fs::write( &filename, &s.as_bytes() )?;
//		write!(buffer, &s);
		Ok(())
	}
}

