#!/usr/bin/env bash
#install script

cargo build --release
cp target/release/linux_creation_tool /usr/bin/

mkdir /etc/linux_creation_tool/pictures
cp pictures/*.png /etc/linux_creation_tool/pictures
cp config.json /etc/linux_creation_tool/

cp pictures/icon.png /usr/share/pixmaps/sev.linux_creation_tool.png
cp sev.linux_creation_tool.desktop /usr/share/applications/