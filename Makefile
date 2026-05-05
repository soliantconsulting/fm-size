.PHONY: check test build release clean help build-all build-linux build-windows build-mac-intel build-mac-m1

# Default target
help:
	@echo "Available commands:"
	@echo "  check    - Run fmt, clippy, and tests"
	@echo "  fmt      - Format code"
	@echo "  clippy   - Run linter"
	@echo "  test     - Run tests"
	@echo "  build    - Build in debug mode (current platform)"
	@echo "  release  - Build in release mode (current platform)"
	@echo "  clean    - Clean build artifacts"
	@echo "  install  - Install release binary"
	@echo ""
	@echo "Cross-platform builds:"
	@echo "  build-all      - Build release for all platforms"
	@echo "  build-linux    - Build release for Linux (x86_64)"
	@echo "  build-windows  - Build release for Windows (x86_64)"
	@echo "  build-mac-intel - Build release for macOS Intel (x86_64)"
	@echo "  build-mac-m1   - Build release for macOS M1/M2 (aarch64)"

# Run all checks (fmt, clippy, test)
check: fmt clippy test
	@echo "✅ All checks passed!"

# Format code
fmt:
	@echo "🔧 Formatting code..."
	cargo fmt --all

# Run linter
clippy:
	@echo "🔍 Running clippy..."
	cargo clippy -- -Dwarnings

# Run tests
test:
	@echo "🧪 Running tests..."
	FM_SIZE_SKIP_PROMPTS=1 cargo test --verbose

# Build in debug mode
build:
	@echo "🔨 Building..."
	cargo build

# Build in release mode
release:
	@echo "🚀 Building release..."
	cargo build --release

# Clean build artifacts
clean:
	@echo "🧹 Cleaning..."
	cargo clean

# Install release binary
install: release
	@echo "📦 Installing binary..."
	cargo install --path . 

# Cross-platform builds
build-all: build-linux build-windows build-mac-intel build-mac-m1
	@echo "✅ Built for all platforms!"

build-linux:
	@echo "🐧 Building for Linux (x86_64)..."
	cargo build --release --target x86_64-unknown-linux-gnu

build-windows:
	@echo "🪟 Building for Windows (x86_64)..."
	cargo build --release --target x86_64-pc-windows-gnu

build-mac-intel:
	@echo "🍎 Building for macOS Intel (x86_64)..."
	cargo build --release --target x86_64-apple-darwin

build-mac-m1:
	@echo "🍎 Building for macOS M1/M2 (aarch64)..."
	cargo build --release --target aarch64-apple-darwin

