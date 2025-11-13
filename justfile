# Justfile for PDF Booklet Imposition Tool
# https://github.com/casey/just

# Default recipe - show available commands
default:
    @just --list

# ============================================================================
# Rust Commands
# ============================================================================

# Build Rust binaries (debug)
build:
    cargo build

# Build Rust binaries (release/optimized)
build-release:
    cargo build --release

# Run Rust booklet tool (debug)
booklet *ARGS:
    cargo run --bin booklet -- {{ARGS}}

# Run Rust booklet tool (release)
booklet-release *ARGS:
    cargo run --release --bin booklet -- {{ARGS}}

# Run Rust signature pages calculator (debug)
sig *ARGS:
    cargo run --bin signature-pages -- {{ARGS}}

# Run Rust signature pages calculator (release)
sig-release *ARGS:
    cargo run --release --bin signature-pages -- {{ARGS}}

# Run all Rust tests
test:
    cargo test

# Run Rust tests with output
test-verbose:
    cargo test -- --nocapture

# Run ignored tests (requires system dependencies)
test-all:
    cargo test -- --ignored --nocapture

# Check Rust code (fast compile check without codegen)
check:
    cargo check

# Run Clippy (Rust linter)
clippy:
    cargo clippy -- -D warnings

# Format Rust code
format:
    cargo fmt

# Check Rust formatting
format-check:
    cargo fmt -- --check

# Generate and open Rust documentation
doc:
    cargo doc --open

# ============================================================================
# Installation & Dependencies
# ============================================================================

# Install system dependencies (Arch Linux)
install-deps-arch:
    sudo pacman -S texlive-core texlive-latexextra texlive-fontsextra \
                   texlive-bin texlive-extra-utils ghostscript rust

# Install system dependencies (Ubuntu/Debian)
install-deps-ubuntu:
    sudo apt install texlive-extra-utils texlive-latex-extra \
                     ghostscript cargo

# Install Rust (if not already installed)
install-rust:
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Just (this tool)
install-just:
    cargo install just

# Check if all required tools are available
check-deps:
    @echo "Checking system dependencies..."
    @command -v gs >/dev/null && echo "✓ ghostscript" || echo "✗ ghostscript (missing)"
    @command -v pdfbook2 >/dev/null && echo "✓ pdfbook2" || echo "✗ pdfbook2 (missing)"
    @command -v pdfcrop >/dev/null && echo "✓ pdfcrop" || echo "✗ pdfcrop (missing)"
    @command -v cargo >/dev/null && echo "✓ cargo/rust" || echo "✗ cargo/rust (missing)"
    @echo ""
    @echo "Optional tools:"
    @command -v hyperfine >/dev/null && echo "✓ hyperfine (for benchmarks)" || echo "✗ hyperfine (install: cargo install hyperfine)"

# ============================================================================
# Cleanup Commands
# ============================================================================

# Clean Rust build artifacts
clean-rust:
    cargo clean

# Clean test output files
clean-pdfs:
    rm -f *_bleed.pdf *_padded.pdf *_booklet.pdf
    rm -rf *_signatures/

# Clean everything
clean: clean-rust clean-pdfs
    @echo "✓ All clean!"

# ============================================================================
# Development Commands
# ============================================================================

# Watch Rust files and rebuild on changes (requires cargo-watch)
watch:
    cargo watch -x check -x test -x build

# Install cargo-watch
install-watch:
    cargo install cargo-watch

# Run pre-commit checks (format, lint, test)
pre-commit: format clippy test
    @echo "✓ Pre-commit checks passed!"

# ============================================================================
# Binary Management
# ============================================================================

# Show binary sizes
binary-sizes:
    @echo "Debug binaries:"
    @ls -lh target/debug/booklet target/debug/signature-pages 2>/dev/null || echo "  (not built yet - run 'just build')"
    @echo ""
    @echo "Release binaries:"
    @ls -lh target/release/booklet target/release/signature-pages 2>/dev/null || echo "  (not built yet - run 'just build-release')"

# Strip release binaries (reduce size further)
strip-binaries: build-release
    strip target/release/booklet
    strip target/release/signature-pages
    @echo "✓ Binaries stripped!"
    @just binary-sizes

# Install Rust binaries to ~/.cargo/bin
install: build-release
    cargo install --path .
    @echo "✓ Installed to ~/.cargo/bin"
    @echo "  Run: booklet --help"
    @echo "  Run: signature-pages --help"

# ============================================================================
# Documentation
# ============================================================================

# Generate README stats
stats:
    @echo "=== Project Statistics ==="
    @echo ""
    @echo "Rust:"
    @find src -name "*.rs" -exec wc -l {} + | tail -1 || echo "  0 lines"
    @echo ""
    @echo "Tests:"
    @echo -n "  Rust: "
    @cargo test --quiet 2>&1 | grep -o "[0-9]* passed" | head -1 || echo "0 passed"
    @echo ""
    @echo "Binary sizes (release):"
    @ls -lh target/release/booklet target/release/signature-pages 2>/dev/null | awk '{print "  " $9 ": " $5}' || echo "  (not built yet)"

# Show help for Rust tool
help:
    cargo run --quiet --bin booklet -- --help

# ============================================================================
# CI/CD Simulation
# ============================================================================

# Run full CI pipeline locally
ci: clippy test build-release
    @echo "✓ CI pipeline passed!"

# Run full release build with checks
release: clean ci strip-binaries binary-sizes
    @echo "✓ Release build complete!"
