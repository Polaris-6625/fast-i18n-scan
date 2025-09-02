#!/bin/bash

# Fast i18n Scan - NPM Publishing Script
# Version: 0.2.0

set -e

echo "🚀 Starting Fast i18n Scan publishing process..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -f "package.json" ]; then
    print_error "This script must be run from the project root directory"
    exit 1
fi

# Check if required tools are installed
print_status "Checking required tools..."

if ! command -v cargo &> /dev/null; then
    print_error "Cargo is not installed. Please install Rust first."
    exit 1
fi

if ! command -v npm &> /dev/null; then
    print_error "npm is not installed. Please install Node.js first."
    exit 1
fi

if ! command -v npx &> /dev/null; then
    print_error "npx is not installed. Please install Node.js first."
    exit 1
fi

print_success "All required tools are available"

# Install npm dependencies
print_status "Installing npm dependencies..."
npm install

# Check if @napi-rs/cli is available
if ! npx napi --version &> /dev/null; then
    print_error "@napi-rs/cli is not available. Installing..."
    npm install -g @napi-rs/cli
fi

print_success "Dependencies installed"

# Clean previous builds
print_status "Cleaning previous builds..."
rm -rf target/
rm -f *.node
rm -rf dist/

# Run Rust tests
print_status "Running Rust tests..."
cargo test
print_success "Rust tests passed"

# Build for all platforms
print_status "Building native modules for all platforms..."

# Build for current platform first
print_status "Building for current platform..."
npx napi build --platform --release

print_success "Native modules built successfully"

# Prepare package for publishing
print_status "Preparing package for publishing..."
npx napi prepublish -t npm

print_success "Package prepared for publishing"

# Verify package contents
print_status "Verifying package contents..."
npm pack --dry-run

# Ask for confirmation before publishing
echo ""
print_warning "Ready to publish fast-i18n-scan@0.2.0 to npm"
echo ""
echo "Package contents:"
echo "- Native bindings for multiple platforms"
echo "- TypeScript definitions"
echo "- JavaScript entry point"
echo "- Documentation"
echo ""

read -p "Do you want to proceed with publishing? (y/N): " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_warning "Publishing cancelled by user"
    exit 0
fi

# Check npm authentication
print_status "Checking npm authentication..."
if ! npm whoami &> /dev/null; then
    print_error "You are not logged in to npm. Please run 'npm login' first."
    exit 1
fi

print_success "npm authentication verified"

# Publish to npm
print_status "Publishing to npm..."
npm publish --access public

if [ $? -eq 0 ]; then
    print_success "🎉 Successfully published fast-i18n-scan@0.2.0 to npm!"
    echo ""
    echo "You can now install it with:"
    echo "  npm install fast-i18n-scan"
    echo ""
    echo "Package URL: https://www.npmjs.com/package/fast-i18n-scan"
else
    print_error "Failed to publish package"
    exit 1
fi

# Clean up
print_status "Cleaning up temporary files..."
rm -f fast-i18n-scan-*.tgz

print_success "Publishing process completed successfully! 🚀"