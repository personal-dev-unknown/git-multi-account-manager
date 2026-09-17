// zig_native/build.zig
//
// ── Purpose ──────────────────────────────────────────────────────────────────
// Compiles the entire Zig native layer into a single C-compatible static
// library: libgm_native.a. This library is linked by gm_adapters at Rust
// compile time via gm_adapters/build.rs.
//
// ── Outputs ───────────────────────────────────────────────────────────────────
//   zig-out/lib/libgm_native.a     ← what Rust links against
//   zig-out/include/gm_native.h    ← C header auto-generated from Zig exports
//
// ── Optimization strategy ─────────────────────────────────────────────────────
// Default: ReleaseSafe
//   - Keeps bounds checking, integer overflow detection, null pointer checks
//   - Still fast for production (comparable to -O2 in C)
//   - Catches bugs in OS integration code that are hard to spot in tests
//
// Production override: `zig build -Doptimize=ReleaseFast`
//   - Removes safety checks for maximum performance
//   - Only use after thorough testing with ReleaseSafe
//
// ── Platform-specific system libraries ────────────────────────────────────────
// Linux:  libsecret-1  (GNOME libsecret D-Bus API for credential storage)
// macOS:  Security.framework (Keychain access)
// Both platforms also need libc for POSIX syscalls (chmod, rename, etc.)
//
// ── Cross-compilation ─────────────────────────────────────────────────────────
// `zig build -Dtarget=aarch64-linux` produces an ARM64 Linux library
// that Rust can link against when cross-compiling for that target.
// The platform detection in platform/index.zig uses comptime so the
// correct backend is baked in at Zig compile time, not selected at runtime.

const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{ .preferred_optimize_mode = .ReleaseSafe });

    const os_tag = target.result.os.tag;

    // ─── Root module ───────────────────────────────────────────────────────
    // Zig 0.16.0-dev requires a *Module for addLibrary.
    //
    // IMPORTANT: we use addIncludePath instead of linkSystemLibrary for the
    // platform credential libraries.  When linkSystemLibrary is called on a
    // module that becomes a static archive, Zig 0.16.0-dev bundles the .so
    // as an archive member, which lld rejects.  The Rust build.rs already
    // emits `cargo:rustc-link-lib=dylib=secret-1` (Linux) / `framework=Security`
    // (macOS) so the system library is resolved at the final Rust link step.
    // We only need the C headers here so @cImport can resolve types/functions.
    const lib_mod = b.createModule(.{
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
        .link_libc = true,
    });
    if (os_tag == .linux) {
        lib_mod.addIncludePath(.{ .cwd_relative = "/usr/include/libsecret-1" });
        lib_mod.addIncludePath(.{ .cwd_relative = "/usr/include/glib-2.0" });
        lib_mod.addIncludePath(.{ .cwd_relative = "/usr/lib/x86_64-linux-gnu/glib-2.0/include" });
    }
    // macOS: Security.framework headers are in the SDK sysroot; Zig finds
    // them automatically via the SDK path — no addIncludePath needed.

    // Disable Zig's stack-clash probing for the FFI boundary. Stack checking
    // emits calls to __zig_probe_stack which is an internal Zig runtime symbol
    // that rust-lld cannot resolve when linking the final binary.
    lib_mod.stack_check = false;

    // ─── Static library ────────────────────────────────────────────────────
    // use_llvm = true: force the LLVM codegen backend so `export fn` symbols
    // are emitted as proper ELF relocatable objects inside the archive.
    // Without this, Zig 0.16.0-dev's self-hosted incremental backend produces
    // a ZCU object with only debug sections and no machine code, leaving all
    // gm_* symbols undefined when Rust links the final binary.
    const lib = b.addLibrary(.{
        .name = "gm_native",
        .linkage = .static,
        .root_module = lib_mod,
        .use_llvm = true,
    });

    // Install the library artifact into zig-out/lib/
    b.installArtifact(lib);

    // ─── C header emission ────────────────────────────────────────────────
    const install_header = b.addInstallHeaderFile(
        b.path("include/gm_native.h"),
        "gm_native.h",
    );
    b.getInstallStep().dependOn(&install_header.step);

    // ─── Unit tests ────────────────────────────────────────────────────────
    // Tests produce an executable, so they DO need linkSystemLibrary for the
    // final link step (unlike the static archive above).
    const test_mod = b.createModule(.{
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
        .link_libc = true,
    });
    if (os_tag == .linux) {
        test_mod.addIncludePath(.{ .cwd_relative = "/usr/include/libsecret-1" });
        test_mod.addIncludePath(.{ .cwd_relative = "/usr/include/glib-2.0" });
        test_mod.addIncludePath(.{ .cwd_relative = "/usr/lib/x86_64-linux-gnu/glib-2.0/include" });
        test_mod.linkSystemLibrary("secret-1", .{});
    } else if (os_tag == .macos) {
        test_mod.linkFramework("Security", .{});
    }

    const unit_tests = b.addTest(.{
        .root_module = test_mod,
    });

    const run_unit_tests = b.addRunArtifact(unit_tests);
    const test_step = b.step("test", "Run Zig unit tests for the native layer");
    test_step.dependOn(&run_unit_tests.step);
}
