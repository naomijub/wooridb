setup:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

run:
	cargo package --manifest-path wql/Cargo.toml
	cargo run --manifest-path woori-db/Cargo.toml --release -- -C opt-level="s" 

release:
	cargo package --manifest-path wql/Cargo.toml
	cargo run --manifest-path woori-db/Cargo.toml --release

debug:
	cargo package --manifest-path wql/Cargo.toml
	cargo run --manifest-path woori-db/Cargo.toml

json:
	cargo package --manifest-path wql/Cargo.toml
	cargo run --manifest-path woori-db/Cargo.toml --release --features json

history:
	cargo package --manifest-path wql/Cargo.toml
	cargo run --manifest-path woori-db/Cargo.toml --release --features history

push:
	docker build -t naomijubs/wooridb:$(tag) .
	docker push naomijubs/wooridb:$(tag)