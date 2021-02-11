use crate::context::Context;
use crate::element_instance::ElementInstance;
use crate::render_context::RenderContext;
use crate::render_buffer::RenderBuffer;

#[derive(Debug)]
pub struct Page {
	element_instances: Vec< ElementInstance >,
}

impl Page {

	pub fn new() -> Self {
		Self {
			element_instances: Vec::new(),
		}
	}

	pub fn add_element_instance( &mut self, element_instance: ElementInstance ) {
		self.element_instances.push( element_instance );
	}

	pub fn run_for_element_instance_with_name(
		&mut self,
		name: &str,
		func: &Box< dyn Fn( &mut ElementInstance ) >,
	) {
		for e in &mut self.element_instances {
			if e.name() == name {
				func( e );
			}
		}
	}

	pub fn shutdown( &mut self ) {
		for e in self.element_instances.iter_mut() {
			e.shutdown();
		}
	}

	pub fn update( &mut self, context: &mut Context ) {
		for e in &mut self.element_instances {
			e.update( context );
		}
	}
	pub fn render( &self, render_buffer: &mut RenderBuffer, render_context: &mut RenderContext ) {
		for e in &self.element_instances {
			if e.is_visible() {
				e.render( render_buffer, render_context );
			}
		};		
	}

	pub async fn run( &mut self ) -> anyhow::Result<()> {
		for e in self.element_instances.iter_mut() {
			e.run().await?;
		}
		Ok(())	
	}

}
