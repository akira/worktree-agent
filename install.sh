#!/bin/bash
set -euo pipefail

# Worktree Agent (wta) installer
# https://github.com/akira/wta
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/akira/wta/main/install.sh | bash
#
# Or download and run locally:
#   ./install.sh

REPO="akira/wta"
BINARY_NAME="wta"
INSTALL_DIR="${WTA_INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}==>${NC} $1"
}

success() {
    echo -e "${GREEN}==>${NC} $1"
}

warn() {
    echo -e "${YELLOW}Warning:${NC} $1"
}

error() {
    echo -e "${RED}Error:${NC} $1" >&2
    exit 1
}

# Detect OS
detect_os() {
    local os
    os="$(uname -s)"
    case "$os" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "darwin" ;;
        *)       error "Unsupported operating system: $os" ;;
    esac
}

# Detect architecture
detect_arch() {
    local arch
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *)             error "Unsupported architecture: $arch" ;;
    esac
}

# Get the latest release version from GitHub
get_latest_version() {
    local version
    version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$version" ]; then
        error "Failed to get latest release version. Please check https://github.com/${REPO}/releases"
    fi
    echo "$version"
}

# Build the download URL for the release asset
build_download_url() {
    local version="$1"
    local os="$2"
    local arch="$3"

    local target
    case "$os" in
        linux)
            target="${arch}-unknown-linux-gnu"
            ;;
        darwin)
            target="${arch}-apple-darwin"
            ;;
    esac

    echo "https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${target}.tar.gz"
}

# Main installation function
main() {
    info "Installing ${BINARY_NAME}..."

    # Detect platform
    local os arch
    os=$(detect_os)
    arch=$(detect_arch)
    info "Detected platform: ${os}/${arch}"

    # Get latest version
    info "Fetching latest release..."
    local version
    version=$(get_latest_version)
    info "Latest version: ${version}"

    # Build download URL
    local url
    url=$(build_download_url "$version" "$os" "$arch")
    info "Downloading from: ${url}"

    # Create temp directory
    local tmpdir
    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    # Download and extract
    local archive="${tmpdir}/${BINARY_NAME}.tar.gz"
    if ! curl -fsSL "$url" -o "$archive"; then
        error "Failed to download release. URL: ${url}"
    fi

    info "Extracting..."
    tar -xzf "$archive" -C "$tmpdir"

    # Find the binary (might be in root or subdirectory)
    local binary
    binary=$(find "$tmpdir" -name "$BINARY_NAME" -type f -perm -u+x 2>/dev/null | head -n1)
    if [ -z "$binary" ]; then
        # Try without execute permission check (will chmod later)
        binary=$(find "$tmpdir" -name "$BINARY_NAME" -type f 2>/dev/null | head -n1)
    fi
    if [ -z "$binary" ]; then
        error "Binary not found in archive"
    fi

    # Create install directory if needed
    mkdir -p "$INSTALL_DIR"

    # Install binary
    info "Installing to ${INSTALL_DIR}/${BINARY_NAME}..."
    chmod +x "$binary"
    mv "$binary" "${INSTALL_DIR}/${BINARY_NAME}"

    success "Successfully installed ${BINARY_NAME} ${version} to ${INSTALL_DIR}/${BINARY_NAME}"

    # Check if install dir is in PATH
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        warn "${INSTALL_DIR} is not in your PATH"
        echo ""
        echo "Add it to your shell configuration:"
        echo ""
        echo "  # For bash (~/.bashrc or ~/.bash_profile):"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        echo "  # For zsh (~/.zshrc):"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        echo "  # For fish (~/.config/fish/config.fish):"
        echo "  set -gx PATH \$HOME/.local/bin \$PATH"
        echo ""
    fi

    echo ""
    success "Installation complete! Run '${BINARY_NAME} --help' to get started."
}

main "$@"
