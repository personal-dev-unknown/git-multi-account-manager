#!/usr/bin/env bash
# scripts/run.sh — Build and run the Git Multi-Account Manager
#
# Usage:
#   ./scripts/run.sh [MODE] [OPTIONS]
#
# Modes:
#   cli       — Interactive CLI (default)
#   web       — Axum HTTP API server  (http://127.0.0.1:5000)
#   desktop   — Tauri desktop GUI
#   build     — Build all binaries without running
#
# Options:
#   --release     Build in release mode (faster, no debug symbols)
#   --rebuild-zig Force a rebuild of the Zig native layer
#   --help        Show this message
#
# Environment (set in .env or export before running):
#   GIT_MANAGER_DB_URL        MySQL URL  (default: mysql://git_manager:dev_password@localhost/git_manager_dev)
#   GIT_MANAGER_WEB_ADDR      Web bind   (default: 127.0.0.1:5000)
#   RUST_LOG                  Log level  (default: info)
#
# Quick-start with MySQL via Docker:
#   docker run -d --name gm-mysql -p 3306:3306 \
#     -e MYSQL_ROOT_PASSWORD=root \
#     -e MYSQL_DATABASE=git_manager_dev \
#     -e MYSQL_USER=git_manager \
#     -e MYSQL_PASSWORD=dev_password \
#     mysql:8
#   Then: ./scripts/run.sh cli

set -euo pipefail

# ── Locate workspace root ────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── Load .env if present ─────────────────────────────────────────────────────
if [[ -f "$ROOT/.env" ]]; then
    # Export non-comment, non-empty lines
    set -a
    # shellcheck disable=SC1090
    source "$ROOT/.env"
    set +a
fi

# ── Defaults ─────────────────────────────────────────────────────────────────
MODE="cli"
RELEASE=""
REBUILD_ZIG=0
CARGO_FLAGS=()
TARGET_DIR="$ROOT/target/debug"

# ── Parse arguments ──────────────────────────────────────────────────────────
for arg in "$@"; do
    case "$arg" in
        cli|web|desktop|build) MODE="$arg" ;;
        --release)
            RELEASE="--release"
            TARGET_DIR="$ROOT/target/release"
            ;;
        --rebuild-zig) REBUILD_ZIG=1 ;;
        --help|-h)
            grep '^#' "$0" | grep -v '#!/' | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *) echo "Unknown argument: $arg  (run with --help for usage)" >&2; exit 1 ;;
    esac
done

[[ -n "$RELEASE" ]] && CARGO_FLAGS+=("--release")

# ── Colour helpers ───────────────────────────────────────────────────────────
bold=$'\e[1m'; green=$'\e[32m'; yellow=$'\e[33m'; red=$'\e[31m'; reset=$'\e[0m'
step() { echo "${bold}${green}▶${reset} $*"; }
warn() { echo "${yellow}⚠  $*${reset}"; }
die()  { echo "${red}✗  $*${reset}" >&2; exit 1; }

# ── Prerequisite checks ───────────────────────────────────────────────────────
command -v zig   >/dev/null 2>&1 || die "zig not found. Install from https://ziglang.org/download/"
command -v cargo >/dev/null 2>&1 || die "cargo not found. Install Rust from https://rustup.rs/"

# ── Build Zig native layer ────────────────────────────────────────────────────
ZIG_LIB="$ROOT/zig_native/zig-out/lib/libgm_native.a"

needs_zig_build=0
if [[ ! -f "$ZIG_LIB" ]]; then
    needs_zig_build=1
elif [[ $REBUILD_ZIG -eq 1 ]]; then
    needs_zig_build=1
elif [[ -n "$(find "$ROOT/zig_native/src" -newer "$ZIG_LIB" -name '*.zig' 2>/dev/null)" ]]; then
    warn "Zig sources are newer than libgm_native.a — rebuilding"
    needs_zig_build=1
fi

if [[ $needs_zig_build -eq 1 ]]; then
    step "Building Zig native layer (libgm_native.a)…"
    (cd "$ROOT/zig_native" && zig build --release=safe) \
        || die "Zig build failed. See output above."
    step "Zig native layer built ✓"
else
    step "Zig native layer up to date ✓"
fi

# ── Select Cargo binary ───────────────────────────────────────────────────────
case "$MODE" in
    cli)     BIN="git-manager"          ;;
    web)     BIN="git-manager-web"      ;;
    desktop) BIN="git-manager-desktop"  ;;
    build)   BIN=""                     ;;
esac

# ── Build ─────────────────────────────────────────────────────────────────────
if [[ -z "$BIN" ]]; then
    step "Building all workspace binaries…"
    cargo build --workspace "${CARGO_FLAGS[@]}"
    step "Build complete ✓"
    echo
    echo "Binaries written to: $TARGET_DIR"
    exit 0
else
    step "Building $BIN…"
    cargo build -p "$(echo "$BIN" | tr '-' '_' | sed 's/git_manager$/git-manager/')" \
        --bin "$BIN" "${CARGO_FLAGS[@]}" 2>/dev/null \
    || cargo build --bin "$BIN" "${CARGO_FLAGS[@]}"
fi

# ── Environment summary ───────────────────────────────────────────────────────
DB_URL="${GIT_MANAGER_DB_URL:-mysql://git_manager:dev_password@localhost/git_manager_dev}"
WEB_ADDR="${GIT_MANAGER_WEB_ADDR:-127.0.0.1:5000}"
LOG="${RUST_LOG:-info}"

echo
echo "${bold}── Runtime config ──────────────────────────────────────────────────${reset}"
echo "  DB  : $DB_URL"
[[ "$MODE" == "web" ]] && echo "  Addr: http://$WEB_ADDR"
echo "  Log : $LOG"
echo

# ── Run ───────────────────────────────────────────────────────────────────────
step "Starting ${BIN}…"
echo "────────────────────────────────────────────────────────────────────────"

export GIT_MANAGER_DB_URL="$DB_URL"
export GIT_MANAGER_WEB_ADDR="$WEB_ADDR"
export RUST_LOG="$LOG"

exec "$TARGET_DIR/$BIN"
