.PHONY: debug install build serve

WWW = target/www

serve:
	cargo watch --shell "make debug && python3 -m http.server 8000 -d ${WWW}"

debug:
	cargo build --target wasm32-unknown-unknown
	rm -rf ${WWW} || true
	wasm-bindgen --no-typescript --target web \
			--out-dir target/www \
			--out-name "mygame" \
			./target/wasm32-unknown-unknown/debug/bevy-hello-world.wasm
	cp index.html ${WWW}/index.html
	cp -r assets ${WWW}/assets

build:
	cargo build --release --target wasm32-unknown-unknown
	rm -rf ${WWW} || true
	wasm-bindgen --no-typescript --target web \
			--out-dir target/www \
			--out-name "mygame" \
			./target/wasm32-unknown-unknown/release/bevy-hello-world.wasm
	cp index.html ${WWW}/index.html
	cp -r assets ${WWW}/assets

install:
	rustup target install wasm32-unknown-unknown
	cargo install wasm-server-runner
	cargo install wasm-bindgen-cli
	cargo install cargo-watch
