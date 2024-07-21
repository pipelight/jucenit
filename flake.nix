{
  description = "Jucenit - A simple web server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
  } @ inputs:
  # flake-utils.lib.eachDefaultSystem
  # (
  #   system: let
  #     pkgs = nixpkgs.legacyPackages.${system};
  #   in rec {
  #     packages.default = pkgs.callPackage ./package.nix {};
  #     devShells.default = pkgs.callPackage ./shell.nix {};
  #     nixosModules = {
  #       jucenit = ./module.nix;
  #     };
  #   }
  # );
  let
    pkgs = nixpkgs.legacyPackages.x86_64-linux;
  in {
    packages.x86_64-linux.default = pkgs.callPackage ./package.nix {};
    devShells.x86_64-linux.default = pkgs.callPackage ./shell.nix {};
    nixosModules = {
      jucenit = ./module.nix;
    };
  };
}
