// crates/gm_interface_desktop/build.rs
//
// The Tauri build script. This must run before the main crate compilation
// so that Tauri can inject its bundle metadata (app name, version, icons, etc.)
// into the compiled binary. Required by every Tauri application.

fn main() {
    tauri_build::build()
}