use crate::bakedexpression::BakedExpression;
use crate::element::{Element, ElementConfig};
use crate::context::Context;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;

use async_trait::async_trait;

#[derive(Debug)]
pub struct BlockElement {
	name: String,
	x: BakedExpression,
	y: BakedExpression,
	width: BakedExpression,
	height: BakedExpression,
	color: u32,
}

impl BlockElement {
}

#[async_trait]
impl Element for BlockElement {
	fn configure( &mut self, config: &ElementConfig ) {
		self.x      = config.get_bakedexpression_u32( "pos_x", 0 );
		self.y      = config.get_bakedexpression_u32( "pos_y", 0 );
		self.width  = config.get_bakedexpression_u32( "width", 0 );
		self.height = config.get_bakedexpression_u32( "height", 0 );
		self.color  = config.get_color_or( "color", 0xffff00ff );
	}

	async fn run( &mut self ) -> anyhow::Result<()> {
		Ok(())
	}

	fn update( &mut self, context: &mut Context ) {
		self.x.bake_u32_or( context, 0 );
		self.y.bake_u32_or( context, 0 );
		self.width.bake_u32_or( context, 0 );
		self.height.bake_u32_or( context, 0 );
	}

	fn render( &self, render_buffer: &mut RenderBuffer, render_context: &mut RenderContext ) {
//		dbg!(&self);
//		dbg!(&render_context);
/*
		for( x, y, block_x, block_y, pixel) in render_buffer.enumerate_pixel_in_block_mut( self.x, self.y, self.width, self.height ) {
			*pixel = self.color;
		}
*/

		render_buffer.for_pixel_in_block( self.x.as_u32(), self.y.as_u32(), self.width.as_u32(), self.height.as_u32(), |_x,_y,_bx,_by,p: &mut u32| {
			*p = self.color;
		});
	}
	fn name( &self ) -> &str {
		&self.name
	}
	fn set_name(&mut self, name: &str ) {
		self.name = name.to_string();
	}

	fn element_type( &self ) -> &str {
		"block"
	}
}

pub struct BlockElementFactory {

}

impl BlockElementFactory {
	pub fn create() -> BlockElement {
		BlockElement {
			name: "".to_string(),
			x: BakedExpression::from_u32( 0 ),
			y: BakedExpression::from_u32( 0 ),
			width: BakedExpression::from_u32( 0 ),
			height: BakedExpression::from_u32( 0 ),
			color: 0xff00ffff,
		}
	}
}

