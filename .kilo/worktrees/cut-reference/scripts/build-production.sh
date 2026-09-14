#!/bin/bash

# WhisprTypr Production Build Script
# This script builds the application for production release

set -e

echo "WhisprTypr Production Build"
echo "=========================="
echo ""

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "Error: Must be run from the project root directory"
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${GREEN}▶ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}Warning: $1${NC}"
}

print_error() {
    echo -e "${RED}Error: $1${NC}"
}

print_success() {
    echo -e "${GREEN}Success: $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

detect_host_platform() {
    case "$(uname -s)" in
        Darwin*)
            echo "macos"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "windows"
            ;;
        *)
            echo "unsupported"
            ;;
    esac
}

# Parse arguments
TARGET_PLATFORM=""
SKIP_CHECKS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --platform)
            TARGET_PLATFORM="$2"
            shift 2
            ;;
        --skip-checks)
            SKIP_CHECKS=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --platform <platform>  Build for specific platform (windows, macos)"
            echo "  --skip-checks          Skip type checking and linting"
            echo "  --help                 Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Step 1: Clean previous builds
print_step "Step 1/7: Cleaning previous builds..."
rm -rf dist
rm -rf src-tauri/target/release/bundle
echo "   Cleaned dist and bundle directories"

# Step 2: Check dependencies
print_step "Step 2/7: Checking dependencies..."
if ! command -v pnpm &> /dev/null; then
    print_error "pnpm is not installed. Please install it first."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo is not installed. Please install it first."
    exit 1
fi

# Check Rust version
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
print_info "Rust version: $RUST_VERSION"

# Check Node version  
NODE_VERSION=$(node --version)
print_info "Node version: $NODE_VERSION"

# Step 3: Install dependencies
print_step "Step 3/7: Installing dependencies..."
pnpm install --frozen-lockfile

# Step 4: Run checks (unless skipped)
if [ "$SKIP_CHECKS" = false ]; then
    print_step "Step 4/7: Running type checks and linting..."
    
    # TypeScript type check
    echo "   Running TypeScript type check..."
    pnpm exec tsc --noEmit
    
    # Rust check
    echo "   Running Rust check..."
    cd src-tauri && cargo check --release && cd ..
    
    print_success "All checks passed!"
else
    print_warning "Step 4/7: Skipping checks (--skip-checks flag set)"
fi

# Step 5: Build frontend
print_step "Step 5/7: Building frontend..."
pnpm build
print_success "Frontend built successfully!"

# Step 6: Build Tauri application
print_step "Step 6/7: Building Tauri application (this may take a while)..."

if [ -n "$TARGET_PLATFORM" ]; then
    print_info "Building for platform: $TARGET_PLATFORM"
    case $TARGET_PLATFORM in
        windows)
            cargo tauri build --target x86_64-pc-windows-msvc --bundles nsis,msi
            ;;
        macos)
            cargo tauri build --target x86_64-apple-darwin --bundles dmg,app
            ;;
        *)
            print_error "Unknown platform: $TARGET_PLATFORM"
            exit 1
            ;;
    esac
else
    HOST_PLATFORM="$(detect_host_platform)"
    case $HOST_PLATFORM in
        windows)
            cargo tauri build --bundles nsis,msi
            ;;
        macos)
            cargo tauri build --bundles dmg,app
            ;;
        *)
            print_error "This project only supports Windows and macOS builds."
            exit 1
            ;;
    esac
fi

# Step 7: Display build results
echo ""
echo "=========================="
print_success "Build completed successfully!"
echo ""
echo "Build artifacts location:"

# List actual build files
if [ -d "src-tauri/target/release/bundle" ]; then
    echo ""
    echo "Generated installers:"
    
    # Find and list all installer files
    find src-tauri/target/release/bundle -type f \( \
        -name "*.msi" -o \
        -name "*.exe" -o \
        -name "*.dmg" -o \
        -name "*.app" \
    \) 2>/dev/null | while read file; do
        size=$(du -h "$file" | cut -f1)
        filename=$(basename "$file")
        echo "   $filename ($size)"
        echo "      Path: $file"
    done
    
    echo ""
    echo "Bundle directories:"
    ls -d src-tauri/target/release/bundle/*/ 2>/dev/null | while read dir; do
        dirname=$(basename "$dir")
        echo "   $dirname/"
    done
fi

echo ""
print_info "To install on this machine:"
echo "   Windows and macOS installers are under src-tauri/target/release/bundle/"
echo ""
