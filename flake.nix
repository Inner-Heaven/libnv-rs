{
  description = "Minimal Rust Development Environment";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    devshell = {
      url = "github:numtide/devshell";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };
  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    devshell,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      cwd = builtins.toString ./.;
      overlays = [devshell.overlays.default rust-overlay.overlays.default];
      pkgs = import nixpkgs {inherit system overlays;};
      rust = pkgs.rust-bin.fromRustupToolchainFile "${cwd}/rust-toolchain.toml";
    in
      with pkgs; {
        devShell = clangStdenv.mkDerivation rec {
          name = "rust";
          nativeBuildInputs = [
            binutils
            cargo-cache
            cargo-deny
            cargo-expand
            cargo-outdated
            cargo-sort
            cargo-sweep
            cargo-wipe
            cargo-release
            cmake
            git-cliff
            gnumake
            pkg-config
            rust
            just
            zlib
          ];
          PROJECT_ROOT = builtins.toString ./.;
          RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";
        };
      });
}
