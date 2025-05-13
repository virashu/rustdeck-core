cargo build --package sample_plugin

Copy-Item .\target\debug\sample_plugin.dll .\plugins\sample_plugin.deckplugin
Write-Output "Copied!"

cargo run --package rustdeck
