PROJECT:=$(shell basename $(CURDIR))
COMMIT:=$(shell git rev-parse --short HEAD 2>/dev/null)-$(shell date "+%Y%m%d%H%M%S")
TAG:=$(shell git describe --tags --dirty 2>/dev/null)
BUILD_CACHE_DIR:=$(CURDIR)/.cache

.PHONY: help
help:
	@grep -E '^[a-zA-Z%_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: fmt ## Build binaries
	cargo build $(CARGO_OPTS) --bins

.PHONY: fmt
fmt: ## Format & Lint codes
	rustup component add rustfmt clippy
	cargo fmt

.PHONY: release
release: ## Release binaries
	CARGO_OPTS="--release" $(MAKE) build

.PHONY: install
install: ## Install binaries
	cargo install $(CARGO_OPTS) --path ./src/app/ --bins

.PHONY: clean
clean: ## Clean build caches
	cargo clean
	rm -rf $(BUILD_CACHE_DIR)

.PHONY: fix
fix:  ## Fix code
	echo cargo fix --allow-dirty --bins

.PHONY: generate
generate: ## Generate managed package list
	echo "Need to have GITHUB_TOKEN to automatically generate package description"
	GITHUB_TOKEN=$(GITHUB_TOKEN) cargo build --manifest-path ./src/generator/Cargo.toml
