// crates/gm_adapters/build.rs
//
// This build script runs before any Rust source in gm_adapters is compiled.
// Its sole responsibility is telling the Rust linker where to find the Zig
// static library (libgm_native.a) and which system libraries the Zig code
// itself links against.
//
// The zig_native/ directory is a completely independent build unit. Cargo knows
// nothing about Zig files — it only sees the compiled artifact. By emitting
// cargo:rustc-link-search and cargo:rustc-link-lib directives here, we make
// the Zig layer invisible to the rest of the build system: gm_adapters compiles
// as if libgm_native.a were a regular Rust dependency.
//
// ── Build order guarantee ─────────────────────────────────────────────────────
// This script panics with a human-readable error if libgm_native.a is not
// present. The error message instructs the developer to build Zig first.
// In CI, the Makefile / build_all.sh scripts handle the ordering explicitly.
// There is no automated way for Cargo to trigger a Zig build — the two build
// systems are deliberately isolated.
//
// ── Platform-specific links ───────────────────────────────────────────────────
// The Zig linux_keyring.zig module links against libsecret-1 (GNOME keyring).
// The Zig macos_keychain.zig links against the Security.framework.
// These must be declared here so the final binary links correctly. On Linux,
// libsecret-1-dev must be installed (sudo apt install libsecret-1-dev).

use std::path::PathBuf;

fn main() {
    // CARGO_MANIFEST_DIR is set by Cargo to the directory containing this Cargo.toml.
    // From crates/gm_adapters/, we traverse up two levels to reach the workspace root,
    // then into zig_native/zig-out/lib/ where `zig build` writes the static library.
    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by Cargo")
    );

    let zig_lib_dir = manifest_dir
        .join("..")   // crates/
        .join("..")   // workspace root
        .join("zig_native")
        .join("zig-out")
        .join("lib");

    // canonicalize() resolves the ".." components and returns an absolute path.
    // If this fails, the directory does not exist — the Zig build has not run.
    let zig_lib_dir = zig_lib_dir.canonicalize().unwrap_or_else(|_| {
        panic!(
            "\n\n\
             ┌─────────────────────────────────────────────────────────────────┐\n\
             │  MISSING: zig_native/zig-out/lib/libgm_native.a                │\n\
             │                                                                 │\n\
             │  The Zig native layer must be built before cargo build.        │\n\
             │  Run from the workspace root:                                   │\n\
             │                                                                 │\n\
             │    cd zig_native && zig build --release=safe && cd ..         │\n\
             │                                                                 │\n\
             └─────────────────────────────────────────────────────────────────┘\n\n"
        )
    });

    // Tell the linker where to find the static archive.
    println!("cargo:rustc-link-search=native={}", zig_lib_dir.display());

    // Link the Zig static library into every binary that depends on gm_adapters.
    println!("cargo:rustc-link-lib=static=gm_native");

    // Platform-specific system libraries that the Zig code calls into.
    // These must be declared here — Cargo has no way to know what native
    // libraries a static archive internally references.
    #[cfg(target_os = "linux")]
    {
        // libsecret-1: GNOME keyring. Install with: sudo apt install libsecret-1-dev
        println!("cargo:rustc-link-lib=dylib=secret-1");
        // glib-2.0: GLib core (g_hash_table_*, g_str_hash, g_error_free, etc.)
        // libsecret-1 depends on GLib, but rust-lld needs an explicit directive
        // because the Zig object references GLib symbols directly.
        println!("cargo:rustc-link-lib=dylib=glib-2.0");
        // gio-2.0: GLib IO layer (g_dbus_error_quark, GDBus, GError domains).
        // libsecret-1 uses D-Bus internally; the Zig keyring code triggers these
        // GIO symbols which live in a separate shared library from glib-2.0.
        println!("cargo:rustc-link-lib=dylib=gio-2.0");
    }

    #[cfg(target_os = "macos")]
    {
        // Security.framework: macOS Keychain Access.
        println!("cargo:rustc-link-lib=framework=Security");
    }

    // Tell Cargo to re-run this script when the Zig source or output changes.
    // Without this, modifying a Zig file would not trigger a relink.
    println!("cargo:rerun-if-changed=../../zig_native/src");
    println!("cargo:rerun-if-changed=../../zig_native/zig-out/lib/libgm_native.a");
}