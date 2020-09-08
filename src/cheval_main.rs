//use window::Window;

use cheval::cheval::Cheval;
use clap::{App, Arg};
use crate::window::Window;

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
						.get_matches();

	let config = matches.value_of("config").unwrap_or("example_config.yaml").to_string();

	dbg!(&config);


	let mut window = Window::new();

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
