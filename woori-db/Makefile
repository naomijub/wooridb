run-watch:
	cargo watch -x 'run --bin app'

run:
	cargo run

fmt:
	cargo fmt

clippy:
	 cargo clippy -- -W clippy::pedantic

lint: fmt clippy

test:
	cargo test -- --nocapture
	rm -rf *.log
	cargo test -- --ignored controllers::wql::test::test_update_content_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::wql::test::test_update_set_post_ok

release:
	cargo run --release