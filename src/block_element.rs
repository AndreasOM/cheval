use crate::element::{Element, ElementConfig};
use crate::context::Context;

use async_trait::async_trait;

#[derive(Debug)]
pub struct BlockElement {
	name: String,
	x: u32,
	y: u32,
	width: u32,
	height: u32,
	color: u32,
}

impl BlockElement {
}

#[async_trait]
impl Element for BlockElement {
	fn configure( &mut self, config: &ElementConfig ) {
		self.x      = config.get_u32_or( "pos_x", 0 );
		self.y      = config.get_u32_or( "pos_y", 0 );
		self.width  = config.get_u32_or( "width", 0 );
		self.height = config.get_u32_or( "height", 0 );
		self.color  = config.get_u32_or( "color", 0xffff00ff );
	}

	fn shutdown( &mut self ) {
		
	}

	async fn run( &mut self ) -> anyhow::Result<()> {
		Ok(())
	}


	fn update( &mut self, context: &mut Context ) {
	}

	fn render( &self, buffer: &mut Vec<u32>, width: usize, height: usize ) {
//		dbg!(&self);
		for y in 0..self.height {
			let py = y + self.y;
			if py >= height as u32 { continue; }
			for x in 0..self.width {
				let px = x + self.x;
				if px >= width as u32 { continue; }

//				dbg!(&px, &py);

				let o = ( py * width as u32 + px ) as usize;
				buffer[ o ] = self.color;
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
		"block"
	}
}

pub struct BlockElementFactory {

}

impl BlockElementFactory {
	pub fn create() -> BlockElement {
		BlockElement {
			name: "".to_string(),
			x: 0,
			y: 0,
			width: 0,
			height: 0,
			color: 0xff00ffff,
		}
	}
}

