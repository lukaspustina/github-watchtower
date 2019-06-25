all: check build test clippy fmt-check

todos:
	rg --vimgrep -g '!Makefile' -i todo 

check:
	cargo check --all --tests --examples

build:
	cargo build --all --tests --examples

test:
	cargo test

clean-package:
	cargo clean -p $$(cargo read-manifest | jq -r .name)

clippy:
	cargo clippy --all --all-targets -- -D warnings $$(source ".clippy.args")

fmt:
	cargo +nightly fmt

fmt-check:
	cargo +nightly fmt -- --check

duplicate_libs:
	cargo tree -d

_update-clippy_n_fmt:
	rustup update
	rustup component add clippy
	rustup component add rustfmt --toolchain=nightly

_cargo_install:
	cargo install -f cargo-tree
	cargo install -f cargo-bump

.PHONY: tests

