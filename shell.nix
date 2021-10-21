let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  nixpkgs = import <nixpkgs> {
    overlays = [ rust_overlay ];
  };
in
  with nixpkgs;
  clangStdenv.mkDerivation {
    name = "rust";
    nativeBuildInputs = [
      pkg-config
      rust-bin.stable.latest
      rustPackages.clippy
      rust-analyzer
      ];
    buildInputs = [ cargo-release ];
  }
