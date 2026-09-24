{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };
      rust-toolchain = fromTOML (builtins.readFile ./rust-toolchain.toml);
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        RUSTC_VERSION = rust-toolchain.toolchain.channel;
        nativeBuildInputs = with pkgs; [
          # dependency of rustdeck-media plugin
          dbus

          pkg-config

          rustup

          rustPlatform.bindgenHook
        ];
      };
    };
}
