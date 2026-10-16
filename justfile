set shell := ["bash", "-uc"]

nightly := `rustc --version | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' | sed 's/^/nightly-/'`

check:
    cargo check --all-targets

fix:
    cargo fix --all-targets --allow-dirty --allow-staged

fmt:
    cargo +{% raw %}{{ nightly }}{% endraw %} fmt
    RUST_LOG=error taplo fmt

fmt-check:
    cargo +{% raw %}{{ nightly }}{% endraw %} fmt --check

lint:
    cargo clippy --all-targets --no-deps -- -D warnings

lint-fix:
    cargo clippy --all-targets --no-deps --fix --allow-dirty --allow-staged

test:
    cargo test

doc:
    cargo doc --no-deps

all: check fmt lint test doc

run:
    RUST_LOG={{ crate_name }}=debug,info \
        cargo run -p {{ project-name }} | tee ./target/{{ crate_name }}.log
