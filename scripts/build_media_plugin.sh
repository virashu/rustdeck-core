#!/bin/bash

cargo build --package rustdeck-media
echo "Built rustdeck-media"

cp -pf ./target/debug/librustdeck_media.so ./plugins/rustdeck_media.deckplugin
echo "Copied rustdeck-media"
