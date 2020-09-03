//use window::Window;

use cheval::cheval::Cheval;
use cheval::block_element::{BlockElement,BlockElementFactory};
use cheval::element::{Element,ElementConfig};

fn render_frame( buffer: &mut Vec<u32>, width: usize, height: usize, cheval: &Cheval )
{
	for y in 0..height {
		for x in 0..width {
			let o = y * width + x;
			buffer[ o ] = 0x00000000;
		}
	}

	cheval.render( buffer, width, height );
}

#[tokio::main]
async fn main() -> Result<(),Box<dyn std::error::Error>> {

	let mut window = Window::new();

	let mut cheval = Cheval::new();

	cheval.load( "example_config.yaml" ).await?;

	dbg!( &cheval );
	while !window.done() {
		cheval.update();
		window.render_frame( &mut render_frame, &cheval );
		window.next_frame();
	}

	cheval.shutdown();

	Ok(())
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
