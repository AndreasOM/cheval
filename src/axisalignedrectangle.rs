use crate::bakedexpression::BakedExpression;
use crate::context::Context;

#[derive(Debug)]
pub struct AxisAlignedRectangle {
	pub x:      BakedExpression,
	pub y:      BakedExpression,
	pub width:  BakedExpression,
	pub height: BakedExpression,
}

impl AxisAlignedRectangle {
	pub fn new() -> Self {
		AxisAlignedRectangle {
			x:      BakedExpression::from_u32(0),
			y:      BakedExpression::from_u32(0),
			width:  BakedExpression::from_u32(0),
			height: BakedExpression::from_u32(0),
		}
	}

	pub fn bake(&mut self, context: &mut Context) -> bool {
		self.x.bake_u32_or(context, 0);
		self.y.bake_u32_or(context, 0);
		self.width.bake_u32_or(context, 0);
		self.height.bake_u32_or(context, 0);
		true
	}

	pub fn bake_or(&mut self, context: &mut Context, ar: &AxisAlignedRectangle) -> bool {
		self.x.bake_u32_or(context, ar.x.as_u32());
		self.y.bake_u32_or(context, ar.y.as_u32());
		self.width.bake_u32_or(context, ar.width.as_u32());
		self.height.bake_u32_or(context, ar.height.as_u32());
		true
	}
}
