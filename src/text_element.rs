use crate::element::{Element, ElementConfig};
use crate::pixel::Pixel;
use crate::context::Context;
use async_trait::async_trait;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;
use crate::axisalignedrectangle::AxisAlignedRectangle;

use std::fs::File;
use std::io::Read;
use rusttype::{point, Font, Scale};

use crate::variable::Variable;

#[derive(Debug)]
pub struct TextElement {
	name: String,
	x: Variable,
	y: Variable,
	width: u32,
	height: u32,
	color: u32,
	text: String,
	fontfile: String,
	size: u32,
	font: Option< Font<'static> >,
	display_text: String,
	bounding_box: AxisAlignedRectangle,
}

impl TextElement {
	fn _fill_box( buffer: &mut Vec<u32>, width: usize, height: usize, x: u32, y: u32, w: u32, h: u32, color: u32 ) {
		for iy in 0..h {
			let py = iy + y;
			if py >= height as u32 { continue; }
			for ix in 0..w {
				let px = ix + x;
				if px >= width as u32 { continue; }

				let o = ( py * width as u32 + px ) as usize;
				buffer[ o ] = color;
			}
		}
	}
}

#[async_trait]
impl Element for TextElement {
	fn configure( &mut self, config: &ElementConfig ) {
//		self.x      = config.get_u32_or( "pos_x", 0 );
		self.x      = config.get_variable_or( "pos_x", 0u32 );
		self.y      = config.get_variable_or( "pos_y", 0u32 );
		self.width  = config.get_u32_or( "width", 0 );
		self.height = config.get_u32_or( "height", 0 );
		self.color  = config.get_u32_or( "color", 0xffff00ff );
		self.text	= config.get_string_or( "text", "" );
		self.fontfile	= config.get_string_or( "font", "" );
		self.size	= config.get_u32_or( "size", 20 );
		self.display_text	= config.get_string_or( "text", "" );

		// NOTE: We could just directly us the self.bounding_box, but want to keep our options open
		let mut bb = AxisAlignedRectangle::new();

		let junk = 0;
		bb.x = config.get_u32_or( "bounding_box_pos_x", junk ); // :TODO: resolve variable or use variable in bb -> self.x );
		bb.y = config.get_u32_or( "bounding_box_pos_y", junk ); // :TODO: resolve variable or use variable in bb -> self.y );
		bb.width = config.get_u32_or( "bounding_box_width", self.width );
		bb.height = config.get_u32_or( "bounding_box_height", self.height );

		self.bounding_box = bb;

		dbg!(&self);
	}

	fn shutdown( &mut self ) {
		
	}
	
	async fn run( &mut self ) -> anyhow::Result<()> {
		Ok(())
	}


	fn update( &mut self, context: &mut Context ) {
		self.display_text = context.expand_string_or( &self.text, "" );
		self.x.bake_u32_or( context, 0 );
		self.y.bake_u32_or( context, 0 );
	}

	fn render( &self, render_buffer: &mut RenderBuffer, render_context: &mut RenderContext ) {

		if &self.name == "Banner Title" {
			dbg!(&self);
		}

//		dbg!(&self);
		render_context.use_font( &self.fontfile );
		render_context.draw_text(
			render_buffer,
			&self.display_text,
			self.x.as_u32(), self.y.as_u32(),
			self.width, self.height,
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

pub struct TextElementFactory {

}

impl TextElementFactory {
	pub fn create() -> TextElement {
		TextElement {
			name: "".to_string(),
			x: Variable::from_u32( 0 ),
			y: Variable::from_u32( 0 ),
			width: 0,
			height: 0,
			color: 0xff00ffff,
			text: "".to_string(),
			fontfile: "".to_string(),
			size: 20,
			font: None,
			display_text: "".to_string(),
			bounding_box: AxisAlignedRectangle::new(),
		}
	}
}

