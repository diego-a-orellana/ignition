# format/clippy/test
all:
	make format && make clippy && make test

.PHONY: format
format:
	cargo +nightly fmt

.PHONY: clippy
clippy:
	cargo clippy --lib

.PHONY: test
test:
	cargo test --lib -vv