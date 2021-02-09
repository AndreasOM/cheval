

#[derive(Clone,Debug)]
pub enum Original {
	EMPTY,
	U32(u32),
	F32(f32),
	STRING(String)
}

#[derive(Debug)]
pub struct Variable {
	original: Original,
//	baked: Baked
}

impl Variable {

	pub fn new() -> Self {
		Self {
			original: Original::EMPTY,
		}
	}
	pub fn from_u32( v: u32 ) -> Self {
		Self {
			original: Original::U32( v ),
		}
	}
	pub fn from_str( v: &str ) -> Self {
		Self {
			original: Original::STRING( v.to_owned() ),
		}
	}

	// :HACK:
	pub fn original(&self) -> Original {
		self.original.clone()
	}
}
