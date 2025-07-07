#!/bin/bash

set -e

echo "🌐 Building WASM package..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack is not installed!"
    echo "📦 Install it with: cargo install wasm-pack"
    exit 1
fi

# Check if wasm32 target is installed
if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    echo "📦 Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Clean previous build
echo "🧹 Cleaning previous WASM build..."
rm -rf public/pkg/

# Build WASM package (remove --release to avoid duplication)
echo "🔨 Building WASM with wasm-pack..."
wasm-pack build \
    --target web \
    --out-dir public/pkg \
    --features wasm \
    --no-default-features

# Check if build was successful
if [ -f "public/pkg/distli_mesh_bc.js" ]; then
    echo "✅ WASM build successful!"
    echo "📁 Files generated in public/pkg/:"
    ls -la public/pkg/
    echo ""
    echo "🌐 You can now open http://localhost:3030 in your browser"
else
    echo "❌ WASM build failed - distli_mesh_bc.js not found"
    exit 1
fi
