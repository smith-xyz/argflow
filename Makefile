.PHONY: build test lint check clean format release help

help:
	@echo "Available targets:"
	@echo "  build    - Build the project"
	@echo "  test     - Run all tests"
	@echo "  lint     - Run clippy linter"
	@echo "  check    - Check code compiles without building"
	@echo "  format   - Format code with rustfmt"
	@echo "  release  - Build release binary"
	@echo "  clean    - Remove build artifacts"

build:
	cargo build

test:
	cargo test

lint:
	cargo clippy

check:
	cargo check

format:
	cargo fmt

release:
	cargo build --release

clean:
	cargo clean

