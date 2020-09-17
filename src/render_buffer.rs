
#[derive(Debug)]
pub struct RenderBuffer {
	pub buffer: Vec<u32>,
	pub width:  usize,
	pub height: usize,
}

/*
pub struct PixelInBlockMut<'a> {
	block_x: u32,
	block_y: u32,
	render_buffer: &'a mut RenderBuffer,
}
impl<'a> std::iter::Iterator for PixelInBlockMut<'a> {
	type Item = ( u32, u32, u32, u32, &'a mut u32 );
    fn next(&mut self) -> Option<Self::Item> {
    	let w = self.render_buffer.width as u32;
    	let h = self.render_buffer.height as u32;

    	if self.block_x < w && self.block_y < h {
    		let o = self.block_y * w + self.block_x;
    		self.block_x += 1;
    		if self.block_x >= w {
    			self.block_x = 0;
    			self.block_y +=1 ;
    		}
    		Some( ( 0, 0, 0, 0, &mut self.render_buffer.buffer[ o as usize ] ) )
    	}
    	else
    	{
	    	None
    	}
    }
}
*/
/*
impl PixelInBlockIteratorMut for RenderBuffer {
	type Item<'a> = ( u32, u32, u32, u32, &'a mut u32 );
    fn next(&mut self) -> Option<Self::Item> {
    	None
    }
}
*/
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
	pub fn for_pixel_in_block( &mut self, pos_x: u32, pos_y: u32, width: u32, height: u32, mut func: impl FnMut( u32, u32, u32, u32, &mut u32 ) ) {
		for y in 0..height {
			let py = y + pos_y;
			if py >= self.height as u32 { continue; }
			for x in 0..width {
				let px = x + pos_x;
				if px >= self.width as u32 { continue; }

//				dbg!(&px, &py);

				let o = ( py * self.width as u32 + px ) as usize;
				let p = &mut self.buffer[ o ];
				func( px, py, x, y, p );
			}
		}
	}
/*
	pub fn enumerate_pixel_in_block_mut( &mut self, x: u32, y: u32, width: u32, height: u32 ) -> PixelInBlockMut {
		PixelInBlockMut {
			block_x: 0,
			block_y: 0,
			render_buffer: self,
		}
	}
*/
}
