use std::path::PathBuf;

use async_trait::async_trait;
use oml_audio::fileloader::FileLoaderDisk;

use crate::context::Context;
use crate::element::{Element, ElementConfig};
use crate::render_buffer::RenderBuffer;
use crate::render_context::RenderContext;

#[derive(Debug)]
pub struct SoundbankElement {
	name:           String,
	config_path:    PathBuf,
	soundbank_file: Option<String>,
}

impl SoundbankElement {}

#[async_trait]
impl Element for SoundbankElement {
	fn configure(&mut self, config: &ElementConfig) {
		self.config_path = config.config_path().clone();
		let filename = config.get_string_or("soundbank_file", "");
		//		let filename  = config.get_path_or( "soundbank_file", "" );
		if filename != "" {
			self.soundbank_file = Some(filename);
		}
	}

	async fn run(&mut self) -> anyhow::Result<()> {
		Ok(())
	}

	fn update(&mut self, context: &mut Context) {
		if let Some(soundbank_file) = &self.soundbank_file {
			let soundbank = &mut context.get_soundbank_mut();
			let mut fileloader = FileLoaderDisk::new(&self.config_path.to_string_lossy());
			fileloader.enable_debug();

			soundbank.enable_debug();
			soundbank.load(&mut fileloader, soundbank_file);
			//			dbg!(&soundbank);
			//			todo!("...");

			self.soundbank_file = None;
		}
	}

	fn render(&self, _render_buffer: &mut RenderBuffer, _render_context: &mut RenderContext) {}
	fn name(&self) -> &str {
		&self.name
	}
	fn set_name(&mut self, name: &str) {
		self.name = name.to_string();
	}

	fn element_type(&self) -> &str {
		"soundbank"
	}
}

pub struct SoundbankElementFactory {}

impl SoundbankElementFactory {
	pub fn create() -> SoundbankElement {
		SoundbankElement {
			name:           "".to_string(),
			config_path:    PathBuf::new(),
			soundbank_file: None,
		}
	}
}
