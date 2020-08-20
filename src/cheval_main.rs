//use window::Window;

use cheval::cheval::Cheval;
use cheval::block_element::{BlockElement,BlockElementFactory};
use cheval::element::{Element,ElementConfig};

fn render_frame( buffer: &mut Vec<u32>, width: usize, height: usize, cheval: &Cheval )
{
	for y in 0..height {
		for x in 0..width {
			let o = y * width + x;
			buffer[ o ] = if x<width/2 { 
					if y<height/2 {
						0xff00ff00
					} 
					else
					{
						0xffffffff
					}
				} 
				else
				{ 
					if y<height/2 {
						0xffff0000
					} 
					else
					{
						0xff0000ff
					}
				};
		}
	}

	cheval.render( buffer, width, height );
}

fn main() {

	let mut window = Window::new();

	let mut cheval = Cheval::new();


	let mut block_element = BlockElementFactory::create();
	block_element.set_name( "Green Block" );
	let mut element_config = ElementConfig::new();
	element_config.set_u32( "color", 0xff80f080);
	element_config.set_u32( "x", 0);
	element_config.set_u32( "y", 0);
	element_config.set_u32( "width", 320);
	element_config.set_u32( "height", 180);
	block_element.configure( &element_config );
	cheval.add_element( Box::new( block_element ) );

	let mut block_element = BlockElementFactory::create();
	block_element.set_name( "Red Block" );
	let mut element_config = ElementConfig::new();
	element_config.set_u32( "color", 0xffff8080);
	element_config.set_u32( "x", 320);
	element_config.set_u32( "y", 0);
	element_config.set_u32( "width", 320);
	element_config.set_u32( "height", 180);
	block_element.configure( &element_config );
	cheval.add_element( Box::new( block_element ) );

	let mut block_element = BlockElementFactory::create();
	block_element.set_name( "White Block" );
	let mut element_config = ElementConfig::new();
	element_config.set_u32( "color", 0xfff0f0f0);
	element_config.set_u32( "x", 0);
	element_config.set_u32( "y", 180);
	element_config.set_u32( "width", 320);
	element_config.set_u32( "height", 180);
	block_element.configure( &element_config );
	cheval.add_element( Box::new( block_element ) );

	let mut block_element = BlockElementFactory::create();
	block_element.set_name( "Blue Block" );
	let mut element_config = ElementConfig::new();
	element_config.set_u32( "color", 0xff8080ff);
	element_config.set_u32( "x", 320);
	element_config.set_u32( "y", 180);
	element_config.set_u32( "width", 320);
	element_config.set_u32( "height", 180);
	block_element.configure( &element_config );
	cheval.add_element( Box::new( block_element ) );

	dbg!( &cheval );
	while !window.done() {
		window.render_frame( &mut render_frame, &cheval );
		window.next_frame();
	}
}

// mod window;
#[cfg(target_arch = "x86_64")]
mod window_minifb;
#[cfg(target_arch = "x86_64")]
pub use window_minifb::Window as Window;

#[cfg(target_arch = "arm")]
mod window_framebuffer;
#[cfg(target_arch = "arm")]
pub use window_framebuffer::Window as Window;
