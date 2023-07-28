# influxive Makefile

.PHONY: all publish test static docs tools tool_rust tool_fmt tool_readme

SHELL = /usr/bin/env sh -eu

all: test

publish:
	@case "$(crate)" in \
		influxive-core) \
			export MANIFEST="./crates/influxive-core/Cargo.toml"; \
			;; \
		influxive-writer) \
			export MANIFEST="./crates/influxive-writer/Cargo.toml"; \
			;; \
		influxive-downloader) \
			export MANIFEST="./crates/influxive-downloader/Cargo.toml"; \
			;; \
		influxive-child-svc) \
			export MANIFEST="./crates/influxive-child-svc/Cargo.toml"; \
			;; \
		influxive-otel-atomic-obs) \
			export MANIFEST="./crates/influxive-otel-atomic-obs/Cargo.toml"; \
			;; \
		influxive-otel) \
			export MANIFEST="./crates/influxive-otel/Cargo.toml"; \
			;; \
		influxive) \
			export MANIFEST="./crates/influxive/Cargo.toml"; \
			;; \
		*) \
			echo "USAGE: make publish crate=influxive-core"; \
			echo "USAGE: make publish crate=influxive-writer"; \
			echo "USAGE: make publish crate=influxive-downloader"; \
			echo "USAGE: make publish crate=influxive-child-svc"; \
			echo "USAGE: make publish crate=influxive-otel-atomic-obs"; \
			echo "USAGE: make publish crate=influxive-otel"; \
			echo "USAGE: make publish crate=influxive"; \
			exit 1; \
			;; \
	esac; \
	export VER="v$$(grep version $${MANIFEST} | head -1 | cut -d ' ' -f 3 | cut -d \" -f 2)"; \
	echo "publish $(crate) $${MANIFEST} $${VER}"; \
	git diff --exit-code; \
	cargo publish --manifest-path $${MANIFEST}; \
	git tag -a "$(crate)-$${VER}" -m "$(crate)-$${VER}"; \
	git push --tags;

test: static tools
	cargo build --all-targets
	RUST_BACKTRACE=1 cargo test -- --nocapture

static: docs tools
	cargo fmt -- --check
	cargo clippy
	@if [ "${CI}x" != "x" ]; then git diff --exit-code; fi

docs: tools
	cargo rdme --force -w influxive-core
	cargo rdme --force -w influxive-writer
	cargo rdme --force -w influxive-downloader
	cargo rdme --force -w influxive-child-svc
	cargo rdme --force -w influxive-otel-atomic-obs
	cargo rdme --force -w influxive-otel
	cargo rdme --force -w influxive

tools: tool_rust tool_fmt tool_clippy tool_readme

tool_rust:
	@if rustup --version >/dev/null 2>&1; then \
		echo "# Makefile # found rustup, setting override stable"; \
		rustup override set stable; \
	else \
		echo "# Makefile # rustup not found, hopefully we're on stable"; \
	fi;

tool_fmt: tool_rust
	@if ! (cargo fmt --version); \
	then \
		if rustup --version >/dev/null 2>&1; then \
			echo "# Makefile # installing rustfmt with rustup"; \
			rustup component add rustfmt; \
		else \
			echo "# Makefile # rustup not found, cannot install rustfmt"; \
			exit 1; \
		fi; \
	else \
		echo "# Makefile # rustfmt ok"; \
	fi;

tool_clippy: tool_rust
	@if ! (cargo clippy --version); \
	then \
		if rustup --version >/dev/null 2>&1; then \
			echo "# Makefile # installing clippy with rustup"; \
			rustup component add clippy; \
		else \
			echo "# Makefile # rustup not found, cannot install clippy"; \
			exit 1; \
		fi; \
	else \
		echo "# Makefile # clippy ok"; \
	fi;

tool_readme: tool_rust
	@if ! (cargo rdme --version); \
	then \
		cargo install cargo-rdme; \
	else \
		echo "# Makefile # readme ok"; \
	fi;
