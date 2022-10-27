#!/usr/bin/env bash
#install script

cargo build --release
sudo cp target/release/linux_creation_tool /usr/bin/

sudo mkdir -p /etc/linux_creation_tool/pictures
sudo cp pictures/*.png /etc/linux_creation_tool/pictures/
sudo cp config.json /etc/linux_creation_tool/

sudo cp pictures/icon.png /usr/share/pixmaps/linux_creation_tool.png
sudo cp linux_creation_tool.desktop /usr/share/applications/
