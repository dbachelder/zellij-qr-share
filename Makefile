.PHONY: build install clean reload

build:
	cargo build --release

install: build
	mkdir -p ~/.config/zellij/plugins
	cp target/wasm32-wasip1/release/qr-share.wasm ~/.config/zellij/plugins/

reload: install
	rm -rf ~/.cache/zellij/

clean:
	cargo clean
