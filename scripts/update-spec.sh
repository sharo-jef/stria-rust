#!/bin/bash

# Stria Rust - Spec Submodule Update Script
# This script updates the spec submodule to the latest version

set -e

echo "Updating spec submodule..."

# Check if we're in the project root
if [ ! -f "Cargo.toml" ]; then
    echo "Error: This script must be run from the project root directory"
    exit 1
fi

# Check if spec directory exists
if [ ! -d "spec" ]; then
    echo "Error: spec directory not found"
    exit 1
fi

# Initialize submodules if needed
echo "Initializing submodules..."
git submodule update --init --recursive

# Update spec submodule to latest
echo "Updating spec submodule to latest commit..."
git submodule update --remote spec

# Check if there are any changes
if git diff --quiet spec; then
    echo "Spec submodule is already up to date"
else
    echo "Spec submodule has been updated"
    
    # Show the changes
    echo "Changes in spec submodule:"
    git diff --stat spec
    
    # Ask user if they want to commit the changes
    read -p "Do you want to commit these changes? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Get the latest commit message from spec submodule
        cd spec
        LATEST_COMMIT=$(git log -1 --pretty=format:"%h - %s")
        cd ..
        
        # Commit the submodule update
        git add spec
        git commit -m "docs: update spec submodule to latest version

Updated to: $LATEST_COMMIT"
        echo "Changes committed successfully"
    else
        echo "Changes not committed. You can commit manually later with:"
        echo "   git add spec"
        echo "   git commit -m \"docs: update spec submodule to latest version\""
    fi
fi

echo "Spec submodule update complete!"
