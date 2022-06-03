.PHONY: build
build:
	cargo build --release
	ln -f $(CURDIR)/target/release/basilisk $(CURDIR)/target/release/testing-basilisk

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
	cargo tarpaulin --avoid-cfg-tarpaulin --features=runtime-benchmarks --workspace --locked  --exclude-files node/* --exclude-files runtime/* --exclude-files infrastructure/* --exclude-files **/weights.rs --ignore-tests -o Xml -o lcov

.PHONY: clippy
clippy:
	cargo clippy --release --locked --all-targets --features=runtime-benchmarks -- -D warnings

.PHONY: format
format:
	cargo fmt

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
