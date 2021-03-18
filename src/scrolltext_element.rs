use crate::bakedexpression::BakedExpression;
use crate::element::{Element, ElementConfig};
use crate::pixel::Pixel;
use crate::context::Context;
use async_trait::async_trait;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;
use crate::axisalignedrectangle::AxisAlignedRectangle;

use std::fs::File;
use std::io::Read;
// use rusttype::{point, Font, Scale};

#[derive(Debug)]
pub struct ScrollTextElement {
	name: String,
	color: u32,
	text: String,
	fontfile: String,
	speed: f32,
	size: u32,
	display_text: String,
	bounding_box: AxisAlignedRectangle,
	offset: f32,
}

impl ScrollTextElement {
}

#[async_trait]
impl Element for ScrollTextElement {
	fn configure( &mut self, config: &ElementConfig ) {
		self.color  		= config.get_u32_or( "color", 0xffff00ff );
		self.text			= config.get_string_or( "text", "" );
		self.fontfile		= config.get_string_or( "font", "" );
		self.speed  		= config.get_f32_or( "speed", 0.0 );
		self.size			= config.get_u32_or( "size", 20 );
		self.display_text	= config.get_string_or( "text", "" );
     
		let mut bb = AxisAlignedRectangle::new();

		bb.x = config.get_bakedexpression_empty( "bounding_box_pos_x" );
		bb.y = config.get_bakedexpression_empty( "bounding_box_pos_y" );
		bb.width = config.get_bakedexpression_empty( "bounding_box_width" );
		bb.height = config.get_bakedexpression_empty( "bounding_box_height" );

		self.bounding_box = bb;
	}

	fn shutdown( &mut self ) {
		
	}
	
	async fn run( &mut self ) -> anyhow::Result<()> {
		Ok(())
	}


	fn update( &mut self, context: &mut Context ) {
		self.display_text = context.expand_string_or( &self.text, "" );
		self.offset -= self.speed * context.time_step() as f32;
		// :TODO: use calculated width of text instead of hardcoded values
		if self.offset < -600.0 {
			self.offset += 600.0 + 500.0;
		}

		// :TODO: bake with default
		self.bounding_box.bake( context );
	}

	fn render( &self, render_buffer: &mut RenderBuffer, render_context: &mut RenderContext ) {
//		dbg!(&self);
		render_context.use_font( &self.fontfile );
		let pos_x = self.bounding_box.x.as_u32() as f32 + self.offset;

		render_context.draw_text(
			render_buffer,
			&self.display_text,
			pos_x as u32, self.bounding_box.y.as_u32(),
			self.bounding_box.width.as_u32(), self.bounding_box.height.as_u32(),
			&self.bounding_box,
			self.size,					// :TODO: maybe move this to use font
			self.color
		);
	}
	fn name( &self ) -> &str {
		&self.name
	}
	fn set_name(&mut self, name: &str ) {
		self.name = name.to_string();
	}

	fn element_type( &self ) -> &str {
		"text"
	}
}

pub struct ScrollTextElementFactory {

}

impl ScrollTextElementFactory {
	pub fn create() -> ScrollTextElement {
		ScrollTextElement {
			name: "".to_string(),
			color: 0xff00ffff,
			text: "".to_string(),
			fontfile: "".to_string(),
			speed: 0.0,
			size: 20,
			display_text: "".to_string(),
			bounding_box: AxisAlignedRectangle::new(),
			offset: 0.0,
		}
	}
}

