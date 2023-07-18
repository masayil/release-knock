BIN_DIR = "dist/bin"

.PHONY: build
build: ## Build and cp the  binary under `./bin`.
	[ -d $(BIN_DIR) ] || mkdir -p $(BIN_DIR)
	cargo build --release --locked && cp target/release/release-knock $(BIN_DIR)

.PHONY: install ## Build and install the  binary under `~/.cargo/bin`.
install:
	cargo install --release --locked

.PHONY: clean
clean: ## Perform a `cargo` clean and remove the binary directories.
	cargo clean
	rm -rf $(BIN_DIR)