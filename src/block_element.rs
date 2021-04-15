use crate::bakedexpression::BakedExpression;
use crate::element::{Element, ElementConfig};
use crate::context::Context;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;
use crate::pixel::Pixel;

use async_trait::async_trait;

#[derive(Debug)]
pub struct BlockElement {
	name: String,
	x: BakedExpression,
	y: BakedExpression,
	width: BakedExpression,
	height: BakedExpression,
	color: u32,
	alpha: BakedExpression,
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
		self.alpha  = config.get_bakedexpression_f32( "alpha", 1.0 );
	}

	async fn run( &mut self ) -> anyhow::Result<()> {
		Ok(())
	}

	fn update( &mut self, context: &mut Context ) {
		self.x.bake_u32_or( context, 0 );
		self.y.bake_u32_or( context, 0 );
		self.width.bake_u32_or( context, 0 );
		self.height.bake_u32_or( context, 0 );
		self.alpha.bake_f32_or( context, 1.0 );
	}

	fn render( &self, render_buffer: &mut RenderBuffer, render_context: &mut RenderContext ) {
//		dbg!(&self);
//		dbg!(&render_context);
/*
		for( x, y, block_x, block_y, pixel) in render_buffer.enumerate_pixel_in_block_mut( self.x, self.y, self.width, self.height ) {
			*pixel = self.color;
		}
*/

		let mut pixel = Pixel::from_u32( self.color );
		let a = self.alpha.as_f32();
		if a < 1.0 && a >= 0.0 {
			pixel.apply_alpha( a );
		}


		render_buffer.for_pixel_in_block(
			self.x.as_u32(), self.y.as_u32(), self.width.as_u32(), self.height.as_u32(),
			|_x,_y,_bx,_by,p: &mut u32| {
				let old_pixel = *p;
				let old_pixel = Pixel::from_u32( old_pixel );
				let blended_pixel = Pixel::blend_with_alpha( &pixel, &old_pixel );

				*p = blended_pixel.to_u32();
			}
		);
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
			alpha: BakedExpression::from_f32( 1.0 ),
		}
	}
}

