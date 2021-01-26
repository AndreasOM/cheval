#[derive(Debug)]
pub struct AxisAlignedRectangle {
	pub x: u32,
	pub y: u32,
	pub width: u32,
	pub height: u32,
}

impl AxisAlignedRectangle {
	pub fn new() -> Self {
		AxisAlignedRectangle{
			x: 0,
			y: 0,
			width: 0,
			height: 0,
		}
	}
}