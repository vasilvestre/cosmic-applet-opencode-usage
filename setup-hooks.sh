#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
#
# Setup script to install git hooks for code quality enforcement

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GIT_DIR="$SCRIPT_DIR/.git"
HOOKS_DIR="$GIT_DIR/hooks"
SOURCE_HOOKS_DIR="$SCRIPT_DIR/.githooks"

echo "🔧 Setting up git hooks..."

# Check if we're in a git repository
if [ ! -d "$GIT_DIR" ]; then
    echo "❌ Error: Not in a git repository"
    exit 1
fi

# Install pre-commit hook
if [ -f "$SOURCE_HOOKS_DIR/pre-commit" ]; then
    echo "📝 Installing pre-commit hook..."
    cp "$SOURCE_HOOKS_DIR/pre-commit" "$HOOKS_DIR/pre-commit"
    chmod +x "$HOOKS_DIR/pre-commit"
    echo "✅ Pre-commit hook installed"
else
    echo "❌ Error: pre-commit hook not found in .githooks/"
    exit 1
fi

echo ""
echo "✅ Git hooks setup complete!"
echo ""
echo "The following checks will run before each commit:"
echo "  - Code formatting (rustfmt)"
echo "  - Linting (clippy with pedantic warnings)"
echo ""
echo "To skip hooks in exceptional cases, use: git commit --no-verify"
