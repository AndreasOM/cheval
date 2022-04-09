// #![recursion_limit = "256"]

use cfg_aliases::cfg_aliases;

fn main() {
	// Doesn't work :( ... println!("cargo:rustc-cfg=feature=\"with-minifb\"");

	cfg_aliases! {
		// Platforms
		wasm: { target_arch = "wasm32" },
		android: { target_os = "android" },
		macos: { target_os = "macos" },
		linux: { target_os = "linux" },
		windows: { target_os = "windows" },
		// window providers
		minifb: {
			any(
				all(macos, feature="minifb", not(wasm)),
				all(linux, feature="minifb", not(wasm)),
				all(windows, feature="minifb", not(wasm))
			)
		},
		framebuffer: { all(linux, feature="framebuffer", not(wasm)) },
	}

	#[cfg(all(feature = "framebuffer", not( target_os = "linux" )))]
	{
		panic!("framebuffer only supported on linux");
	}

	if std::env::var("TARGET").unwrap().contains("-apple") {
		println!("cargo:rustc-link-lib=framework=Foundation");
		println!("cargo:rustc-link-lib=framework=AVFAudio");
	}
}
