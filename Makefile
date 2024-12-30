format-rust:
	@cargo +nightly fmt

format-python:
	@ruff check --fix

format: format-rust format-python

lint-rust:
	@cargo +nightly fmt --check
	@cargo clippy -- -D warnings

lint-python:
	@ruff check

lint: lint-rust lint-python

test-rust:
	@cargo test --verbose

test: test-rust
