ifdef TARGET
	TARGET_ARG=--target $(TARGET)
else
	TARGET_ARG=
endif

all: check build test clippy fmt-check

$(info TARGET_ARG="$(TARGET_ARG)")

todos:
	rg --vimgrep -g '!Makefile' -i todo 

check:
	cargo check $(TARGET_ARG) --all --tests --examples

build:
	cargo build $(TARGET_ARG) --all --tests --examples

test:
	cargo test $(TARGET_ARG)

test-all:
	cargo test $(TARGET_ARG)
	cargo test $(TARGET_ARG) -- --ignored

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

_cargo_install:
	cargo install -f cargo-tree
	cargo install -f cargo-bump

_install:
	@if test $$TARGET; then \
		echo "Adding rust target $(TARGET)"; \
		rustup target add $(TARGET); \
	fi
	rustup component add clippy
	rustup toolchain install nightly
	rustup component add rustfmt --toolchain=nightly

sha_config: tests/config.toml
	shasum -a 256 $<

.PHONY: 

