.PHONY : compile install clean setup

compile:
	cargo build --release

install: compile setup

setup: compile pictures/endeavor.png pictures/fedora.png pictures/icon.png pictures/mint.png pictures/missing.png pictures/pop!_os.png pictures/zorin.png config.json linux_creation_tool.desktop
	sudo mkdir -p /etc/linux_creation_tool/pictures/
	sudo cp pictures/* /etc/linux_creation_tool/pictures/
	sudo cp pictures/icon.png /usr/share/pixmaps/linux_creation_tool.png
	sudo cp linux_creation_tool.desktop /usr/share/applications/
	sudo cp target/release/linux_creation_tool /usr/bin
	sudo cp config.json /etc/linux_creation_tool/

update:
	cargo update

clean:
	cargo clean
