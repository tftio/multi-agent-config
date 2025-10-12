#!/bin/bash

# install.sh - Install Multi Agent Config from GitHub releases
# Usage: curl -sSL https://raw.githubusercontent.com/jfb/multi-agent-config/main/install.sh | sh
# Or: INSTALL_DIR=/usr/local/bin curl -sSL ... | sh

set -euo pipefail

# Configuration
GITHUB_REPO="jfb/multi-agent-config"
BINARY_NAME="multi-agent-config"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output (only if terminal supports it)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1" >&2; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1" >&2; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1" >&2; }
log_error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }

# Detect platform and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)     os="unknown-linux-gnu" ;;
        Darwin*)    os="apple-darwin" ;;
        CYGWIN*|MINGW*|MSYS*) os="pc-windows-msvc" ;;
        *)          log_error "Unsupported operating system: $(uname -s)"; exit 1 ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        aarch64|arm64)  arch="aarch64" ;;
        armv7l)         arch="armv7" ;;
        *)              log_error "Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac

    echo "${arch}-${os}"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Download file with progress
download_file() {
    local url="$1"
    local output="$2"

    if command_exists curl; then
        curl -sSL --fail --progress-bar "$url" -o "$output"
    elif command_exists wget; then
        wget -q --show-progress --progress=bar:force:noscroll "$url" -O "$output"
    else
        log_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Get latest release information from GitHub API
get_latest_release() {
    local api_url="https://api.github.com/repos/$GITHUB_REPO/releases/latest"
    local response

    log_info "Fetching latest release information..."

    if command_exists curl; then
        response=$(curl -sSL "$api_url")
    elif command_exists wget; then
        response=$(wget -qO- "$api_url")
    else
        log_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi

    echo "$response"
}

# Extract download URL for the target platform
get_download_url() {
    local release_json="$1"
    local target="$2"
    local binary_pattern="${BINARY_NAME}-${target}\.tar\.gz"

    # Try to extract download URL using basic shell tools
    # Look for browser_download_url containing our target pattern
    echo "$release_json" | grep -o '"browser_download_url": *"[^"]*"' | \
        grep -o 'https://[^"]*' | \
        grep "$binary_pattern" | \
        head -n1
}

# Get checksum URL for verification
# Checksum file is named without the archive extension (e.g., project-aarch64-apple-darwin.sha256)
get_checksum_url() {
    local release_json="$1"
    local target="$2"
    local checksum_pattern="${BINARY_NAME}-${target}\.sha256"

    echo "$release_json" | grep -o '"browser_download_url": *"[^"]*"' | \
        grep -o 'https://[^"]*' | \
        grep "$checksum_pattern" | \
        head -n1
}

# Verify file checksum (mandatory)
verify_checksum() {
    local file="$1"
    local checksum_file="$2"

    if [ ! -f "$checksum_file" ]; then
        log_error "Checksum file not available."
        log_error "Checksum verification is mandatory for security."
        return 1
    fi

    log_info "Verifying checksum..."

    # Extract expected hash and verify directly
    local expected_sum=$(cut -d' ' -f1 "$checksum_file")
    local actual_sum

    if command_exists sha256sum; then
        actual_sum=$(sha256sum "$file" | cut -d' ' -f1)
    elif command_exists shasum; then
        actual_sum=$(shasum -a 256 "$file" | cut -d' ' -f1)
    else
        log_error "No checksum utility available (sha256sum or shasum required)."
        log_error "Checksum verification is mandatory for security."
        return 1
    fi

    if [ "$expected_sum" = "$actual_sum" ]; then
        log_info "Checksum verification passed"
        return 0
    else
        log_error "Checksum verification failed!"
        log_error "Expected: $expected_sum"
        log_error "Actual:   $actual_sum"
        return 1
    fi
}

# Extract archive based on file extension
extract_archive() {
    local archive="$1"
    local dest_dir="$2"

    case "$archive" in
        *.zip)
            if command_exists unzip; then
                unzip -q "$archive" -d "$dest_dir"
            else
                log_error "unzip command not found. Please install unzip."
                exit 1
            fi
            ;;
        *.tar.gz)
            tar -xzf "$archive" -C "$dest_dir"
            ;;
        *)
            log_error "Unsupported archive format: $archive"
            exit 1
            ;;
    esac
}

# Extract version from release JSON
get_version() {
    local release_json="$1"
    echo "$release_json" | grep -o '"tag_name": *"[^"]*"' | \
        grep -o 'multi-agent-config-v[0-9][^"]*' | head -n1 | sed 's/^multi-agent-config-v//'
}

# Main installation function
main() {
    log_info "Installing Multi Agent Config..."

    # Detect target platform
    local target
    target=$(detect_platform)
    log_info "Detected platform: $target"

    # Get latest release information
    local release_json
    release_json=$(get_latest_release)

    # Extract version, download URL, and checksum URL
    local version download_url checksum_url
    version=$(get_version "$release_json")
    download_url=$(get_download_url "$release_json" "$target")
    checksum_url=$(get_checksum_url "$release_json" "$target")

    if [ -z "$version" ]; then
        log_error "Could not determine latest version"
        exit 1
    fi

    if [ -z "$download_url" ]; then
        log_error "No release found for platform: $target"
        log_info "Available releases:"
        echo "$release_json" | grep -o '"browser_download_url": *"[^"]*"' | \
            grep -o 'https://[^"]*' | sed 's/^/  /'
        exit 1
    fi

    log_info "Latest version: $version"
    log_info "Download URL: $download_url"

    # Create install directory if it doesn't exist
    if [ ! -d "$INSTALL_DIR" ]; then
        log_info "Creating install directory: $INSTALL_DIR"
        mkdir -p "$INSTALL_DIR"
    fi

    # Create temporary directory for download and extraction
    local temp_dir
    temp_dir=$(mktemp -d)
    trap "rm -rf \"$temp_dir\"" EXIT

    local archive_file="$temp_dir/archive.zip"
    local checksum_file="$temp_dir/checksum.sha256"

    # Download archive
    log_info "Downloading $BINARY_NAME v$version..."
    download_file "$download_url" "$archive_file"

    # Download and verify checksum (mandatory)
    if [ -z "$checksum_url" ]; then
        log_error "No checksum file available in release."
        log_error "Checksum verification is mandatory for security."
        exit 1
    fi

    log_info "Downloading checksum file..."
    download_file "$checksum_url" "$checksum_file"

    if ! verify_checksum "$archive_file" "$checksum_file"; then
        log_error "Checksum verification failed, aborting installation"
        exit 1
    fi

    # Extract archive
    log_info "Extracting archive..."
    extract_archive "$archive_file" "$temp_dir"

    # Find the binary in the extracted contents
    local binary_path
    binary_path=$(find "$temp_dir" -name "$BINARY_NAME" -type f | head -n1)

    if [ -z "$binary_path" ]; then
        log_error "Could not find binary '$BINARY_NAME' in archive"
        exit 1
    fi

    # Make executable and move to install directory
    chmod +x "$binary_path"
    local install_path="$INSTALL_DIR/$BINARY_NAME"
    mv "$binary_path" "$install_path"

    log_success "$BINARY_NAME v$version installed to $install_path"

    # Check if install directory is in PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            log_warning "$INSTALL_DIR is not in your PATH"
            log_info "Add it to your PATH with: export PATH=\"\$PATH:$INSTALL_DIR\""
            ;;
    esac

    # Verify installation
    if [ -x "$install_path" ]; then
        log_info "Verifying installation..."
        "$install_path" --version || log_warning "Could not verify installation"
    fi

    log_success "Installation complete! Run '$BINARY_NAME --help' to get started."
}

# Run main function
main "$@"