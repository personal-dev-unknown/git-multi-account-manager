{
  description = "git-multi-account-manager dev environment (Rust + Zig native layer)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    zig-overlay.url = "github:mitchellh/zig-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, zig-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        zigpkgs = zig-overlay.packages.${system};

        rustToolchain = pkgs.rust-bin.stable.latest.default;
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain

            # Pinned older than the global `zig` (0.16.0) in languages.nix —
            # this codebase uses APIs (GeneralPurposeAllocator, std.posix.getenv,
            # std.fs.makeDirAbsolute) that 0.16.0 removed. Check
            # zig_native/build.zig.zon for a declared minimum_zig_version and
            # adjust this pin if it names something other than 0.15.x.
            zigpkgs."0.15.2"

            # GTK/GLib dev libraries — fixes the glib-sys / gobject-sys /
            # gdk-pixbuf-sys / pango-sys pkg-config failures.
            pkgs.pkg-config
            pkgs.glib
            pkgs.gtk3

            # linux_keyring.zig needs this header (libsecret/secret.h).
            pkgs.libsecret

            # For testing this app's own ssh-keygen / ssh-agent interaction —
            # scoped here since it's specific to what this project does, not
            # a general everyday need.
            pkgs.openssh
          ];

          shellHook = ''
            echo "git-multi-account-manager dev shell loaded"
            echo "  $(rustc --version)"
            echo "  $(zig version)"
          '';
        };
      });
}
