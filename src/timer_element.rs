use crate::element::{Element, ElementConfig};
use crate::context::Context;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;
use crate::variable::Variable;

use async_trait::async_trait;
use hhmmss::Hhmmss;

#[derive(Debug)]
enum Mode {
	Countdown,
	StopWatch,
}

#[derive(Debug)]
pub struct TimerElement {
	name: String,
	variable: String,
	text_variable: String,
	hide_on_zero: bool,
	repeat: bool,
	mode: Mode,
	initial_value: Variable,
	scale: Variable,
}

impl TimerElement {
}

#[async_trait]
impl Element for TimerElement {
	fn configure( &mut self, config: &ElementConfig ) {
		self.variable		= config.get_string_or( "variable", "" );
		self.text_variable	= config.get_string_or( "text_variable", "" );
		self.hide_on_zero	= config.get_bool_or( "hide_on_zero", false );
		self.repeat			= config.get_bool_or( "repeat", false );
		self.mode = match config.get_string_or( "mode", "Countdown" ).as_ref() {
			"StopWatch" => Mode::StopWatch,
			_ => Mode::Countdown,
		};
		self.initial_value	= config.get_variable_or( "initial_value", 0 );
		self.scale		= config.get_variable_or( "scale", 1u32 );
	}

	async fn run( &mut self ) -> anyhow::Result<()> {
		Ok(())
	}


	fn update( &mut self, context: &mut Context ) {
		self.scale.bake_f32_or( context, 1.0 );
		let scale = self.scale.as_f32();
		// count
		let ov = match context.get_string( &self.variable ) {
			Some( value ) => {
//				dbg!(&self.name, &value);
				if let Ok( v ) = value.parse::<f32>() {
					let v = match self.mode {
						Mode::Countdown => {
							v - scale * context.time_step() as f32
						},
						Mode::StopWatch => {
							if v >= 0.0 {
								v + scale * context.time_step() as f32
							} else {
								v
							}
						},
					};
//					let v = if v > 0.0 { v } else { 0.0 };
//					let duration = std::time::Duration::new( v as u64, 0);
					let v = if v < 0.0 {
						if self.repeat {
							let initial_value = context.expand_var_to_f32_or( &self.initial_value, 0.0 );
							v + initial_value
						} else {
							v
						}
					} else {
						v
					};

					let s = format!("{}", v );
					context.set_string( &self.variable, &s );					
					Some( v )
				} else {
//					let v = &self.initial_value;
//					context.set_string( &self.variable, &v );
					None
				}
			},
			None => {
				let v = context.expand_var_to_f32_or( &self.initial_value, 0.0 );
				let s = format!("{}", v );
				println!("Setting initial value for {} to {}", &self.name, &s );
				context.set_string( &self.variable, &s );
				dbg!(&context);
				None
			},
		};
		// format
		match ov {
			Some( v ) => {
				if v <= 1.0 && self.hide_on_zero {
					context.set_string( &self.text_variable, "" );
				} else if v <= 86400.0 {
					let duration = std::time::Duration::new( v as u64, 0);
					context.set_string( &self.text_variable, &duration.hhmmss() );
				} else {
					context.set_string( &self.text_variable, ":TODO: >24h" );
				}
			},
			None => {
				context.set_string( &self.text_variable, "NaN" );
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

pub struct TimerElementFactory {

}

impl TimerElementFactory {
	pub fn create() -> TimerElement {
		TimerElement {
			name: "".to_string(),
			variable: "".to_string(),
			text_variable: "".to_string(),
			hide_on_zero: false,
			repeat: false,
			mode: Mode::Countdown,
			initial_value: Variable::new(),
			scale: Variable::from_f32( 1.0 ),
		}
	}
}

