{
  rust-toolchain,
  rust-bin,

  dbus,
  pkg-config,
  makeRustPlatform,
}:

let
  rustPlatform = makeRustPlatform {
    cargo = rust-bin.fromRustupToolchain rust-toolchain;
    rustc = rust-bin.fromRustupToolchain rust-toolchain;
  };
in

rustPlatform.buildRustPackage {
  pname = "rustdeck";
  version = "1.0.0";

  src = ./..;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    dbus # dependency of rustdeck-media plugin
  ];

  cargoHash = "sha256-P/E93e2oVd0/zBc5zJgQqc8jcE7SzIQG0Shc7bqrIU0=";

  buildPhase = ''
    export HOME=$TEMPDIR

    cargo build --locked --release --package rustdeck-media
    cargo build --locked --release --package rustdeck-obs
    cargo build --locked --release --package rustdeck-system

    cargo build --locked --release --package rustdeck-server
  '';

  installPhase = ''
    mkdir -p $out/plugins
    mkdir -p $out/bin

    cp ./target/release/librustdeck_media.so $out/plugins/rustdeck_media.deckplugin
    cp ./target/release/librustdeck_obs.so $out/plugins/rustdeck_obs.deckplugin
    cp ./target/release/librustdeck_system.so $out/plugins/rustdeck_system.deckplugin

    cp ./target/release/rustdeck-server $out/bin/rustdeck-server
  '';

  meta = {
    mainProgram = "rustdeck-server";
  };
}
