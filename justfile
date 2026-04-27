set shell := ["bash", "-uc"]

nightly := `rustc --version | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' | sed 's/^/nightly-/'`

check:
	cargo check --tests

fix:
    cargo fix --allow-dirty --allow-staged --tests

fmt:
    cargo +{{nightly}} fmt

fmt-check:
    cargo +{{nightly}} fmt --check

lint:
	cargo clippy --no-deps --tests -- -D warnings

lint-fix:
    cargo clippy --no-deps --tests --fix --allow-dirty --allow-staged

test:
	cargo test

doc:
	cargo doc --no-deps

all: check fmt lint test doc

run:
	RUST_LOG={{ crate_name }}=debug,info \
		cargo run -p {{ project-name }} | tee ./target/{{ crate_name }}.log
