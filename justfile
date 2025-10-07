set shell := ["bash", "-uc"]

check:
	cargo check --tests

fmt toolchain="+nightly":
	cargo {{ "{{toolchain}}" }} fmt

fmt-check toolchain="+nightly":
	cargo {{ "{{toolchain}}" }} fmt --check

fix:
	cargo fix --tests --allow-dirty --allow-staged

lint:
	cargo clippy --tests --no-deps -- -D warnings

lint-fix:
	cargo clippy --tests --no-deps --allow-dirty --allow-staged --fix

test:
	cargo test

doc:
	cargo doc --no-deps

all: check fmt lint test doc

run:
	RUST_LOG={{ crate_name }}=debug,info \
		cargo run -p {{ project-name }} | tee ./target/{{ crate_name }}.log
