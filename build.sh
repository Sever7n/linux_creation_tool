#!/usr/bin/env bash
#install script

cargo build --release
sudo cp target/release/linux_creation_tool /usr/bin/

sudo mkdir /etc/linux_creation_tool/pictures
sudo cp pictures/*.png /etc/linux_creation_tool/pictures/
sudo cp config.json /etc/linux_creation_tool/

sudo cp pictures/icon.png /usr/share/pixmaps/sev.linux_creation_tool.png
sudo cp sev.linux_creation_tool.desktop /usr/share/applications/
