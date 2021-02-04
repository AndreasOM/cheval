
#[derive(Debug)]
pub enum Variable {
	EMPTY,
	U32(u32),
	F32(f32),
	STRING(String)
}
