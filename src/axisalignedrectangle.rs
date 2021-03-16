use crate::bakedexpression::BakedExpression;
use crate::context::Context;

#[derive(Debug)]
pub struct AxisAlignedRectangle {
	pub x: BakedExpression,
	pub y: BakedExpression,
	pub width: u32,
	pub height: u32,
}

impl AxisAlignedRectangle {
	pub fn new() -> Self {
		AxisAlignedRectangle{
			x: BakedExpression::from_u32( 0 ),
			y: BakedExpression::from_u32( 0 ),
			width: 0,
			height: 0,
		}
	}

	pub fn bake( &mut self, context: &mut Context ) -> bool {
		self.x.bake_u32_or( context, 0 );
		self.y.bake_u32_or( context, 0 );
		true
	}

}