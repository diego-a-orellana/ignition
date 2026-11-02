# format/clippy/test
all:
	make format && make clippy && make test

.PHONY: format
format:
	cargo +nightly fmt

.PHONY: clippy
clippy:
	cargo clippy --lib --features "download-onnxruntime,download-opencv"

.PHONY: test
test:
	IGNITION_BUCKET_URL=http://127.0.0.1 cargo test --no-default-features --features download --test retrieval
