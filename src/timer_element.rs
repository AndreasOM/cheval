use crate::bakedexpression::BakedExpression;
use crate::element::{Element, ElementConfig};
use crate::context::Context;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;

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
	initial_value: BakedExpression,
	scale: BakedExpression,
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
		self.initial_value	= config.get_bakedexpression_f32( "initial_value", 0.0 );
		self.scale			= config.get_bakedexpression_f32( "scale", 1.0 );

//		dbg!(&self);
//		todo!("die");
	}

	async fn run( &mut self ) -> anyhow::Result<()> {
		Ok(())
	}


	fn update( &mut self, context: &mut Context ) {
		self.scale.bake_f32_or( context, 1.0 );
		let scale: f32 = self.scale.as_f32();

//		let scale: f32 = self.scale.into(); // :TODO: fix me

		// count
//		dbg!(&self.variable);
		let ov = match context.get_f32( &self.variable ) {
			Some( v ) => {
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
//					dbg!(&self);
					if self.repeat {
						self.initial_value.bake_f32_or( context, 0.0 );
						let initial_value = self.initial_value.as_f32();
						v + initial_value
					} else {
						v
					}
				} else {
					v
				};

				context.set_f32( &self.variable, v );
				Some( v )
			},
			None => {
				self.initial_value.bake_f32_or( context, 0.0 );
				let v = self.initial_value.as_f32();
				println!("Setting initial value for {} to {}", &self.name, v );
				context.set_f32( &self.variable, v );
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
			initial_value: BakedExpression::from_u32( 0 ),
			scale: BakedExpression::from_f32( 1.0 ),
		}
	}
}

