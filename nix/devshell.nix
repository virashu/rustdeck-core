{
  rust-toolchain,

  mkShell,

  dbus,
  pkg-config,
  rustup,
  rustPlatform,
}:

mkShell {
  RUSTC_VERSION = rust-toolchain.channel;
  nativeBuildInputs = [
    dbus # dependency of rustdeck-media plugin

    pkg-config

    rustup
    rustPlatform.bindgenHook
  ];
}
