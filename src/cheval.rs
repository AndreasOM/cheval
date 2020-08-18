//use window::Window;


fn render_frame( buffer: &mut Vec<u32>, width: usize, height: usize )
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
}

fn main() {

	let mut window = Window::new();

	while !window.done() {
		window.render_frame( &mut render_frame );
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
