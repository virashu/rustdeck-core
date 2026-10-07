{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
      rust-toolchain = (fromTOML (builtins.readFile ./rust-toolchain.toml)).toolchain;
    in
    {
      devShells.${system}.default = pkgs.callPackage ./nix/devshell.nix { inherit rust-toolchain; };
      packages.${system}.default = pkgs.callPackage ./nix/package.nix { inherit rust-toolchain; };
    };
}
