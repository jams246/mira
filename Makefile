.PHONY: build\:production build\:msi

build\:production:
	cargo build --release --locked --bin mira --target x86_64-pc-windows-msvc --config .cargo/release.toml

build\:msi: build\:production
	cargo wix --no-build --target x86_64-pc-windows-msvc
