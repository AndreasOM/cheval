
#[derive(Debug)]
pub struct RenderBuffer {
	pub buffer: Vec<u32>,
	pub width:  usize,
	pub height: usize,
}

impl RenderBuffer {
	pub fn new(
		width:  usize,
		height: usize,		
	) -> Self {
		Self {
			buffer: vec![0u32; width * height],
			width,
			height,
		}
	}
}
