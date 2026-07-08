#!/usr/bin/env bash
#
# install-seal.sh
# Installs the latest commit of the "seal" shell (https://github.com/4rjxn/seal)
# by cloning/pulling the repo and compiling it with cargo.
#
# Usage:
#   ./install-seal.sh
#
# Environment overrides:
#   SEAL_REPO      - repo URL              (default: https://github.com/4rjxn/seal.git)
#   SEAL_BRANCH    - branch to build        (default: trunk)
#   SEAL_SRC_DIR   - where to clone source  (default: ~/.local/src/seal)
#   SEAL_INSTALL_DIR - where to put binary  (default: ~/.local/bin)

set -euo pipefail

REPO_URL="${SEAL_REPO:-https://github.com/4rjxn/seal.git}"
BRANCH="${SEAL_BRANCH:-trunk}"
SRC_DIR="${SEAL_SRC_DIR:-$HOME/.local/src/seal}"
INSTALL_DIR="${SEAL_INSTALL_DIR:-$HOME/.local/bin}"
BIN_NAME="seal"

info()  { printf '\033[1;34m[info]\033[0m %s\n' "$1"; }
ok()    { printf '\033[1;32m[ok]\033[0m %s\n' "$1"; }
error() { printf '\033[1;31m[error]\033[0m %s\n' "$1" >&2; }

# --- 1. Check for git ---------------------------------------------------
if ! command -v git >/dev/null 2>&1; then
    error "git is not installed. Please install git and re-run this script."
    exit 1
fi

# --- 2. Check for Rust / cargo, install via rustup if missing ----------
if ! command -v cargo >/dev/null 2>&1; then
    info "cargo not found. Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
else
    info "cargo found: $(cargo --version)"
fi

# --- 3. Clone or update the repo ----------------------------------------
mkdir -p "$(dirname "$SRC_DIR")"

if [ -d "$SRC_DIR/.git" ]; then
    info "Existing checkout found at $SRC_DIR, updating..."
    git -C "$SRC_DIR" remote set-url origin "$REPO_URL"
    git -C "$SRC_DIR" fetch --all --prune
    git -C "$SRC_DIR" checkout "$BRANCH"
    git -C "$SRC_DIR" reset --hard "origin/$BRANCH"
else
    info "Cloning $REPO_URL (branch: $BRANCH) into $SRC_DIR..."
    git clone --branch "$BRANCH" "$REPO_URL" "$SRC_DIR"
fi

COMMIT_HASH="$(git -C "$SRC_DIR" rev-parse --short HEAD)"
info "Building commit $COMMIT_HASH"

# --- 4. Build with cargo --------------------------------------------------
info "Compiling with cargo (release mode)... this may take a while."
(
    cd "$SRC_DIR"
    cargo build --release
)

BUILT_BIN="$SRC_DIR/target/release/$BIN_NAME"

if [ ! -f "$BUILT_BIN" ]; then
    error "Build finished but binary not found at $BUILT_BIN"
    error "Check the Cargo.toml [package] name matches '$BIN_NAME', or set BIN_NAME accordingly."
    exit 1
fi

# --- 5. Install the binary ------------------------------------------------
mkdir -p "$INSTALL_DIR"
cp -f "$BUILT_BIN" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME"

ok "Installed '$BIN_NAME' (commit $COMMIT_HASH) to $INSTALL_DIR/$BIN_NAME"

# --- 6. PATH check ---------------------------------------------------------
case ":$PATH:" in
    *":$INSTALL_DIR:"*)
        ok "You're all set. Run it with: $BIN_NAME"
        ;;
    *)
        info "$INSTALL_DIR is not on your PATH."
        info "Add this to your shell rc file (e.g. ~/.bashrc, ~/.zshrc):"
        printf '\n    export PATH="%s:$PATH"\n\n' "$INSTALL_DIR"
        ;;
esac
