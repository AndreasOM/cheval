use crate::element::{Element, ElementConfig};
use crate::context::Context;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;

use async_trait::async_trait;
use hhmmss::Hhmmss;

#[derive(Debug)]
pub struct CountdownElement {
	name: String,
	variable: String,
	text_variable: String,
	hide_on_zero: bool,
}

impl CountdownElement {
}

#[async_trait]
impl Element for CountdownElement {
	fn configure( &mut self, config: &ElementConfig ) {
		self.variable		= config.get_string_or( "variable", "" );
		self.text_variable	= config.get_string_or( "text_variable", "" );
		self.hide_on_zero	= config.get_bool_or( "hide_on_zero", false );
	}

	async fn run( &mut self ) -> anyhow::Result<()> {
		Ok(())
	}

	fn update( &mut self, context: &mut Context ) {
		match context.get_string( &self.variable ) {
			Some( value ) => {
//				dbg!(&value);
				if let Ok( v ) = value.parse::<f32>() {
					let v = v - context.time_step() as f32;
					let v = if v > 0.0 { v } else { 0.0 };
					let s = format!("{}", v );
					context.set_string( &self.variable, &s );
					let duration = std::time::Duration::new( v as u64, 0);
					let fs = if v <= 1.0 {
						if self.hide_on_zero {
							"".to_string()
						} else {
							duration.hhmmss()
						}
					} else if v <= 86400.0 {
						duration.hhmmss()
					} else {
						":TODO: >24h".to_string()
					};
					context.set_string( &self.text_variable, &fs );
				} else {
					context.set_string( &self.text_variable, "NaN" );
				}

			},
			None => {
				context.set_string( &self.variable, "0.0" );
				context.set_string( &self.text_variable, "" );
			}
		}
	}

	fn name( &self ) -> &str {
		&self.name
	}
	fn set_name(&mut self, name: &str ) {
		self.name = name.to_string();
	}

	fn element_type( &self ) -> &str {
		"countdown"
	}
}

pub struct CountdownElementFactory {

}

impl CountdownElementFactory {
	pub fn create() -> CountdownElement {
		CountdownElement {
			name: "".to_string(),
			variable: "".to_string(),
			text_variable: "".to_string(),
			hide_on_zero: false,
		}
	}
}

