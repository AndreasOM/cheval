
//#[derive(Debug,Copy,Clone)]
#[derive(PartialEq,Eq)]
pub struct Pixel {
	color: u32,
}

impl std::fmt::Debug for Pixel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pixel")
         .field( "color", &format!( "{:#08x}", &self.color ) )
         .finish()
    }
}

impl Into<Pixel> for u32 {
    fn into(self) -> Pixel {
    	Pixel {
    		color: self,
    	}
    }
}

impl Into<u32> for Pixel {
    fn into(self) -> u32 {
    	self.color
    }
}

impl Pixel {
	pub fn from_u32( color: u32 ) -> Pixel {
		Pixel {
			color: color,
		}
	}

	pub fn to_u32( &self ) -> u32 {
		self.color
	}

	pub fn apply_alpha( &mut self, alpha: f32 ) {
		let a = ( ( self.color >> 24 )&0x000000ff ) as f32;
		let r = ( ( self.color >> 16 )&0x000000ff ) as f32;
		let g = ( ( self.color >>  8 )&0x000000ff ) as f32;
		let b = ( ( self.color >>  0 )&0x000000ff ) as f32;

		let a = ( a * alpha ) as u32;
		let r = ( r * alpha ) as u32;
		let g = ( g * alpha ) as u32;
		let b = ( b * alpha ) as u32;

		self.color = ( a << 24 )|( r << 16 )|( g << 8 )|b;
	}

	pub fn blend_with_alpha( n: &Pixel, o: &Pixel ) -> Pixel {
//		dbg!(n, o);
		let a = n.color;
		let b = o.color;

		let aa = ( ( a >> 24 )&0x000000ff ) as u8;
//		let aa = 0xff;
		let ra = ( ( a >> 16 )&0x000000ff ) as u8;
		let ga = ( ( a >>  8 )&0x000000ff ) as u8;
		let ba = ( ( a >>  0 )&0x000000ff ) as u8;

		let ab = ( ( b >> 24 )&0x000000ff ) as u8;
//		let ab = 0xff;
		let rb = ( ( b >> 16 )&0x000000ff ) as u8;
		let gb = ( ( b >>  8 )&0x000000ff ) as u8;
		let bb = ( ( b >>  0 )&0x000000ff ) as u8;

//		dbg!( aa, ab );
//		let a = Pixel::mix_byte( aa, ab, aa ) as u32;
//		let a = ( aa + ab ) as u32;	// no blending of alpha, we "overlay"/add alpha
		let a = aa.saturating_add( ab ) as u32;	// no blending of alpha, we "overlay"/add alpha
		let r = Pixel::mix_byte( ra, rb, aa ) as u32;
		let g = Pixel::mix_byte( ga, gb, aa ) as u32;
		let b = Pixel::mix_byte( ba, bb, aa ) as u32;

//		let a = 0x80;
		let argb = ( a << 24 )|( r << 16 )|( g << 8 )|b;

		Pixel {
			color: argb,
			//color: n.color,
		}
	}

	pub fn blend_with_alpha_and_opacity( n: &Pixel, o: &Pixel, op: f32 ) -> Pixel {
//		dbg!(n, o);
		let a = n.color;
		let b = o.color;

		let aa = ( ( a >> 24 )&0x000000ff ) as u8;
//		let aa = 0xff;
		let ra = ( ( a >> 16 )&0x000000ff ) as u8;
		let ga = ( ( a >>  8 )&0x000000ff ) as u8;
		let ba = ( ( a >>  0 )&0x000000ff ) as u8;

		let ab = ( ( b >> 24 )&0x000000ff ) as u8;
//		let ab = 0xff;
		let rb = ( ( b >> 16 )&0x000000ff ) as u8;
		let gb = ( ( b >>  8 )&0x000000ff ) as u8;
		let bb = ( ( b >>  0 )&0x000000ff ) as u8;

//		dbg!( aa, ab );
//		let a = Pixel::mix_byte( aa, ab, aa ) as u32;
//		let a = ( aa + ab ) as u32;	// no blending of alpha, we "overlay"/add alpha
		let aa = ( aa as f32 * op ) as u8;	// :TODO: round?
		let a = aa.saturating_add( ab ) as u32;	// no blending of alpha, we "overlay"/add alpha
		let r = Pixel::mix_byte( ra, rb, aa ) as u32;
		let g = Pixel::mix_byte( ga, gb, aa ) as u32;
		let b = Pixel::mix_byte( ba, bb, aa ) as u32;

//		let a = 0x80;
		let argb = ( a << 24 )|( r << 16 )|( g << 8 )|b;

		Pixel {
			color: argb,
			//color: n.color,
		}
	}

	// :TODO: this needs more thought, but I am too tired
	fn mix_byte( a: u8, b: u8, f: u8 ) -> u8 {
		// rOut = (rA * aA / 255) + (rB * aB * (255 - aA) / (255*255))
		let a = a as u32;
		let b = b as u32;
		let f = f as u32;

//		dbg!( a, b, f );

		let a = a * (       f );
		let b = b * ( 255 - f );

//		dbg!( a, b );

		let ab = ( a + b );// / ( 255 * 255 );

//		dbg!( ab );

//		let ab = ab / ( 255 * 255 );
		let ab = ab / 255;

//		dbg!( ab );

		ab as u8
		/*
		let f = ( f as f32 )/255.0;
		dbg!(f);
		let fa = a as f32 * f;
		let fb = b as f32 * ( 1.0 - f );

		( fa + fb ) as u8
		*/
	}	

}


#[cfg(test)]
mod tests {
    use crate::pixel::Pixel;

    #[test]
    fn mix_byte_works() {
    	// full left/a
    	for j in 0..=255 {
			for i in 0..=255 {
				assert_eq!( i, Pixel::mix_byte( i, j, 0xff ) );
			};
		};
		// full right/b
    	for j in 0..=255 {
			for i in 0..=255 {
				assert_eq!( j, Pixel::mix_byte( i, j, 0x00 ) );
			};
		};

		for i in 0..=255 {
			assert_eq!( i, Pixel::mix_byte( 0xff, 0x00, i ) );
		};
		/*
    	assert_eq!( 0x00, Pixel::mix_byte( 0x00, 0x00, 0x00 ) );
    	assert_eq!( 0x00, Pixel::mix_byte( 0x00, 0xff, 0xff ) ); // full a
    	assert_eq!( 0xff, Pixel::mix_byte( 0x00, 0xff, 0x00 ) ); // full b
    	assert_eq!( 0x7f, Pixel::mix_byte( 0x00, 0xff, 0x80 ) ); // 50/50 (well: 50.x/49.9x)
    	*/
    }

    #[test]
    fn apply_alpha_works() {
		let mut p: Pixel = 0xffffffff.into();
		p.apply_alpha( 1.0 );
		assert_eq!( Pixel::from_u32( 0xffffffff ), p );

		let mut p: Pixel = 0xffffffff.into();
		p.apply_alpha( 0.0 );
		assert_eq!( Pixel::from_u32( 0x00000000 ), p );

		let mut p: Pixel = 0xffffffff.into();
		p.apply_alpha( 0.25 );
		assert_eq!( Pixel::from_u32( 0x3f3f3f3f ), p );


		let mut p: Pixel = 0xffffffff.into();
		p.apply_alpha( 0.5 );
		assert_eq!( Pixel::from_u32( 0x7f7f7f7f ), p );


		let mut p: Pixel = 0xffffffff.into();
		p.apply_alpha( 0.75 );
		assert_eq!( Pixel::from_u32( 0xbfbfbfbf ), p );

//        assert_eq!( Pixel::from_u32( 0xffffffff ), Pixel::apply_alpha( &mut( 0xffffffff.into() ), 1.0 ) );
//        assert_eq!( Pixel::from_u32( 0x00000000 ), Pixel::apply_alpha( &( 0xffffffff.into() ), 0.0 ) );
    }

    #[test]
    fn blending_with_alpha_works() {
        assert_eq!( Pixel::from_u32( 0xffffffff ), Pixel::blend_with_alpha( &( 0xffffffff.into() ), &( 0x00000000.into() ) ) );
        assert_eq!( Pixel::from_u32( 0xffffffff ), Pixel::blend_with_alpha( &( 0x00000000.into() ), &( 0xffffffff.into() ) ) );
        assert_eq!( Pixel::from_u32( 0xff000000 ), Pixel::blend_with_alpha( &( 0xff000000.into() ), &( 0x00000000.into() ) ) );
        assert_eq!( Pixel::from_u32( 0xff000000 ), Pixel::blend_with_alpha( &( 0x00000000.into() ), &( 0xff000000.into() ) ) );
        assert_eq!( Pixel::from_u32( 0xffff0000 ), Pixel::blend_with_alpha( &( 0xffff0000.into() ), &( 0x00000000.into() ) ) );
        assert_eq!( Pixel::from_u32( 0xffff0000 ), Pixel::blend_with_alpha( &( 0x00000000.into() ), &( 0xffff0000.into() ) ) );
        assert_eq!( Pixel::from_u32( 0x80808080 ), Pixel::blend_with_alpha( &( 0x80ffffff.into() ), &( 0x00000000.into() ) ) );
        assert_eq!( Pixel::from_u32( 0x80606060 ), Pixel::blend_with_alpha( &( 0x80808080.into() ), &( 0x00404040.into() ) ) );


        assert_eq!( Pixel::from_u32( 0x80800000 ), Pixel::blend_with_alpha( &( 0x80ff0000.into() ), &( 0x00000000.into() ) ) );
        assert_eq!( Pixel::from_u32( 0xffbf3f3f ), Pixel::blend_with_alpha( &( 0x80ff0000.into() ), &( 0xff808080.into() ) ) );


        assert_eq!( Pixel::from_u32( 0x80404040 ), Pixel::blend_with_alpha( &( 0x80808080.into() ), &( 0x00000000.into() ) ) );
    }
}

