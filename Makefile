.PHONY: install test clean check format fmt

install:
	cargo install --path . --force

test:
	cargo test

clean:
	cargo clean

check:
	cargo check

format fmt:
	cargo fmt
