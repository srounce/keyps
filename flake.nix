{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ flake-parts, nixpkgs, treefmt-nix, rust-overlay, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        treefmt-nix.flakeModule
      ];
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      perSystem = { system, pkgs, ... }:
        let
          lib = pkgs.lib;
          stdenv = pkgs.stdenv;

          apple_sdk = pkgs.darwin.apple_sdk_11_0;

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

          crateInfo = builtins.fromTOML (builtins.readFile ./Cargo.toml);

          projectCrate = rustPlatform.buildRustPackage {
            inherit (crateInfo.package) name description;

            src = ./.;

            nativeBuildInputs = nativeBuildInputs;

            buildInputs = buildInputs;

            cargoLock.lockFile = ./Cargo.lock;
          };
        in
        {
          _module.args.pkgs = import nixpkgs {
            inherit system;
            overlays = [
              (import rust-overlay)
            ];
          };

          packages.default = projectCrate;

          devShells.default = pkgs.mkShell {
            nativeBuildInputs = nativeBuildInputs;

            buildInputs = buildInputs ++ [
              # projectCrate

              pkgs.cargo-watch
              pkgs.cargo-machete

              rustVersion

              pkgs.treefmt
            ];

          };

          treefmt.config = {
            projectRootFile = "flake.nix";
            programs.nixpkgs-fmt.enable = true;
            programs.rustfmt.enable = true;
            programs.rustfmt.package = rustVersion;
            programs.prettier.enable = true;
          };
        };
    };
}
