SHELL := /bin/bash

CARGO ?= cargo
PACKAGE := primary
WASM_TARGET := wasm32v1-none

.PHONY: all build test fmt fmt-check clippy verify clean deploy-testnet

all: verify

build:
	@bash scripts/build.sh

test:
	$(CARGO) test --workspace --all-targets --locked

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy --workspace --all-targets --locked -- -D warnings

verify: fmt-check clippy test build

clean:
	$(CARGO) clean
	rm -rf artifacts

deploy-testnet: build
	@bash scripts/deploy_testnet.sh
