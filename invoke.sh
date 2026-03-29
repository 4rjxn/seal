#!/bin/sh

# Seal Installer 🦭

set -e

# --- Configuration ---
GITHUB_REPO="4rjxn/seal"
BINARY_NAME="seal"
INSTALL_DIR="/usr/local/bin"

# Colors for output
BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

printf "${BLUE}==>${NC} Installing Seal...\n"

# --- Check for Cargo (Recommended for Source Install) ---
if command -v cargo >/dev/null 2>&1; then
    printf "${BLUE}==>${NC} Rust detected. Installing via Cargo for optimal performance...\n"
    # If the script is running via curl | sh, we can't easily install from local path.
    # We suggest installing from the git repo.
    if cargo install --git "https://github.com/$GITHUB_REPO" >/dev/null 2>&1; then
        printf "${GREEN}==>${NC} Seal installed successfully to $(cargo home)/bin/seal\n"
        exit 0
    else
        printf "${RED}!<${NC} Cargo install failed. Installation Failed...\n"
    fi
fi

