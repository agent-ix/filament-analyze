# =============================================================================
# Filament Analyze Makefile
# =============================================================================

ifneq ($(filter ci,$(MAKECMDGOALS)),)
ifneq ($(strip $(MAKEFLAGS)),)
$(error local CI refuses non-empty MAKEFLAGS)
endif
ifneq ($(origin CARGO),undefined)
$(error local CI refuses a CARGO override)
endif
endif

override CARGO := cargo
LOCKED_BUILD ?= locked-build.sh
QUOIN ?= quoin

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make figures          - Regenerate docs/images from docs/images/figures.json (ixd build)"
	@echo "  make figures-check    - Fail when docs/images are stale; writes nothing"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make real-tool-test   - Ignored real Z3/cvc5 tests with retained SUITE-004 results"
	@echo "  make spec             - Quire-validate the specification and plan"
	@echo "  make msrv             - Test the locked graph with Rust 1.98.1"
	@echo "  make rustdoc          - Build warning-free public documentation"
	@echo "  make coverage         - Enforce the local line-coverage floor"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - cargo deny check licenses"
	@echo "  make ci               - All CI gates locally (fmt-check + lint + test + deny + spec + msrv + rustdoc + coverage)"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: lint
lint:
	$(CARGO) clippy --all-targets -- -D warnings

.PHONY: test
test:
	$(CARGO) test

# The wrapper owns admission for every cargo invocation started by the lane.
.PHONY: real-tool-test
real-tool-test:
	LOCKED_BUILD="$(LOCKED_BUILD)" LOCKED_BUILD_DEFAULT="$(if $(filter file,$(origin LOCKED_BUILD)),1,0)" CARGO="$(CARGO)" QUOIN="$(QUOIN)" python3 scripts/real_tool_test.py

.PHONY: spec
spec:
	quire validate --scope . 'spec/**/*.md' 'planning/**/*.md' 'plan/**/*.md' 'reviews/**/*.md'

.PHONY: msrv
msrv:
	$(CARGO) +1.98.1 test --locked

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings $(CARGO) doc --locked --no-deps

.PHONY: coverage
coverage:
	$(CARGO) llvm-cov --all-targets --locked --fail-under-lines 90

.PHONY: build
build:
	$(CARGO) build --release

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny deny-advisories deny-bans deny-licenses deny-sources
deny: deny-advisories deny-bans deny-licenses deny-sources

deny-advisories:
	$(CARGO) deny check advisories

deny-bans:
	$(CARGO) deny check bans

deny-licenses:
	$(CARGO) deny check licenses

deny-sources:
	$(CARGO) deny check sources

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test deny spec msrv rustdoc coverage

# =============================================================================
# Documentation figures
# =============================================================================
# docs/images/figures.json defines the figures; ixd-figures (agent-ix/ix-diagrams) renders them.
# Install it with: cargo install --git https://github.com/agent-ix/ix-diagrams ix-diagrams

IXD ?= ixd

.PHONY: figures
figures:
	$(IXD) build docs/images/figures.json --out docs/images

.PHONY: figures-check
figures-check:
	$(IXD) build docs/images/figures.json --out docs/images --check
