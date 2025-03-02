{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.*.tar.gz";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems f;
    in
    {
      overlays.default = final: prev: {
        rustToolchain = prev.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rustfmt"
            "rustfmt"
          ];
        };
      };

      devShells = forEachSupportedSystem (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              rust-overlay.overlays.default
              self.overlays.default
            ];
          };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustToolchain
              openssl
              pkg-config
              cargo-deny
              cargo-edit
              cargo-watch
              rust-analyzer
            ];

            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
            };
          };
        }
      );

      packages = forEachSupportedSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        rec {
          minesweeper-rs = pkgs.rustPlatform.buildRustPackage {
            pname = "minesweeper-rs";
            version = "0.0.1";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };
          default = minesweeper-rs;
        }
      );

      apps = forEachSupportedSystem (system: rec {
        minesweeper-rs = {
          type = "app";
          program = "${self.packages.${system}.minesweeper-rs}/bin/minesweeper-rs";
        };
        default = minesweeper-rs;
      });
    };
}
