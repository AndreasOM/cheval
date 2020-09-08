//use window::Window;

use cheval::cheval::Cheval;
use clap::{App, Arg};
use crate::window::WindowFactory;
//use crate::window::WindowTrait;

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
						.get_matches();

	let config = matches.value_of("config").unwrap_or("example_config.yaml").to_string();
	let window_type = matches.value_of("window-type").unwrap_or(&WindowFactory::get_default_window_type()).to_string();

	dbg!(&config);
	dbg!(&window_type);

	let mut window = WindowFactory::create( &window_type );

	let mut cheval = Cheval::new();

	cheval.load( &config ).await?;

	dbg!( &cheval );
	while !window.done() {
		cheval.update();
		window.render_frame( &mut render_frame, &cheval );
		window.next_frame();
	}

	cheval.shutdown();

	Ok(())
}

mod window;
