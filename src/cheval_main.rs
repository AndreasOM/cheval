//use window::Window;

use cheval::cheval::Cheval;
use cheval::render_buffer::RenderBuffer;
use cheval::window::WindowFactory;
use cheval::window::WindowMode;
use clap::{App, Arg};
#[cfg(all(feature = "with_termion"))]
use termion::{input::TermRead, raw::IntoRawMode};
use tracing::*;
use tracing_subscriber::FmtSubscriber;

fn render_frame(render_buffer: &mut RenderBuffer, cheval: &mut Cheval) {
	/*
		for y in 0..render_buffer.height {
			for x in 0..render_buffer.width {
				let o = y * render_buffer.width + x;
				render_buffer.buffer[ o ] = 0x00000000;
			}
		}
	*/
	let size = render_buffer.width * render_buffer.height;
	for p in &mut render_buffer.buffer[0..size] {
		*p = 0x00000000;
	}

	/*
		unsafe
		{
			let p = render_buffer.buffer.as_mut_ptr();

			// Initialize elements via raw pointer writes, then set length.
			let size = render_buffer.width * render_buffer.height;
			for i in 0..size {
				*p.add(i) = 0 as u32;
			}
	}
	*/

	cheval.render(render_buffer);
}

#[tokio::main]
//#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let subscriber = FmtSubscriber::builder()
		// all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
		// will be written to stdout.
		.with_max_level(Level::TRACE)
		// completes the builder.
		.finish();

	tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

	const VERSION: &'static str = env!("CARGO_PKG_VERSION");
	let version = VERSION.to_string(); //format!("{}",VERSION);

	let matches = App::new("cheval")
		.version(version.as_ref())
		.author("Andreas N. <andreas@omni-mad.com>")
		.arg(
			Arg::with_name("config")
				.long("config")
				.short("c")
				.value_name("CONFIG")
				.help("Set the config file to load.")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("window-title")
				.long("window-title")
				.short("t")
				.value_name("WINDOW-TITLE")
				.help("Set the window title (prefix) to use.")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("window-type")
				.long("window-type")
				.short("w")
				.value_name("WINDOW-TYPE")
				.help("Set the window type to use.")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("window-mode")
				.long("window-mode")
				.value_name("WINDOW-MODE")
				.help("Set the window mode to use.")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("window-layout")
				.long("window-layout")
				.value_name("WINDOW-LAYOUT")
				.help("Set the file to keep the window layout.")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("frames")
				.long("frames")
				.short("f")
				.value_name("FRAMES")
				.help("Set the number of frames to render.")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("scaling")
				.long("scaling")
				.short("s")
				.value_name("SCALING")
				.help("Set the scaling for the rendering.")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("enable-http")
				.long("enable-http")
				.help("Enable HTTP api.")
				.takes_value(false),
		)
		.get_matches();

	let config = matches.value_of("config").unwrap_or(".").to_string();
	let window_title = matches.value_of("window-title").unwrap_or("").to_string();
	let window_type = matches
		.value_of("window-type")
		.unwrap_or(&WindowFactory::get_default_window_type())
		.to_string();
	let window_mode = matches.value_of("window-mode").unwrap_or("RGB").to_string();
	let window_layout = matches.value_of("window-layout").unwrap_or("").to_string();
	let frames = matches.value_of("frames").unwrap_or("0").to_string();
	let enable_http = matches.occurrences_of("enable-http") > 0;

	let frames = match frames.parse::<u32>() {
		Ok(frames) => frames,
		Err(_) => panic!("Invalid frames {:?}", frames),
	};

	let scaling = matches.value_of("scaling").unwrap_or("1").to_string();

	let scaling = match scaling.parse::<f32>() {
		Ok(scaling) => scaling,
		Err(_) => panic!("Invalid scaling {:?}", scaling),
	};

	let window_mode: &str = &window_mode;
	let window_mode: WindowMode = window_mode.into();
	debug!("config:        {}", &config);
	debug!("window_title:  {}", &window_title);
	debug!("window_type:   {}", &window_type);
	debug!("window_mode:   {}", &window_mode);
	debug!("window_layout: {}", &window_layout);
	debug!("enable_http:   {}", &enable_http);

	let mut window = WindowFactory::create(&window_title, &window_type, &window_mode, scaling);

	if window_layout.len() > 0 {
		window.restore_positions(&window_layout);
	}

	let mut cheval = Cheval::new();

	if enable_http {
		cheval.enable_http();
	}

	cheval.load(&config).await?;
	cheval.initialize()?;

	debug!("Initialized cheval: {:#?}", &cheval);
	let mut frame_count = 0;
	/* :TODO: hide behind feature flag
		let guard = pprof::ProfilerGuard::new(100).unwrap();
	*/
	//	let mut stdin = std::io::stdin().into_raw_mode().unwrap();

	#[cfg(all(feature = "with_termion"))]
	let mut stdout = std::io::stdout().into_raw_mode().unwrap();
	// Use asynchronous stdin
	#[cfg(all(feature = "with_termion"))]
	let mut stdin = termion::async_stdin().keys();

	cheval.run()?;

	while !window.done() && !cheval.done() {
		while let Some(key) = window.get_key() {
			cheval.add_key(key);
		}
		#[cfg(all(feature = "with_termion"))]
		for c in stdin.next() {
			match c {
				Ok(termion::event::Key::Esc) => cheval.add_key(27), // ASCII ESC
				Ok(termion::event::Key::Left) => cheval.add_key(63234),
				Ok(termion::event::Key::Right) => cheval.add_key(63235),
				Ok(termion::event::Key::Char(c)) => cheval.add_key(c as u32),
				_ => {},
			}
		}
		cheval.update();
		window.render_frame(&mut render_frame, &mut cheval);
		window.next_frame();
		frame_count += 1;
		if frames > 0 && frame_count >= frames {
			break;
		}
	}

	if window_layout.len() > 0 {
		window.store_positions(&window_layout);
	}
	/* :TODO: hide behind feature flag
		if let Ok(report) = guard.report().build() {
			println!("report: {}", &report);
			let file = File::create("flamegraph.svg").unwrap();
			report.flamegraph(file).unwrap();
		};
	*/
	cheval.shutdown()?;

	Ok(())
}

//mod window;
