#!/bin/bash
set -e

REPO="dvaji/infera"
BINARY_NAME="infs"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

get_latest_version() {
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

get_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)
            if [ "$ARCH" = "x86_64" ]; then
                echo "linux-x86_64"
            elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
                echo "linux-aarch64"
            else
                error "Unsupported architecture: $ARCH"
            fi
            ;;
        Darwin)
            if [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
                echo "macos-aarch64"
            else
                error "Unsupported macOS architecture: $ARCH. Only Apple Silicon (aarch64/arm64) is supported."
            fi
            ;;
        *)
            error "Unsupported OS: $OS"
            ;;
    esac
}

check_dependencies() {
    if ! command -v curl &> /dev/null; then
        error "curl is required but not installed. Please install curl first."
    fi
}

detect_shell_profile() {
    SHELL_PROFILE=""
    
    case "${SHELL##*/}" in
        zsh)
            if [ -f "$HOME/.zshrc" ]; then
                SHELL_PROFILE="$HOME/.zshrc"
            fi
            ;;
        bash)
            if [ -f "$HOME/.bashrc" ]; then
                SHELL_PROFILE="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                SHELL_PROFILE="$HOME/.bash_profile"
            fi
            ;;
        fish)
            if [ -f "$HOME/.config/fish/config.fish" ]; then
                SHELL_PROFILE="$HOME/.config/fish/config.fish"
            fi
            ;;
        *)
            if [ -f "$HOME/.profile" ]; then
                SHELL_PROFILE="$HOME/.profile"
            fi
            ;;
    esac

    if [ -z "$SHELL_PROFILE" ]; then
        for profile in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile" "$HOME/.bash_profile"; do
            if [ -f "$profile" ]; then
                SHELL_PROFILE="$profile"
                break
            fi
        done
    fi
}

install_infs() {
    check_dependencies

    VERSION="${1:-$(get_latest_version)}"
    PLATFORM=$(get_platform)
    ASSET_NAME="infs-$PLATFORM"
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET_NAME"

    info "Installing infs $VERSION for $PLATFORM..."

    mkdir -p "$INSTALL_DIR"

    TEMP_FILE=$(mktemp)
    trap 'rm -f "$TEMP_FILE"' EXIT

    info "Downloading from $DOWNLOAD_URL..."
    if ! curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_FILE"; then
        error "Failed to download infs. The version may not have a release for your platform."
    fi

    chmod +x "$TEMP_FILE"

    mv "$TEMP_FILE" "$INSTALL_DIR/$BINARY_NAME"

    info "Installed infs to $INSTALL_DIR/$BINARY_NAME"

    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        detect_shell_profile
        if [ -n "$SHELL_PROFILE" ]; then
            info "Adding $INSTALL_DIR to PATH in $SHELL_PROFILE..."
            echo "" >> "$SHELL_PROFILE"
            echo "# Added by infs installer" >> "$SHELL_PROFILE"
            if [[ "${SHELL##*/}" == "fish" ]]; then
                echo "set -gx PATH \$PATH $INSTALL_DIR" >> "$SHELL_PROFILE"
            else
                echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_PROFILE"
            fi
            export PATH="$PATH:$INSTALL_DIR"
            info "Added to PATH. Run 'source $SHELL_PROFILE' or restart your shell."
        else
            warn "Could not detect shell profile. Add the following manually:"
            echo ""
            echo "    export PATH=\"\$PATH:$INSTALL_DIR\""
        fi
    fi

    "$INSTALL_DIR/$BINARY_NAME" --version
    info "Installation complete!"
}

if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: curl -fsSL https://raw.githubusercontent.com/dvaji/infera/main/install.sh | bash"
    echo "   or: curl -fsSL https://raw.githubusercontent.com/dvaji/infera/main/install.sh | bash -s -- v1.0.0"
    echo ""
    echo "Environment variables:"
    echo "  INSTALL_DIR  - Directory to install infs (default: \$HOME/.local/bin)"
    echo ""
    echo "Examples:"
    echo "  INSTALL_DIR=/usr/local/bin curl -fsSL ... | bash"
    exit 0
fi

install_infs "$1"
