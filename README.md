<h1 style="text-align: center">Rust Deck 🖲️</h1>

<p style="text-align: center">
  An application to control PC behavior
  remotely, using mobile device
</p>

<p style="text-align: center; background: rgb(150,100,0);">
  ⚠️ Very unstable ⚠️
</p>

#### Inspired by:
  - [Stream Deck](https://www.elgato.com/us/en/s/welcome-to-stream-deck)
  - [Macro Deck](https://macro-deck.app/)

> [!IMPORTANT]
> Project uses Rust's nightly `min-specialization` feature

## Build

### Taskfile

Project uses `Taskfile` (`go-task`)

Run this command to produce `dist/rustdeck.zip` package:
```shell
task package
```
The package will contain executable and builtin plugins.

> [!IMPORTANT]
> Task build on windows relies on coreutils (`cygwin`) installed

### Without Taskfile

Project has scripts in according directory. There's `package.ps1` script.
