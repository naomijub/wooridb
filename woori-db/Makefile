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
	cargo test --features test_read -- --ignored io::read::test::encryption_test
	cargo test --features test_read -- --ignored io::read::test::local_data_test
	cargo test --features test_read -- --ignored io::read::test::offset_test
	rm -rf *.log
	cargo test -- --ignored controllers::tx_test::test_update_content_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::tx_test::test_update_set_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::tx_test::test_delete_withput_update
	rm -rf *.log
	cargo test -- --ignored controllers::tx_test::test_delete_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::tx_test::test_update_uniqueness_set_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::tx_test::test_match_any_update_fail
	rm -rf *.log
	cargo test -- --ignored controllers::tx_test::test_match_any_update_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::tx_test::test_match_all_update_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::tx_test::test_match_all_update_fake_key
	rm -rf *.log
	cargo test -- --ignored controllers::tx_test::test_match_any_update_fake_key
	rm -rf *.log
	cargo test -- --ignored controllers::query_test::test_update_set_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::query_test::test_select_args_id_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::query_test::test_select_keys_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::query_test::test_select_all_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::query_test::test_select_all_ids_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::query_test::test_select_keys_ids_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::query_test::test_select_all_without_encrypts_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::tx_test::test_update_set_encrypt_post_ok
	rm -rf *.log
	cargo test -- --ignored controllers::tx_test::test_update_content_encrypt_post_err
	rm -rf *.log
	cargo test -- --ignored controllers::tx_test::test_check_encrypt_post_ok
	rm -rf *.log

release:
	cargo run --release