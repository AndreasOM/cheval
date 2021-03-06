use cfg_aliases::cfg_aliases;

fn main() {
    // Doesn't work :( ... println!("cargo:rustc-cfg=feature=\"with-minifb\"");
	cfg_aliases! {
        // Platforms
        wasm: { target_arch = "wasm32" },
        android: { target_os = "android" },
        macos: { target_os = "macos" },
        linux: { target_os = "linux" },
        // window providers
        minifb: { all(macos, not(wasm)) },
        framebuffer: { all(linux, not(wasm)) },
    }
}
