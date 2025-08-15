set shell := ["bash", "-uc"]

check:
	cargo check --tests

fmt toolchain="+nightly":
	cargo {{ "{{toolchain}}" }} fmt

fmt-check toolchain="+nightly":
	cargo {{ "{{toolchain}}" }} fmt --check

lint:
	cargo clippy --no-deps --tests -- -D warnings

test:
	cargo test

fix:
	cargo fix --allow-dirty --allow-staged

doc:
	cargo doc --no-deps

all: check fmt lint test doc

run:
	RUST_LOG={{ crate_name }}=debug,info \
		cargo run -p {{ project-name }} | tee ./target/{{ crate_name }}.log
