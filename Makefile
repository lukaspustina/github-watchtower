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

DOCKER_BUILD_IMAGE = lukaspustina/rust_musl:stable
RUN_DOCKER = docker run --rm -it -v "$$(pwd)":/home/rust/src -v "$$(pwd)/.cargo.cache/git":/home/rust/.cargo/git -v "$$(pwd)/.cargo.cache/registry":/home/rust/.cargo/registry $(DOCKER_BUILD_IMAGE)
RUN_DOCKER = docker run --rm -it -v "$$(pwd)":/home/rust/src -v "$$(pwd)/.cargo.cache/git":/home/rust/.cargo/git -v "$$(pwd)/.cargo.cache/registry":/home/rust/.cargo/registry $(DOCKER_BUILD_IMAGE)
FUNC_NAME = github-watchtower
FUNC_NAME_BIN = github-watchtower

.cargo.cache/git:
	mkdir -p $@
	$(RUN_DOCKER) sudo chown -R rust:rust /home/rust/.cargo/git

.cargo.cache/registry:
	mkdir -p $@
	$(RUN_DOCKER) sudo chown -R rust:rust /home/rust/.cargo/registry

cross_compile: ../target/x86_64-unknown-linux-musl/release/$(FUNC_NAME_BIN)
../target/x86_64-unknown-linux-musl/release/$(FUNC_NAME_BIN): .cargo.cache/git .cargo.cache/registry
	$(RUN_DOCKER) cargo test --package $(FUNC_NAME) --release
	$(RUN_DOCKER) cargo build --package $(FUNC_NAME) --release


upgrade: upgrade-docker-images

upgrade-docker-images:
	docker pull $(DOCKER_BUILD_IMAGE)


clean-cross:
	$(RUN_DOCKER) cargo clean

clean-cross-me:
	$(RUN_DOCKER) cargo clean --package $(FUNC_NAME)
	rm target/x86_64-unknown-linux-musl/release/$(FUNC_NAME_BIN)



.PHONY: 

