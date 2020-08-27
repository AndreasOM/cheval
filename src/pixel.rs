
pub struct Pixel {
	color: u32,
}

// :TODO: create conversion traits

impl Pixel {
	pub fn from_u32( color: u32 ) -> Pixel {
		Pixel {
			color: color,
		}
	}

	pub fn to_u32( &self ) -> u32 {
		self.color
	}

	pub fn blend_with_alpha( a: Pixel, b: Pixel ) -> Pixel {
		let a = a.color;
		let b = b.color;

		let aa = ( ( a >> 24 )&0x000000ff ) as u8;
		let ra = ( ( a >> 16 )&0x000000ff ) as u8;
		let ga = ( ( a >>  8 )&0x000000ff ) as u8;
		let ba = ( ( a >>  0 )&0x000000ff ) as u8;

//		let ab = ( ( b >> 24 )&0x000000ff ) as u8;
		let rb = ( ( b >> 16 )&0x000000ff ) as u8;
		let gb = ( ( b >>  8 )&0x000000ff ) as u8;
		let bb = ( ( b >>  0 )&0x000000ff ) as u8;

		let r = Pixel::mix_byte( ra, rb, aa ) as u32;
		let g = Pixel::mix_byte( ga, gb, aa ) as u32;
		let b = Pixel::mix_byte( ba, bb, aa ) as u32;

		let argb = ( a << 24 )|( r << 16 )|( g << 8 )|b;

		Pixel {
			color: argb,
		}
	}

	pub fn blend_with_alpha_and_opacity( a: Pixel, b: Pixel, o: f32 ) -> Pixel {
		let a = a.color;
		let b = b.color;

		let aa = ( ( a >> 24 )&0x000000ff ) as u8;
		let ra = ( ( a >> 16 )&0x000000ff ) as u8;
		let ga = ( ( a >>  8 )&0x000000ff ) as u8;
		let ba = ( ( a >>  0 )&0x000000ff ) as u8;

//		let ab = ( ( b >> 24 )&0x000000ff ) as u8;
		let rb = ( ( b >> 16 )&0x000000ff ) as u8;
		let gb = ( ( b >>  8 )&0x000000ff ) as u8;
		let bb = ( ( b >>  0 )&0x000000ff ) as u8;

		let aa = ( aa as f32 * o ) as u8;
		let r = Pixel::mix_byte( ra, rb, aa ) as u32;
		let g = Pixel::mix_byte( ga, gb, aa ) as u32;
		let b = Pixel::mix_byte( ba, bb, aa ) as u32;

		let argb = ( a << 24 )|( r << 16 )|( g << 8 )|b;

		Pixel {
			color: argb,
		}
	}

	fn mix_byte( a: u8, b: u8, f: u8 ) -> u8 {
		let f = ( f as f32 )/255.0;
		let fa = a as f32 * f;
		let fb = b as f32 * ( 1.0 - f );

		( fa + fb ) as u8
	}	

}
