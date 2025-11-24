set shell := ["bash", "-uc"]

rust_version := `grep channel rust-toolchain.toml | sed -r 's/channel = "(.*)"/\1/'`
nightly := "nightly-2025-10-29"

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
