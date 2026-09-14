#!/bin/bash

# WhisprTypr Development Script
# Quick development setup and run

set -e

echo "WhisprTypr Development Mode"
echo "==========================="

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "Error: Must be run from the project root directory"
    exit 1
fi

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    pnpm install
fi

# Run in development mode
echo "Starting development server..."
pnpm tauri dev
