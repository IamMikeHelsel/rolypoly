SHELL := /bin/bash

.PHONY: build test fmt lint ci

build:
	cargo build --bins --all-targets

test:
	cargo test --all --no-default-features

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings

ci: fmt lint test

