use crate::bakedexpression::BakedExpression;
use crate::element::{Element, ElementConfig};
use crate::context::Context;
use async_trait::async_trait;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;
use crate::axisalignedrectangle::AxisAlignedRectangle;

use rusttype::Font;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TextElement {
	name: String,
	ar: AxisAlignedRectangle,
	color: u32,
	text: BakedExpression,
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
		self.ar.x      = config.get_bakedexpression_u32( "pos_x", 0 );
		self.ar.y      = config.get_bakedexpression_u32( "pos_y", 0);
		self.ar.width  = config.get_bakedexpression_u32( "width", 0 );
		self.ar.height = config.get_bakedexpression_u32( "height", 0 );
		self.color  = config.get_u32_or( "color", 0xffff00ff );
		self.text	= config.get_bakedexpression_string( "text", "" );
		self.fontfile	= config.get_path_or( "font", "" );
		self.size	= config.get_u32_or( "size", 20 );
		self.display_text	= config.get_string_or( "text", "" );

		// NOTE: We could just directly us the self.bounding_box, but want to keep our options open
		let mut bb = AxisAlignedRectangle::new();

		bb.x = config.get_bakedexpression_empty( "bounding_box_pos_x" );
		bb.y = config.get_bakedexpression_empty( "bounding_box_pos_y" );
		bb.width = config.get_bakedexpression_empty( "bounding_box_width" );
		bb.height = config.get_bakedexpression_empty( "bounding_box_height" );

		self.bounding_box = bb;

//		dbg!(&self);
	}

	fn shutdown( &mut self ) {
		
	}
	
	async fn run( &mut self ) -> anyhow::Result<()> {
		Ok(())
	}


	fn update( &mut self, context: &mut Context ) {
//		self.display_text = context.expand_string_or( &self.text, "" );
		// Note: we could just bake ar
		self.ar.x.bake_u32_or( context, 0 );
		self.ar.y.bake_u32_or( context, 0 );
		self.ar.width.bake_u32_or( context, 0 );
		self.ar.height.bake_u32_or( context, 0 );
		self.bounding_box.bake_or( context, &self.ar );

		self.text.bake_string_or( context, "" );
	}

	fn render( &self, render_buffer: &mut RenderBuffer, render_context: &mut RenderContext ) {
/*
		if &self.name == "Banner Title" {
			dbg!(&self);
		}
*/
//		dbg!(&self);
		match render_context.use_font( &self.fontfile ) {
			// :TODO: handle error
			_ => {},			
		}
		match render_context.draw_text(
			render_buffer,
			&self.text.as_string(),
			self.ar.x.as_u32(), self.ar.y.as_u32(),
			self.ar.width.as_u32(), self.ar.height.as_u32(),
			&self.bounding_box,
			self.size,					// :TODO: maybe move this to use font
			self.color
		) {
			// :TODO: handle error
			_ => {},			
		}
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
			ar: AxisAlignedRectangle::new(),
			color: 0xff00ffff,
			text: BakedExpression::from_str(""),
			fontfile: "".to_string(),
			size: 20,
			font: None,
			display_text: "".to_string(),
			bounding_box: AxisAlignedRectangle::new(),
		}
	}
}

