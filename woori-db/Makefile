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
	rm -rf *.log
	cargo test -- --ignored controllers::wql::test::test_delete_withput_update
	rm -rf *.log
	cargo test -- --ignored controllers::wql::test::test_delete_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::wql::test::test_update_uniqueness_set_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::wql::test::test_match_any_update_fail
	rm -rf *.log
	cargo test -- --ignored controllers::wql::test::test_match_any_update_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::wql::test::test_match_all_update_post_ok
	rm -rf *.log

release:
	cargo run --release