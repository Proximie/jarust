#!/bin/bash
set -e

# Publish crates to crates.io

CRATES_TOKEN="$1"

if [ -z "$CRATES_TOKEN" ]; then
    echo "Usage: $0 <crates_token>"
    echo "Example: $0 your_crates_io_token"
    exit 1
fi

echo "Publishing crates to crates.io..."

# Publish in dependency order
echo "Publishing jarust_rt..."
cargo publish -p jarust_rt --token "$CRATES_TOKEN"
sleep 10

echo "Publishing jarust_interface..."
cargo publish -p jarust_interface --token "$CRATES_TOKEN"
sleep 10

echo "Publishing jarust_core..."
cargo publish -p jarust_core --token "$CRATES_TOKEN"
sleep 10

echo "Publishing jarust_plugins..."
cargo publish -p jarust_plugins --token "$CRATES_TOKEN"
sleep 10

echo "Publishing jarust..."
cargo publish -p jarust --token "$CRATES_TOKEN"

echo "✓ Successfully published all crates to crates.io"