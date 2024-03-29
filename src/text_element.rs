use async_trait::async_trait;
use rusttype::Font;

use crate::axisalignedrectangle::AxisAlignedRectangle;
use crate::bakedexpression::BakedExpression;
use crate::context::Context;
use crate::element::{Element, ElementConfig};
use crate::render_buffer::RenderBuffer;
use crate::render_context::RenderContext;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TextElement {
	name:            String,
	ar:              AxisAlignedRectangle,
	color:           u32,
	text:            BakedExpression,
	fontfile:        String,
	size:            u32,
	font:            Option<Font<'static>>,
	display_text:    String,
	bounding_box:    AxisAlignedRectangle,
	shadow_color:    u32,
	shadow_offset_x: BakedExpression,
	shadow_offset_y: BakedExpression,
	glow_color:      u32,
	glow_size:       BakedExpression,
}

impl TextElement {
	fn _fill_box(
		buffer: &mut Vec<u32>,
		width: usize,
		height: usize,
		x: u32,
		y: u32,
		w: u32,
		h: u32,
		color: u32,
	) {
		for iy in 0..h {
			let py = iy + y;
			if py >= height as u32 {
				continue;
			}
			for ix in 0..w {
				let px = ix + x;
				if px >= width as u32 {
					continue;
				}

				let o = (py * width as u32 + px) as usize;
				buffer[o] = color;
			}
		}
	}
}

#[async_trait]
impl Element for TextElement {
	fn configure(&mut self, config: &ElementConfig) {
		//		self.x      = config.get_u32_or( "pos_x", 0 );
		self.ar.x = config.get_bakedexpression_u32("pos_x", 0);
		self.ar.y = config.get_bakedexpression_u32("pos_y", 0);
		self.ar.width = config.get_bakedexpression_u32("width", 0);
		self.ar.height = config.get_bakedexpression_u32("height", 0);
		self.color = config.get_u32_or("color", 0xffff00ff);
		self.text = config.get_bakedexpression_string("text", "");
		self.fontfile = config.get_path_or("font", "");
		self.size = config.get_u32_or("size", 20);
		self.display_text = config.get_string_or("text", "");
		self.shadow_color = config.get_u32_or("shadow_color", 0xff11ffff);
		self.shadow_offset_x = config.get_bakedexpression_u32("shadow_offset_x", 0);
		self.shadow_offset_y = config.get_bakedexpression_u32("shadow_offset_y", 0);
		self.glow_color = config.get_u32_or("glow_color", 0xffffff11);
		self.glow_size = config.get_bakedexpression_u32("glow_size", 0);

		// NOTE: We could just directly us the self.bounding_box, but want to keep our options open
		let mut bb = AxisAlignedRectangle::new();

		bb.x = config.get_bakedexpression_empty("bounding_box_pos_x");
		bb.y = config.get_bakedexpression_empty("bounding_box_pos_y");
		bb.width = config.get_bakedexpression_empty("bounding_box_width");
		bb.height = config.get_bakedexpression_empty("bounding_box_height");

		self.bounding_box = bb;

		//		dbg!(&self);
	}

	fn shutdown(&mut self) {}

	async fn run(&mut self) -> anyhow::Result<()> {
		Ok(())
	}

	fn update(&mut self, context: &mut Context) {
		//		self.display_text = context.expand_string_or( &self.text, "" );
		// Note: we could just bake ar
		self.ar.x.bake_u32_or(context, 0);
		self.ar.y.bake_u32_or(context, 0);
		self.ar.width.bake_u32_or(context, 0);
		self.ar.height.bake_u32_or(context, 0);
		self.bounding_box.bake_or(context, &self.ar);

		self.text.bake_string_or(context, "");

		self.shadow_offset_x.bake_u32_or(context, 0);
		self.shadow_offset_y.bake_u32_or(context, 0);
		self.glow_size.bake_u32_or(context, 0);
	}

	fn render(&self, render_buffer: &mut RenderBuffer, render_context: &mut RenderContext) {
		/*
				if &self.name == "Banner Title" {
					dbg!(&self);
				}
		*/
		//		dbg!(&self);
		match render_context.use_font(&self.fontfile) {
			// :TODO: handle error
			_ => {},
		}
		// :TODO: kids, don't do glow like this! ever!
		let gs = self.glow_size.as_u32() as i32;
		if gs != 0 {
			for y in -gs..=gs {
				for x in -gs..=gs {
					if !(x == 0 && y == 0) {
						match render_context.draw_text(
							render_buffer,
							&self.text.as_string(),
							(self.ar.x.as_u32() as i32 + x) as u32,
							(self.ar.y.as_u32() as i32 + y) as u32,
							self.ar.width.as_u32(),
							self.ar.height.as_u32(),
							&self.bounding_box,
							self.size, // :TODO: maybe move this to use font
							self.glow_color,
						) {
							// :TODO: handle error
							_ => {},
						}
					}
				}
			}
		}
		let (x, y) = (self.shadow_offset_x.as_u32(), self.shadow_offset_y.as_u32());
		if (x, y) != (0, 0) {
			match render_context.draw_text(
				render_buffer,
				&self.text.as_string(),
				self.ar.x.as_u32() + x,
				self.ar.y.as_u32() + y,
				self.ar.width.as_u32(),
				self.ar.height.as_u32(),
				&self.bounding_box,
				self.size, // :TODO: maybe move this to use font
				self.shadow_color,
			) {
				// :TODO: handle error
				_ => {},
			}
		}
		match render_context.draw_text(
			render_buffer,
			&self.text.as_string(),
			self.ar.x.as_u32(),
			self.ar.y.as_u32(),
			self.ar.width.as_u32(),
			self.ar.height.as_u32(),
			&self.bounding_box,
			self.size, // :TODO: maybe move this to use font
			self.color,
		) {
			// :TODO: handle error
			_ => {},
		}
	}
	fn name(&self) -> &str {
		&self.name
	}
	fn set_name(&mut self, name: &str) {
		self.name = name.to_string();
	}

	fn element_type(&self) -> &str {
		"text"
	}
}

pub struct TextElementFactory {}

impl TextElementFactory {
	pub fn create() -> TextElement {
		TextElement {
			name:            "".to_string(),
			ar:              AxisAlignedRectangle::new(),
			color:           0xff00ffff,
			text:            BakedExpression::from_str(""),
			fontfile:        "".to_string(),
			size:            20,
			font:            None,
			display_text:    "".to_string(),
			bounding_box:    AxisAlignedRectangle::new(),
			shadow_color:    0xff11ffff,
			shadow_offset_x: BakedExpression::from_u32(0),
			shadow_offset_y: BakedExpression::from_u32(0),
			glow_color:      0xffffff11,
			glow_size:       BakedExpression::from_u32(0),
		}
	}
}
