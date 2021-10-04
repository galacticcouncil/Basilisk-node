.PHONY: build
build:
	cargo build --release
	ln -s $(CURDIR)/target/release/basilisk $(CURDIR)/target/release/testing-basilisk

.PHONY: check
check:
	cargo check --release

.PHONY: build-benchmarks
build-benchmarks:
	cargo build --release --features runtime-benchmarks

.PHONY: test
test:
	cargo test --release

.PHONY: test-benchmarks
test-benchmarks:
	cargo test --release --features runtime-benchmarks

.PHONY: clippy
clippy:
	cargo clippy --release --all-targets --all-features -- -D warnings

.PHONY: build-docs
build-docs:
	cargo doc --release --target-dir ./Basilisk-dev-docs --no-deps

.PHONY: clean
clean:
	cargo clean
