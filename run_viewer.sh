#!/bin/bash
# SPDX-License-Identifier: GPL-3.0-only

# Simple launcher for the OpenCode Usage Viewer

echo "===================================="
echo "OpenCode Usage Viewer Launcher"
echo "===================================="
echo ""

# Check if we have data
echo "Checking database..."
DB_PATH="$HOME/.local/share/cosmic-applet-opencode-usage/usage.db"
if [ ! -f "$DB_PATH" ]; then
    echo "❌ Database not found at: $DB_PATH"
    exit 1
fi

# Count records
RECORD_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM usage_snapshots;" 2>/dev/null || echo "0")
echo "✅ Database found with $RECORD_COUNT snapshots"
echo ""

# Default to release build for better performance
BUILD_TYPE="${1:-release}"

if [ "$BUILD_TYPE" = "debug" ]; then
    echo "Building viewer (DEBUG mode)..."
    cargo build --bin cosmic-applet-opencode-usage-viewer --quiet
    if [ $? -ne 0 ]; then
        echo "❌ Build failed"
        exit 1
    fi
    echo "✅ Build successful"
    echo ""
    echo "Launching viewer window (DEBUG)..."
    echo "(The window should appear on your desktop)"
    echo ""
    echo "Press Ctrl+C to exit"
    echo "===================================="
    echo ""
    cargo run --bin cosmic-applet-opencode-usage-viewer
else
    echo "Building viewer (RELEASE mode - optimized)..."
    cargo build --release --bin cosmic-applet-opencode-usage-viewer --quiet
    if [ $? -ne 0 ]; then
        echo "❌ Build failed"
        exit 1
    fi
    echo "✅ Build successful"
    echo ""
    echo "Launching viewer window (RELEASE)..."
    echo "(The window should appear on your desktop)"
    echo ""
    echo "Press Ctrl+C to exit"
    echo "===================================="
    echo ""
    ./target/release/cosmic-applet-opencode-usage-viewer
fi
