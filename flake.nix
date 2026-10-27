{
  description = "A devShell example";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      rust-overlay,
      flake-parts,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = ["aarch64-darwin"];
      perSystem = { system, pkgs, ... }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            pkg-config
            rust-bin.beta.latest.default
            rust-analyzer
          ];
        };
      };
    };
}
