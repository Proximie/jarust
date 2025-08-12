#!/bin/bash
set -e

# Update Cargo.toml versions across the workspace

VERSION="$1"

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.2.3"
    exit 1
fi

echo "Updating Cargo.toml versions to $VERSION"

sed -i '' 's/^version = ".*"/version = "'$VERSION'"/' Cargo.toml
sed -i '' 's/\(jarust_[a-zA-Z_]*\) = { version = "[^"]*"/\1 = { version = "'$VERSION'"/g' Cargo.toml

echo "✓ Updated all Cargo.toml files to version $VERSION"
