use std::collections::VecDeque;

use tracing::*;

use crate::bakedexpression::BakedExpression;
use crate::context::Context;
use crate::element::ElementConfig;
use crate::element_instance::ElementInstance;
use crate::render_buffer::RenderBuffer;
use crate::render_context::RenderContext;

#[derive(Debug)]
enum Visibility {
	Hidden,
	Visible,
}

#[derive(Debug)]
pub struct Page {
	name:              String,
	element_instances: Vec<ElementInstance>,
	sound_on_show:     BakedExpression,
	visibility:        Visibility,
	sound_queue:       VecDeque<String>,
}

impl Page {
	pub fn new() -> Self {
		Self {
			name:              String::new(),
			element_instances: Vec::new(),
			sound_on_show:     BakedExpression::new(),
			visibility:        Visibility::Hidden,
			sound_queue:       VecDeque::new(),
		}
	}

	pub fn configure(&mut self, config: &ElementConfig) {
		self.sound_on_show = config.get_bakedexpression_string("sound_on_show", "");
	}

	pub fn add_element_instance(&mut self, element_instance: ElementInstance) {
		self.element_instances.push(element_instance);
	}

	pub fn run_for_element_instance_with_name(
		&mut self,
		name: &str,
		func: &Box<dyn Fn(&mut ElementInstance)>,
	) {
		for e in &mut self.element_instances {
			if e.name() == name {
				func(e);
			}
		}
	}

	pub fn shutdown(&mut self) {
		for e in self.element_instances.iter_mut() {
			e.shutdown();
		}
	}

	pub fn update(&mut self, context: &mut Context) {
		self.sound_on_show.bake_string_or(context, "");
		for e in &mut self.element_instances {
			e.update(context);
		}
		while let Some(sound) = self.sound_queue.pop_front() {
			//			println!("sound: {}", &sound);
			context.play_sound(&sound);
		}
	}
	pub fn render(&self, render_buffer: &mut RenderBuffer, render_context: &mut RenderContext) {
		if self.is_visible() {
			for e in &self.element_instances {
				if e.is_visible() {
					e.render(render_buffer, render_context);
				}
			}
		}
	}

	pub async fn run(&mut self) -> anyhow::Result<()> {
		for e in self.element_instances.iter_mut() {
			e.run().await?;
		}
		Ok(())
	}

	pub fn is_visible(&self) -> bool {
		match self.visibility {
			Visibility::Visible => true,
			_ => false,
		}
	}

	pub fn set_name(&mut self, name: &str) {
		self.name = name.to_string();
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn hide(&mut self) {
		self.visibility = Visibility::Hidden;
	}

	pub fn show(&mut self) {
		debug!("Sound on show: {:?}", &self.sound_on_show);
		let sound_on_show = self.sound_on_show.as_string();
		if sound_on_show.len() > 0 {
			self.sound_queue.push_back(sound_on_show.to_string());
		}
		self.visibility = Visibility::Visible;
	}
}
