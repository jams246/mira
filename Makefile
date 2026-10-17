.PHONY: build\:production

build\:production:
	cargo build --release --locked --bin mira --target x86_64-pc-windows-msvc --config .cargo/release.toml
