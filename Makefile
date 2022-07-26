.PHONY : compile install clean setup

compile:
	cargo build --release

install: compile setup

setup: compile pictures/endeavor.png pictures/fedora.png pictures/icon.png pictures/mint.png pictures/missing.png pictures/pop!_os.png pictures/zorin.png config.json linux_creation_tool.desktop
	sudo mkdir -p /etc/linux_creation_tool/pictures/
	sudo cp pictures/* /etc/linux_creation_tool/pictures/
	sudo cp pictures/icon.png /usr/share/pixmaps/
	sudo cp linux_creation_tool.desktop /usr/share/applications/
	sudo cp target/release/linux_creation_tool /usr/bin

update:
	cargo update

clean:
	cargo clean
