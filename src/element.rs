use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tracing::*;

use crate::bakedexpression::BakedExpression;
use crate::context::Context;
use crate::file_cache::FileCache;
use crate::render_buffer::RenderBuffer;
use crate::render_context::RenderContext;

#[derive(Debug)]
pub enum ElementConfigEntry {
	U32(u32),
	F32(f32),
	STRING(String),
	BOOL(bool),
}

#[derive(Debug)]
pub struct ElementConfig {
	entries:     HashMap<String, ElementConfigEntry>,
	config_path: PathBuf,
}

impl ElementConfig {
	pub fn new(config_path: &Path) -> Self {
		Self {
			entries:     HashMap::new(),
			config_path: PathBuf::from(config_path),
		}
	}

	pub fn set(&mut self, name: &str, value: &str) {
		if value == "true" {
			self.entries
				.insert(name.to_string(), ElementConfigEntry::BOOL(true));
		} else if value == "false" {
			self.entries
				.insert(name.to_string(), ElementConfigEntry::BOOL(false));
		} else if let Ok(v) = value.parse::<u32>() {
			self.entries
				.insert(name.to_string(), ElementConfigEntry::U32(v));
		} else if let Ok(v) = value.parse::<f32>() {
			self.entries
				.insert(name.to_string(), ElementConfigEntry::F32(v));
		} else {
			if let Some(v) = value.strip_prefix("0x") {
				match u32::from_str_radix(v, 16) {
					Ok(v) => self
						.entries
						.insert(name.to_string(), ElementConfigEntry::U32(v)),
					_ => self.entries.insert(
						name.to_string(),
						ElementConfigEntry::STRING(value.to_string()),
					),
				};
			} else {
				self.entries.insert(
					name.to_string(),
					ElementConfigEntry::STRING(value.to_string()),
				);
			}
		}
	}

	pub fn set_u32(&mut self, name: &str, value: u32) {
		self.entries
			.insert(name.to_string(), ElementConfigEntry::U32(value));
	}

	pub fn get_u32_or(&self, name: &str, default: u32) -> u32 {
		match self.entries.get(name) {
			Some(ElementConfigEntry::U32(v)) => *v,
			Some(ElementConfigEntry::F32(v)) => *v as u32,
			_ => default,
		}
	}

	pub fn get_f32_or(&self, name: &str, default: f32) -> f32 {
		match self.entries.get(name) {
			Some(ElementConfigEntry::F32(v)) => *v,
			Some(ElementConfigEntry::U32(v)) => *v as f32,
			_ => default,
		}
	}

	pub fn get_bakedexpression(&self, name: &str, default: &str) -> BakedExpression {
		match self.entries.get(name) {
			Some(ElementConfigEntry::STRING(v)) => BakedExpression::from_str(v),
			Some(ElementConfigEntry::U32(u)) => BakedExpression::from_u32(*u),
			Some(ElementConfigEntry::F32(f)) => BakedExpression::from_f32(*f),
			_ => BakedExpression::from_str(default),
		}
	}

	pub fn get_bakedexpression_empty(&self, name: &str) -> BakedExpression {
		match self.entries.get(name) {
			Some(ElementConfigEntry::STRING(v)) => BakedExpression::from_str(v),
			Some(ElementConfigEntry::U32(u)) => BakedExpression::from_u32(*u),
			Some(ElementConfigEntry::F32(f)) => BakedExpression::from_f32(*f),
			_ => BakedExpression::new(),
		}
	}

	pub fn get_bakedexpression_u32(&self, name: &str, default: u32) -> BakedExpression {
		match self.entries.get(name) {
			Some(ElementConfigEntry::STRING(v)) => BakedExpression::from_str(v),
			Some(ElementConfigEntry::U32(u)) => BakedExpression::from_u32(*u),
			// :TODO: do we want to allow x.0 here
			//			Some( ElementConfigEntry::F32( f ) ) => BakedExpression::from_f32( *f ),
			_ => BakedExpression::from_u32(default),
		}
	}

	pub fn get_bakedexpression_f32(&self, name: &str, default: f32) -> BakedExpression {
		match self.entries.get(name) {
			Some(ElementConfigEntry::STRING(v)) => BakedExpression::from_str(v),
			Some(ElementConfigEntry::U32(u)) => BakedExpression::from_f32(*u as f32),
			Some(ElementConfigEntry::F32(f)) => BakedExpression::from_f32(*f),
			_ => BakedExpression::from_f32(default),
		}
	}

	pub fn get_bakedexpression_string(&self, name: &str, default: &str) -> BakedExpression {
		match self.entries.get(name) {
			Some(ElementConfigEntry::STRING(v)) => BakedExpression::from_str(v),
			Some(ElementConfigEntry::U32(u)) => BakedExpression::from_u32(*u),
			Some(ElementConfigEntry::F32(f)) => BakedExpression::from_f32(*f),
			_ => BakedExpression::from_str(default),
		}
	}
	/*
		pub fn get_variable_or( &self, name: &str, default: &Variable ) -> Variable {
			match self.entries.get( name ) {
				Some( ElementConfigEntry::U32( v ) ) => Variable::from_u32( *v ),
				Some( ElementConfigEntry::STRING( v ) ) => Variable::from_str( v ),
				_ => default.clone(),
			}
		}
	*/
	pub fn get_string_or(&self, name: &str, default: &str) -> String {
		match self.entries.get(name) {
			Some(ElementConfigEntry::STRING(s)) => s.clone(),
			Some(ElementConfigEntry::U32(v)) => format!("{}", v),
			Some(ElementConfigEntry::F32(v)) => format!("{}", v),
			_ => default.to_string(),
		}
	}

	pub fn get_color_or(&self, name: &str, default: u32) -> u32 {
		match self.entries.get(name) {
			Some(ElementConfigEntry::STRING(s)) => match s.parse() {
				Ok(css_color::Rgba {
					red,
					green,
					blue,
					alpha,
				}) => {
					let r = (red * 255.0) as u32;
					let g = (green * 255.0) as u32;
					let b = (blue * 255.0) as u32;
					let a = (alpha * 255.0) as u32;

					a << 24 | r << 16 | g << 8 | b
				},
				Err(_) => default,
			},
			Some(ElementConfigEntry::U32(v)) => *v,
			//			Some( ElementConfigEntry::F32( v ) ) => format!("{}", v),
			_ => default,
		}
	}

	// :TODO: return Path instead of String
	pub fn get_path_or(&self, name: &str, default: &str) -> String {
		let filename = self.get_string_or(name, default);
		let filename = Path::new(&filename);
		debug!("get_path_or -> {:?}", &filename);
		let filename = self.config_path.join(filename);
		debug!("get_path_or {:?} -> {:?}", &self.config_path, &filename);
		let filename = match FileCache::canonicalize(&filename) {
			Ok(f) => f,
			_ => filename, //panic!("File not found {:?}", &filename ),
		};
		debug!("get_path_or -> {:?}", &filename);

		filename.to_string_lossy().to_string()
	}

	pub fn get_bool_or(&self, name: &str, default: bool) -> bool {
		match self.entries.get(name) {
			Some(ElementConfigEntry::BOOL(b)) => *b,
			_ => default,
		}
	}

	pub fn config_path(&self) -> &PathBuf {
		&self.config_path
	}
}

#[async_trait]
pub trait Element {
	fn configure(&mut self, config: &ElementConfig);
	fn shutdown(&mut self) {}
	fn update(&mut self, _context: &mut Context) {}
	// fn render( &self, _buffer: &mut Vec<u32>, _width: usize, _height: usize ) {}
	fn render(&self, _render_buffer: &mut RenderBuffer, _render_context: &mut RenderContext) {}
	async fn run(&mut self) -> anyhow::Result<()>;
	fn name(&self) -> &str;
	fn set_name(&mut self, name: &str);
	fn element_type(&self) -> &str;
}

impl std::fmt::Debug for dyn Element {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		writeln!(
			f,
			"[Trait] Element: {} [{}]",
			self.name(),
			self.element_type()
		)
	}
}
