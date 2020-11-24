
use crate::context::Context;
use crate::element::{Element,ElementConfig};
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;

#[derive(Debug)]
pub struct ElementInstance {
	element: Box< dyn Element >,
	is_visible: bool,
}


impl ElementInstance {
	pub fn new(
		element: Box< dyn Element>
	) -> Self {
		Self {
			element: element,
			is_visible: true,
		}
	}

	pub fn name( &self ) -> &str {
		self.element.name()
	}

	pub async fn run( &mut self ) -> anyhow::Result<()> {
		self.element.run().await
	}

	pub fn update( &mut self, context: &mut Context ) {
		self.element.update( context )
	}

	pub fn render(
		&self,
		render_buffer: &mut RenderBuffer,
		render_context: &mut RenderContext
	) {
		self.element.render( render_buffer, render_context )
	}

	pub fn shutdown( &mut self ) {
		self.element.shutdown()
	}

	pub fn is_visible( &self ) -> bool {
		self.is_visible
	}

	pub fn hide( &mut self ) {
		self.is_visible = false;
	} 

	pub fn show( &mut self ) {
		self.is_visible = true;
	} 
}
