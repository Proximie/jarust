#!/bin/bash
set -e

# Create git tag and push changes and tag

VERSION="$1"
EMAIL="$2"

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version> <email>"
    echo "Example: $0 1.2.3 user@example.com"
    exit 1
fi

if [ -z "$EMAIL" ]; then
    echo "Usage: $0 <version> <email>"
    echo "Example: $0 1.2.3 user@example.com"
    exit 1
fi

echo "Creating release for version $VERSION"

echo "Configuring git..."
git config --local user.email "$EMAIL"
git config --local user.name "proximie-machine-user"

echo "Adding and committing changes..."
git add .
git commit -m "chore(release): bump version to $VERSION" || true

echo "Creating git tag $VERSION..."
git tag -a "$VERSION" -m "Release $VERSION"

echo "Pushing changes and tag to remote..."
git push origin HEAD
git push origin "$VERSION"

echo "✓ Successfully released version $VERSION"
