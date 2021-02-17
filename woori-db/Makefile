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
	cargo test --release -- --ignored auth::controllers::test::bad_request_if_user_password_is_wrong
	cargo test --release -- --ignored auth::controllers::test::create_new_user_ok
	cargo test --release -- --ignored auth::controllers::test::create_new_user_wrong_admin
	cargo test --release -- --ignored auth::controllers::test::get_token_test
	cargo test --features test_read io::read::test::encryption_test
	cargo test --features test_read io::read::test::local_data_test
	cargo test --features test_read io::read::test::offset_test
	cargo test --features test_read io::read::unique_data_test
	rm -rf data/*.log
	cargo test -- --ignored controllers::tx_test::test_update_content_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::tx_test::test_update_set_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::tx_test::test_delete_withput_update
	rm -rf data/*.log
	cargo test -- --ignored controllers::tx_test::test_delete_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::tx_test::test_update_uniqueness_set_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::tx_test::test_match_any_update_fail
	rm -rf data/*.log
	cargo test -- --ignored controllers::tx_test::test_match_any_update_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::tx_test::test_match_all_update_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::tx_test::test_match_all_update_fake_key
	rm -rf data/*.log
	cargo test -- --ignored controllers::tx_test::test_match_any_update_fake_key
	rm -rf data/*.log
	cargo test -- --ignored controllers::query_test::test_update_set_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::query_test::test_select_args_id_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::query_test::test_select_keys_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::query_test::test_select_all_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::query_test::test_select_all_ids_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::query_test::test_select_keys_ids_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::query_test::test_select_all_without_encrypts_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::tx_test::test_update_set_encrypt_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::tx_test::test_update_content_encrypt_post_err
	rm -rf data/*.log
	cargo test -- --ignored controllers::tx_test::test_check_encrypt_post_ok
	rm -rf data/*.log
	cargo test -- --ignored controllers::clauses_test::simple_where_clause_eq
	rm -rf data/*.log
	cargo test -- --ignored controllers::clauses_test::clause_between
	rm -rf data/*.log
	cargo test -- --ignored controllers::clauses_test::clause_in
	rm -rf data/*.log
	cargo test -- --ignored controllers::clauses_test::clause_ge_le
	rm -rf data/*.log
	cargo test -- --ignored controllers::clauses_test::clause_or
	rm -rf data/*.log
	cargo test --release -- --ignored auth::controllers::routes_test_with_auth::query_and_tx_with_token
	rm -rf data/*.log

release:
	cargo run --release