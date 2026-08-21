#!/bin/bash

# Release build script for git-scanner
# Usage: ./scripts/release.sh

set -e  # Exit on error

echo "🔨 Building git-scanner in release mode..."

# Clean previous build
echo "🧹 Cleaning previous build..."
cargo clean

# Build with optimizations
echo "⚡ Building with maximum optimizations..."
cargo build --release

# Check if build succeeded
if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed!"
    exit 1
fi

# Copy binary to root
echo "📦 Copying binary..."
cp target/release/git-scanner ./git-scanner

# Display binary info
echo ""
echo "📊 Binary Information:"
ls -lh git-scanner
echo ""

# Run tests if needed
read -p "Run tests? (y/n): " run_tests
if [ "$run_tests" = "y" ]; then
    echo "🧪 Running tests..."
    cargo test --release
fi

# Run benchmarks if needed
read -p "Run benchmarks? (y/n): " run_bench
if [ "$run_bench" = "y" ]; then
    echo "📊 Running benchmarks..."
    cargo bench
fi

echo ""
echo "✅ Release build complete!"
echo "🚀 Run with: ./git-scanner"