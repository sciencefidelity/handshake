{
  description = "A Rust devshell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
        name = manifest.name;
        version = manifest.version;
      in
      with pkgs;
      {
        packages.default = pkgs.rustPlatform.buildRustPackage
          {
            pname = name;
            version = version;
            cargoLock.lockFile = ./Cargo.lock;
            src = pkgs.lib.cleanSource ./.;
          };

        devShells.default = mkShell {
          buildInputs = [
            cargo-feature
            cargo-machete
            cargo-tarpaulin
            pkg-config
            taplo
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-analyzer" "rust-src" ];
            })
          ];
        };

        apps.${name} = {
          type = "app";
          program = "${self.packages.${system}.runme}/bin/${name}";
        };
      }
    );
}
