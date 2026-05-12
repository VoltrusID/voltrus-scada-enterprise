#!/usr/bin/env bash
set -euo pipefail

# ── Voltrus cross-compile release script ──
# Builds all targets via cargo-zigbuild and uploads to Cloudflare R2.
#
# Usage:
#   ./release.sh                  # build all + upload
#   ./release.sh --build-only     # build only, skip upload
#   ./release.sh --upload-only    # skip build, upload existing artifacts
#
# Requires: zig, cargo-zigbuild, rustup targets, CF_API_TOKEN + CF_ACCOUNT_ID env vars

VERSION="$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"
TAG="v${VERSION}"
R2_BUCKET="voltrus-releases"
BUILD_DIR="target/release-artifacts"
FD_LIMIT=65536

# Targets: name, rust target, binary extension
TARGETS=(
  "macos-arm64|aarch64-apple-darwin|"
  "linux-arm64|aarch64-unknown-linux-musl|"
  "linux-x86_64|x86_64-unknown-linux-musl|"
  "windows-x86_64|x86_64-pc-windows-gnu|.exe"
)

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }

# ── Parse args ──
BUILD=true
UPLOAD=true
case "${1:-}" in
  --build-only) UPLOAD=false ;;
  --upload-only) BUILD=false ;;
  "") ;;
  *) error "Unknown argument: $1. Use --build-only or --upload-only" ;;
esac

# ── Preflight ──
if [ "$BUILD" = true ]; then
  command -v zig >/dev/null 2>&1 || error "zig not found. Install: brew install zig"
  command -v cargo zigbuild >/dev/null 2>&1 || error "cargo-zigbuild not found. Install: cargo install cargo-zigbuild"
fi

if [ "$UPLOAD" = true ]; then
  [ -z "${CF_API_TOKEN:-}" ] && error "CF_API_TOKEN env var not set"
  [ -z "${CF_ACCOUNT_ID:-}" ] && error "CF_ACCOUNT_ID env var not set"
fi

info "Voltrus ${TAG}"

# ── Build frontend ──
build_frontend() {
  info "Building frontend..."
  (
    cd client
    bun install --frozen-lockfile 2>/dev/null || bun install
    bun run build
    mkdir -p ../static
    cp -r dist/* ../static/
  )
}

# ── Build one target ──
build_target() {
  local name="$1"
  local target="$2"
  local ext="$3"

  info "Building ${name} (${target})..."

  # Ensure target is installed
  rustup target add "$target" 2>/dev/null || true

  local bin_voltrus="target/${target}/release/voltrus${ext}"
  local bin_modbus="target/${target}/release/modbus-sim${ext}"

  # Build (native for macOS, zigbuild for cross)
  if [[ "$target" == *"-apple-"* ]]; then
    bash -c "ulimit -n ${FD_LIMIT} && cargo build --release --target ${target} --bin voltrus --bin modbus-sim"
  else
    bash -c "ulimit -n ${FD_LIMIT} && cargo zigbuild --release --target ${target} --bin voltrus --bin modbus-sim"
  fi

  # Verify
  [ -f "$bin_voltrus" ] || error "Build failed: $bin_voltrus not found"
  [ -f "$bin_modbus" ] || warn "modbus-sim not found for ${name} (may need feature flag)"

  # Package
  local pkg_dir="${BUILD_DIR}/${name}"
  mkdir -p "$pkg_dir"
  cp "$bin_voltrus" "$pkg_dir/"
  [ -f "$bin_modbus" ] && cp "$bin_modbus" "$pkg_dir/"
  cp config.yaml "$pkg_dir/config.yaml.example"
  [ -f deploy/voltrus.service ] && cp deploy/voltrus.service "$pkg_dir/"
  cp -r static "$pkg_dir/static"

  # Compress
  local archive
  if [[ "$name" == *"windows"* ]]; then
    archive="${BUILD_DIR}/voltrus-${TAG}-${name}.zip"
    (cd "${BUILD_DIR}" && zip -r -q "$(basename "$archive")" "${name}/")
  else
    archive="${BUILD_DIR}/voltrus-${TAG}-${name}.tar.gz"
    COPYFILE_DISABLE=1 tar --no-xattrs -czf "$archive" -C "$pkg_dir" .
  fi

  local size
  size=$(stat -f%z "$archive" 2>/dev/null || stat -c%s "$archive" 2>/dev/null)
  info "  -> $(basename "$archive") ($(( size / 1048576 )) MB)"
}

# ── Upload to R2 ──
upload_to_r2() {
  local file="$1"
  local key="$2"
  info "Uploading $(basename "$file") to R2..."

  local http_code
  http_code=$(curl -s -o /dev/null -w "%{http_code}" -X PUT \
    "https://api.cloudflare.com/client/v4/accounts/${CF_ACCOUNT_ID}/r2/buckets/${R2_BUCKET}/objects/${key}" \
    -H "Authorization: Bearer ${CF_API_TOKEN}" \
    -H "Content-Type: application/octet-stream" \
    --data-binary @"$file")

  if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 300 ]; then
    info "  -> OK (${http_code})"
  else
    error "  -> FAILED (HTTP ${http_code})"
  fi
}

# ── Main ──
mkdir -p "$BUILD_DIR"

if [ "$BUILD" = true ]; then
  build_frontend

  for entry in "${TARGETS[@]}"; do
    IFS='|' read -r name target ext <<< "$entry"
    build_target "$name" "$target" "$ext"
  done
fi

if [ "$UPLOAD" = true ]; then
  info "Uploading to R2..."

  for entry in "${TARGETS[@]}"; do
    IFS='|' read -r name target ext <<< "$entry"

    if [[ "$name" == *"windows"* ]]; then
      archive="${BUILD_DIR}/voltrus-${TAG}-${name}.zip"
    else
      archive="${BUILD_DIR}/voltrus-${TAG}-${name}.tar.gz"
    fi

    [ -f "$archive" ] || error "Archive not found: $archive"

    upload_to_r2 "$archive" "voltrus-${TAG}/${name}/$(basename "$archive")"
    upload_to_r2 "$archive" "voltrus-latest/${name}/$(basename "$archive")"
  done

  info "All uploads complete!"
fi

echo ""
info "Done! Artifacts in ${BUILD_DIR}/:"
ls -lh "${BUILD_DIR}"/*.tar.gz "${BUILD_DIR}"/*.zip 2>/dev/null | awk '{printf "  %-50s %s\n", $NF, $5}'
