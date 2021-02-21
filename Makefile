setup:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

run:
	cargo package --manifest-path wql/Cargo.toml
	cargo run --manifest-path woori-db/Cargo.toml --release

release:
	cargo package --manifest-path wql/Cargo.toml
	cargo run --manifest-path woori-db/Cargo.toml --release

debug:
	cargo package --manifest-path wql/Cargo.toml
	cargo run --manifest-path woori-db/Cargo.toml