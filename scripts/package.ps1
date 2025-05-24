New-Item -ItemType Directory -Path .\dist              -Force > $null
New-Item -ItemType Directory -Path .\dist\pack         -Force > $null
New-Item -ItemType Directory -Path .\dist\pack\plugins -Force > $null

cargo build --package rustdeck-media --release
cargo build --package rustdeck       --release

Copy-Item .\target\release\rustdeck.exe       .\dist\pack\rustdeck.exe                      -Force
Copy-Item .\target\release\rustdeck_media.dll .\dist\pack\plugins\rustdeck_media.deckplugin -Force

7z a -tzip -r .\dist\rustdeck.zip -w .\dist\pack\*
