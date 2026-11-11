.PHONY: dev test lint dmg install clean

dev:
	cargo tauri dev

test:
	cd src-tauri && cargo test

lint:
	cd src-tauri && cargo clippy --all-targets -- -D warnings

dmg:
	cargo tauri build --bundles app,dmg

install: dmg
	./scripts/install-app.sh

clean:
	cd src-tauri && cargo clean
