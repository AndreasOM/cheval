//use window::Window;

use cheval::cheval::Cheval;
use clap::{App, Arg};
use crate::window::WindowFactory;
//use crate::window::WindowTrait;
use cheval::render_buffer::RenderBuffer;

fn render_frame( render_buffer: &mut RenderBuffer, cheval: &Cheval )
{
	for y in 0..render_buffer.height {
		for x in 0..render_buffer.width {
			let o = y * render_buffer.width + x;
			render_buffer.buffer[ o ] = 0x00000000;
		}
	}

	cheval.render( render_buffer );
}

#[tokio::main]
async fn main() -> Result<(),Box<dyn std::error::Error>> {

	let matches = App::new("cheval")
						.version("0.1")
						.author("Andreas N. <andreas@omni-mad.com>")
						.arg( Arg::with_name("config")
							.long("config")
							.short("c")
							.value_name("CONFIG")
							.help("Set the config file to load.")
							.takes_value(true)
						)
						.arg( Arg::with_name("window-type")
							.long("window-type")
							.short("w")
							.value_name("WINDOW-TYPE")
							.help("Set the window type to use.")
							.takes_value(true)
						)
						.arg( Arg::with_name("frames")
							.long("frames")
							.short("f")
							.value_name("FRAMES")
							.help("Set the number of frames to render.")
							.takes_value(true)
						)
						.get_matches();

	let config = matches.value_of("config").unwrap_or("example_config.yaml").to_string();
	let window_type = matches.value_of("window-type").unwrap_or(&WindowFactory::get_default_window_type()).to_string();
	let frames = matches.value_of("frames").unwrap_or("0").to_string();

	let frames = match frames.parse::<u32>() {
		Ok( frames ) => frames,
		Err( _ ) => panic!("Invalid frames {:?}", frames ),
	};

	dbg!(&config);
	dbg!(&window_type);

	let mut window = WindowFactory::create( &window_type );

	let mut cheval = Cheval::new();

	cheval.load( &config ).await?;

	dbg!( &cheval );
	let mut frame_count = 0;
	while !window.done() {
		cheval.update();
		window.render_frame( &mut render_frame, &cheval );
		window.next_frame();
		frame_count += 1;
		if frames > 0 && frame_count >= frames {
			break;
		}
	}

	cheval.shutdown();

	Ok(())
}

mod window;
