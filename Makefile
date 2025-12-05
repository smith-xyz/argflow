.PHONY: build test lint check clean format format-check ci release help

help:
	@echo "Available targets:"
	@echo "  build        - Build the project"
	@echo "  test         - Run all tests"
	@echo "  lint         - Run clippy linter"
	@echo "  check        - Check code compiles without building"
	@echo "  format       - Format code with rustfmt"
	@echo "  format-check - Check code formatting without modifying"
	@echo "  ci           - Run all CI checks (format-check, lint, check, test)"
	@echo "  release      - Build release binary"
	@echo "  clean        - Remove build artifacts"

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

format-check:
	cargo fmt --check

ci: format-check lint check test

release:
	cargo build --release

clean:
	cargo clean

