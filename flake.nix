{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system overlays; };

        lib = pkgs.lib;
        stdenv = pkgs.stdenv;

        apple_sdk = pkgs.darwin.apple_sdk;

        overlays = [ (import rust-overlay) ];

        rustVersion = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "rust-analyzer" ];
        };
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        nativeBuildInputs = [
        ] ++ lib.optionals stdenv.isLinux [
          pkgs.pkg-config
        ];

        buildInputs = [
          pkgs.openssl
        ] ++ lib.optionals stdenv.isDarwin [
          apple_sdk.frameworks.Security
        ];

        crateInfo = builtins.fromTOML builtins.readFile ./Cargo.toml;

        projectCrate = rustPlatform.buildRustPackage {
          inherit (crateInfo.package) name info;

          src = ./.;

          nativeBuildInputs = nativeBuildInputs;

          buildInputs = buildInputs;

          cargoLock.lockFile = ./Cargo.lock;
        };
      in
      {
        defaultPackage = projectCrate;

        formatter = pkgs.nixpkgs-fmt;

        devShell = pkgs.mkShell {
          nativeBuildInputs = nativeBuildInputs;

          buildInputs = buildInputs ++ [
            # projectCrate

            pkgs.cargo-watch
            pkgs.cargo-machete

            rustVersion

            pkgs.treefmt
          ];
          
        };
      }
    );
}
