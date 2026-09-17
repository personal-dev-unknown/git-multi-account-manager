// build.zig  — Root Zig build file
//
// Purpose: Provides a single entry point to build the ENTIRE project
// (Zig native layer + trigger Rust compilation) from one command.
// Running `zig build` from the workspace root:
//   1. Compiles zig_native/ into libgm_native.a
//   2. Runs `cargo build --workspace --release` as a build step
//
// Why this exists: Without this, developers must remember to run
// `cd zig_native && zig build` before every `cargo build`. Forgetting
// this produces a confusing linker error deep inside the Rust build.
// This file makes the correct order automatic and impossible to skip.
//
// For CI, both build systems run explicitly in separate steps so that
// errors from each are clearly attributed. This file is for local dev.

const std = @import("std");

pub fn build(b: *std.Build) void {
    const target   = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // ── Step 1: Build the Zig native library ──────────────────────────────
    // Delegate to the zig_native/ sub-project. This runs zig_native/build.zig
    // and produces zig_native/zig-out/lib/libgm_native.a
    const zig_native = b.dependency("zig_native", .{
        .target   = target,
        .optimize = optimize,
    });

    // Expose the compiled artifact so downstream steps can depend on it
    const native_lib = zig_native.artifact("gm_native");
    b.installArtifact(native_lib);

    // ── Step 2: Run Rust compilation ──────────────────────────────────────
    // cargo build --workspace --release is run after the Zig library is ready.
    // The GIT_MANAGER_ZIG_LIB env var tells gm_adapters/build.rs where to
    // find libgm_native.a without hardcoding the path.
    const cargo_build = b.addSystemCommand(&.{
        "cargo", "build", "--workspace", "--release",
    });
    cargo_build.setEnvironmentVariable(
        "GIT_MANAGER_ZIG_LIB",
        b.getInstallPath(.lib, ""),
    );
    // Rust build depends on the Zig library being installed first
    cargo_build.step.dependOn(b.getInstallStep());

    const build_step = b.step("all", "Build Zig native layer and all Rust crates");
    build_step.dependOn(&cargo_build.step);

    // ── Step 3: Test step ─────────────────────────────────────────────────
    // `zig build test` runs both Zig unit tests and Rust tests in sequence.
    const zig_tests = b.addTest(.{
        .root_source_file = .{ .path = "zig_native/src/root.zig" },
        .target           = target,
        .optimize         = optimize,
    });
    const run_zig_tests = b.addRunArtifact(zig_tests);

    const cargo_test = b.addSystemCommand(&.{
        "cargo", "test", "--workspace",
    });
    cargo_test.step.dependOn(&run_zig_tests.step);

    const test_step = b.step("test", "Run all Zig and Rust tests");
    test_step.dependOn(&cargo_test.step);
}