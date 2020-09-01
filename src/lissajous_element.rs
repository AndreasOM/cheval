use crate::element::{Element, ElementConfig};
use crate::context::Context;
use async_trait::async_trait;

#[derive(Debug)]
pub struct LissajousElement {
	name: String,
	x: u32,
	y: u32,
	width: u32,
	height: u32,
	color: u32,
	count: u32,
	offset: f32,
	t: f32,
}

impl LissajousElement {
}

#[async_trait]
impl Element for LissajousElement {
	fn configure( &mut self, config: &ElementConfig ) {
		self.x      = config.get_u32_or( "pos_x", 0 );
		self.y      = config.get_u32_or( "pos_y", 0 );
		self.width  = config.get_u32_or( "width", 0 );
		self.height = config.get_u32_or( "height", 0 );
		self.color  = config.get_u32_or( "color", 0xffff00ff );
		self.count  = config.get_u32_or( "count", 1 );
		self.offset  = config.get_u32_or( "offset", 0 ) as f32;
	}
	async fn run( &mut self )  -> anyhow::Result<()> {
		Ok(())
	}


	fn update( &mut self, context: &mut Context ) {
		self.t += 0.1;
	}
	
	fn render( &self, buffer: &mut Vec<u32>, width: usize, height: usize ) {
		for c in 0..self.count {
			let t = self.t + self.offset + 0.1 * c as f32;
			let x = ( ( self.width as f32 * t.sin() ) as isize ).saturating_add( self.x as isize ) as i32;
			let t = t * 1.5;
			let y = ( ( self.height as f32 * t.sin() ) as isize ).saturating_add( self.y as isize ) as i32;
			for dy in 0..5 {
				for dx in 0..5 {
					if x < 0 || x.saturating_add( dx ) >= width as i32 { continue; }
					if y < 0 || y.saturating_add( dy ) >= height as i32 { continue; }

					let o = ( ( y+dy ) * width as i32 + x + dx ) as usize;

					buffer[ o ] = self.color;
				}
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
		"lissajous"
	}

}

pub struct LissajousElementFactory {

}

impl LissajousElementFactory {
	pub fn create() -> LissajousElement {
		LissajousElement {
			name: "".to_string(),
			x: 0,
			y: 0,
			width: 0,
			height: 0,
			count: 1,
			color: 0xff00ffff,
			t: 0.0,
			offset: 0.0,
		}
	}
}

