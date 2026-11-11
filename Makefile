SIGNING_IDENTITY ?= Penna Local Signing

.PHONY: dev test lint certificate dmg install clean

dev:
	cargo tauri dev

test:
	cd src-tauri && cargo test

lint:
	cd src-tauri && cargo clippy --all-targets -- -D warnings

certificate:
	./scripts/create-signing-certificate.sh "$(SIGNING_IDENTITY)"

dmg:
	APPLE_SIGNING_IDENTITY="$(SIGNING_IDENTITY)" cargo tauri build --bundles app,dmg

install: dmg
	./scripts/install-app.sh

clean:
	cd src-tauri && cargo clean
