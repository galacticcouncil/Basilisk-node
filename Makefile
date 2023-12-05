.PHONY: build
build:
	cargo build --release

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

.PHONY: coverage
coverage:
	cargo tarpaulin --avoid-cfg-tarpaulin --all-features --workspace --locked  --exclude-files node/* --exclude-files runtime/* --exclude-files infrastructure/* --exclude-files **/weights.rs --ignore-tests -o Xml -o lcov

.PHONY: clippy
clippy:
	cargo clippy --release --locked --all-targets --all-features -- -D warnings -A deprecated
	cargo clippy --release --locked --all-targets -- -D warnings -A deprecated

.PHONY: format
format:
	cargo fmt

.PHONY: try-runtime
try-runtime:
	try-runtime --runtime ./target/release/wbuild/basilisk-runtime/basilisk_runtime.wasm on-runtime-upgrade --checks all live --uri wss://rpc.basilisk.cloud:443

.PHONY: build-docs
build-docs:
	cargo doc --release --target-dir ./Basilisk-dev-docs --no-deps

.PHONY: clean
clean:
	cargo clean

.PHONY: docker
docker: build
	ln -f $(CURDIR)/target/release/basilisk $(CURDIR)/basilisk
	docker build --tag basilisk .
	rm $(CURDIR)/basilisk

checksum:
	sha256sum target/release/basilisk > target/release/basilisk.sha256
	cp target/release/wbuild/basilisk-runtime/basilisk_runtime.compact.compressed.wasm target/release/
	sha256sum target/release/basilisk_runtime.compact.compressed.wasm > target/release/basilisk_runtime.compact.compressed.wasm.sha256

release: build checksum

all: clippy test test-benchmarks build build-benchmarks checksum
