all: build test

build:
	cargo build --release --lib --target=wasm32-unknown-unknown
	chisel run --config chisel.toml; true

test:
	#node relayer/index.js
	../../target/release/phase2-scout smptvbrs.yaml
