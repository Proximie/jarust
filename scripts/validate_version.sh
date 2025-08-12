#!/bin/bash
set -e

# Validate semver and higher than prev one

INPUT_VERSION="$1"

if [ -z "$INPUT_VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.2.3"
    exit 1
fi

# Semver check
if [[ ! "$INPUT_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be semantic version major.minor.patch (e.g., 1.2.3)"
    exit 1
fi

LATEST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "0.0.0")
LATEST_VERSION=${LATEST_TAG#v}  # Remove 'v' prefix if present

version_compare() {
    local ver1=$1
    local ver2=$2
    
    IFS='.' read -ra VER1 <<< "$ver1"
    IFS='.' read -ra VER2 <<< "$ver2"
    
    if [[ ${VER2[0]} -gt ${VER1[0]} ]]; then
        return 0
    elif [[ ${VER2[0]} -lt ${VER1[0]} ]]; then
        return 1
    fi
    
    if [[ ${VER2[1]} -gt ${VER1[1]} ]]; then
        return 0
    elif [[ ${VER2[1]} -lt ${VER1[1]} ]]; then
        return 1
    fi
    
    if [[ ${VER2[2]} -gt ${VER1[2]} ]]; then
        return 0
    else
        return 1
    fi
}

echo "Latest version: $LATEST_VERSION"
echo "Input version: $INPUT_VERSION"

if version_compare "$LATEST_VERSION" "$INPUT_VERSION"; then
    echo "✓ Version $INPUT_VERSION is higher than $LATEST_VERSION"
else
    echo "Error: Version $INPUT_VERSION must be higher than the current version $LATEST_VERSION"
    exit 1
fi
