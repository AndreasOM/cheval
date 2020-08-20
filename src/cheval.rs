use crate::element::Element;

#[derive(Debug)]
pub struct Cheval {
	elements: Vec< Box< dyn Element > >,
}

impl Cheval {
	pub fn new() -> Self {
		Self {
			elements: Vec::new(),
		}
	}

	pub fn add_element( &mut self, element: Box< dyn Element > ) {
		self.elements.push( element );
	}

	pub fn render( &self, buffer: &mut Vec<u32>, width: usize, height: usize ) {
		for e in &self.elements {
//			dbg!(e);
			e.render( buffer, width, height );
		}
	}
}
