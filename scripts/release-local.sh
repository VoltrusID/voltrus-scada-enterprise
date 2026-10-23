#!/usr/bin/env bash
set -euo pipefail

# ── Voltrus release script (official build: open core + Enterprise) ──
# Cross-compiles via cargo-zigbuild and uploads to Cloudflare R2.
#
# Layout: the `voltrus` binary is built from enterprise/ (it links the open core in
# the core/ submodule), `modbus-sim` from core/, and the web client from core/client
# with the enterprise client module (output core/static). The packaged archives
# have the same names and layout as before the open-core split.
#
# Usage:
#   ./scripts/release-local.sh                          # All platforms
#   ./scripts/release-local.sh --target aarch64          # macOS ARM64 only
#   ./scripts/release-local.sh --target x86_64           # Linux x86_64 only
#   ./scripts/release-local.sh --skip-upload             # Build only, skip upload
#   ./scripts/release-local.sh --dry-run                 # Build, then print the upload plan; run nothing remote
#   NOTARIZE=1 ./scripts/release-local.sh                # With macOS notarization
#   NOTARIZE=1 ./scripts/release-local.sh --target aarch64
#
# Environment: BUILD_DIR (default target/release-artifacts) moves the packaged output;
# CARGO_TARGET_DIR, if set, is used for both cargo invocations.
#
# Requires:
#   - node + npm (frontend build)
#   - zig, cargo-zigbuild (cross-compilation)
#   - wrangler (R2 upload; not needed with --skip-upload or --dry-run)

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(grep '^version' enterprise/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"
TAG="v${VERSION}"
R2_BUCKET="voltrus-releases"
D1_DB="voltrus-releases-db"
BUILD_DIR="${BUILD_DIR:-target/release-artifacts}"
ENTERPRISE_TARGET_DIR="${CARGO_TARGET_DIR:-enterprise/target}"
CORE_TARGET_DIR="${CARGO_TARGET_DIR:-core/target}"
# D1 build number must sort like semver: 1.0.0 -> 1000000, 0.47.0 -> 47000.
# (Concatenating digits made 1.0.0 -> 100 sort below 0.47.0 -> 470.)
IFS=. read -r _V_MAJ _V_MIN _V_PAT <<< "${VERSION%%-*}"
VERSION_NUM=$(( 10#${_V_MAJ:-0} * 1000000 + 10#${_V_MIN:-0} * 1000 + 10#${_V_PAT:-0} ))
FD_LIMIT=65536
NOTARIZE="${NOTARIZE:-0}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; exit 1; }
step()  { echo -e "${CYAN}[STEP]${NC} $*"; }

# ── Target lookup (bash 3.2 compatible) ──
resolve_target() {
  case "$1" in
    aarch64|macos-arm64)   echo "macos-arm64|aarch64-apple-darwin|" ;;
    linux-arm64)           echo "linux-arm64|aarch64-unknown-linux-musl|" ;;
    linux-x86_64|x86_64)   echo "linux-x86_64|x86_64-unknown-linux-musl|" ;;
    windows-x86_64)        echo "windows-x86_64|x86_64-pc-windows-gnu|.exe" ;;
    all)                   echo "" ;;
    *)                     return 1 ;;
  esac
}

ALL_TARGETS=(
  "macos-arm64|aarch64-apple-darwin|"
  "linux-arm64|aarch64-unknown-linux-musl|"
  "linux-x86_64|x86_64-unknown-linux-musl|"
  "windows-x86_64|x86_64-pc-windows-gnu|.exe"
)

# ── Parse args ──
UPLOAD=true
DRY_RUN=false
TARGET_FILTER=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-upload) UPLOAD=false ;;
    --dry-run) DRY_RUN=true ;;
    --target)
      shift
      TARGET_FILTER="${1:-}"
      [ -z "$TARGET_FILTER" ] && error "--target requires an argument"
      ;;
    *) error "Unknown argument: $1. Use --target <platform>, --skip-upload or --dry-run" ;;
  esac
  shift
done

# Resolve targets to build
TARGETS=()
if [ -n "$TARGET_FILTER" ] && [ "$TARGET_FILTER" != "all" ]; then
  entry="$(resolve_target "$TARGET_FILTER")" || error "Unknown target '$TARGET_FILTER'. Valid: aarch64 macos-arm64 linux-arm64 linux-x86_64 x86_64 windows-x86_64 all"
  TARGETS=("$entry")
else
  TARGETS=("${ALL_TARGETS[@]}")
fi

# ── Preflight ──
command -v node >/dev/null 2>&1 || error "node not found"
command -v npm >/dev/null 2>&1 || error "npm not found"
command -v zig >/dev/null 2>&1 || error "zig not found. Install: brew install zig"
command -v cargo >/dev/null 2>&1 || error "cargo not found"

if [ "$UPLOAD" = true ] && [ "$DRY_RUN" = false ]; then
  command -v wrangler >/dev/null 2>&1 || error "wrangler not found. Install: npm install -g wrangler"
fi

if [ "$NOTARIZE" = "1" ]; then
  command -v notarytool >/dev/null 2>&1 || error "notarytool not found (requires Xcode)"
  [ -z "${APPLE_ID:-}" ] && error "APPLE_ID env var not set"
  [ -z "${APPLE_TEAM_ID:-}" ] && error "APPLE_TEAM_ID env var not set"
  [ -z "${APPLE_APP_PASSWORD:-}" ] && error "APPLE_APP_PASSWORD env var not set"
fi

[ -f core/Cargo.toml ] || git submodule update --init core
[ -f core/Cargo.toml ] || error "core submodule missing: run git submodule update --init core"

info "Voltrus ${TAG}"
info "Targets: $(for t in "${TARGETS[@]}"; do IFS='|' read -r n _ _ <<< "$t"; echo -n "$n "; done)"
$DRY_RUN && info "Dry run: nothing is uploaded and no remote command runs"

# ── Build frontend (core client + enterprise client module -> core/static) ──
step "Building frontend..."
(
  cd core/client
  npm ci
  VOLTRUS_ENTERPRISE_CLIENT="$ROOT/enterprise/client/src/index.ts" npm run build
)
[ -f core/static/index.html ] || error "Frontend build did not produce core/static/index.html"
info "Frontend built"

# ── Build one target ──
build_target() {
  local name="$1"
  local target="$2"
  local ext="$3"

  info "Building ${name} (${target})..."

  rustup target add "$target" 2>/dev/null || true

  local bin_voltrus="${ENTERPRISE_TARGET_DIR}/${target}/release/voltrus${ext}"
  local bin_modbus="${CORE_TARGET_DIR}/${target}/release/modbus-sim${ext}"

  local build_cmd="cargo zigbuild"
  [[ "$target" == *"-apple-"* ]] && build_cmd="cargo build"

  # The official `voltrus` binary comes from the enterprise crate, modbus-sim from core.
  bash -c "ulimit -n ${FD_LIMIT} && cd enterprise && ${build_cmd} --release --locked --target ${target} --bin voltrus"
  bash -c "ulimit -n ${FD_LIMIT} && ${build_cmd} --release --locked --manifest-path core/Cargo.toml --target ${target} --bin modbus-sim"

  [ -f "$bin_voltrus" ] || error "Build failed: $bin_voltrus not found"
  [ -f "$bin_modbus" ] || warn "modbus-sim not found for ${name}"

  local pkg_dir="${BUILD_DIR}/${name}"
  rm -rf "$pkg_dir"
  mkdir -p "$pkg_dir"
  cp "$bin_voltrus" "$pkg_dir/"
  [ -f "$bin_modbus" ] && cp "$bin_modbus" "$pkg_dir/"
  cp core/config.yaml "$pkg_dir/config.yaml.example"
  [ -f core/deploy/voltrus.service ] && cp core/deploy/voltrus.service "$pkg_dir/"
  cp -r core/static "$pkg_dir/static"

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

  # Notarize macOS builds
  if [ "$NOTARIZE" = "1" ] && [[ "$name" == *"macos"* ]]; then
    step "Notarizing ${name}..."
    local tmp_zip="/tmp/voltrus-notarize-${name}.zip"
    cp "$archive" "$tmp_zip"
    xcrun notarytool submit "$tmp_zip" \
      --apple-id "$APPLE_ID" \
      --team-id "$APPLE_TEAM_ID" \
      --password "$APPLE_APP_PASSWORD" \
      --wait
    rm -f "$tmp_zip"
    info "  -> Notarization complete"
  fi
}

# ── Upload to R2 ──
upload_to_r2() {
  local file="$1"
  local key="$2"
  if [ "$DRY_RUN" = true ]; then
    info "[dry-run] would upload $(basename "$file") -> ${R2_BUCKET}/${key}"
    return
  fi
  info "Uploading $(basename "$file") -> ${key}"
  wrangler r2 object put "${R2_BUCKET}/${key}" --file "$file" --remote 2>&1 | grep -E "complete|error|Error" || true
}

# ── D1 statement (printed, not run, in a dry run) ──
d1_execute() {
  local sql="$1"
  if [ "$DRY_RUN" = true ]; then
    info "[dry-run] would run on ${D1_DB}: ${sql}"
    return
  fi
  wrangler d1 execute "${D1_DB}" --remote --command "$sql" 2>&1 | grep -E "Executed|error|Error" || true
}

# ── Main ──
mkdir -p "$BUILD_DIR"

step "Building targets..."
for entry in "${TARGETS[@]}"; do
  IFS='|' read -r name target ext <<< "$entry"
  build_target "$name" "$target" "$ext"
done

if [ "$UPLOAD" = true ]; then
  step "Uploading to R2..."
  for entry in "${TARGETS[@]}"; do
    IFS='|' read -r name target ext <<< "$entry"

    if [[ "$name" == *"windows"* ]]; then
      archive="${BUILD_DIR}/voltrus-${TAG}-${name}.zip"
    else
      archive="${BUILD_DIR}/voltrus-${TAG}-${name}.tar.gz"
    fi

    [ -f "$archive" ] || error "Archive not found: $archive"

    upload_to_r2 "$archive" "voltrus/${TAG}/${name}/$(basename "$archive")"
    upload_to_r2 "$archive" "voltrus/latest/${name}/$(basename "$archive")"
  done
  if [ "$DRY_RUN" = true ]; then info "Dry run: R2 upload plan printed, nothing sent"; else info "All uploads complete!"; fi

  # ── Sync D1 release metadata ──
  step "Syncing D1 release metadata..."
  PUB_DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  SIZE_SUM=0
  for entry in "${TARGETS[@]}"; do
    IFS='|' read -r name _ _ <<< "$entry"
    for f in "${BUILD_DIR}"/voltrus-${TAG}-${name}.*; do
      [ -f "$f" ] || continue
      s=$(stat -f%z "$f" 2>/dev/null || stat -c%s "$f" 2>/dev/null)
      SIZE_SUM=$((SIZE_SUM + s))
    done
  done
  d1_execute "INSERT OR REPLACE INTO releases (version, short_version_string, title, description, length, pub_date) VALUES (${VERSION_NUM}, '${VERSION}', 'Voltrus ${TAG}', 'Voltrus SCADA ${TAG} release', ${SIZE_SUM}, '${PUB_DATE}')"

  for entry in "${TARGETS[@]}"; do
    IFS='|' read -r name _ _ <<< "$entry"

    if [[ "$name" == *"windows"* ]]; then
      archive="${BUILD_DIR}/voltrus-${TAG}-${name}.zip"
    else
      archive="${BUILD_DIR}/voltrus-${TAG}-${name}.tar.gz"
    fi

    size=$(stat -f%z "$archive" 2>/dev/null || stat -c%s "$archive" 2>/dev/null)
    r2_key="voltrus/latest/${name}/$(basename "$archive")"
    filename="$(basename "$archive")"

    d1_execute "INSERT OR REPLACE INTO release_files (version, platform, filename, r2_key, size) VALUES (${VERSION_NUM}, '${name}', '${filename}', '${r2_key}', ${size})"
    info "  D1: ${name} -> ${r2_key} (${size} bytes)"
  done
  if [ "$DRY_RUN" = false ]; then info "D1 sync complete!"; fi
fi

echo ""
info "Done! Artifacts:"
for entry in "${TARGETS[@]}"; do
  IFS='|' read -r name _ _ <<< "$entry"
  for f in "${BUILD_DIR}"/voltrus-${TAG}-${name}.*; do
    [ -f "$f" ] || continue
    size=$(stat -f%z "$f" 2>/dev/null || stat -c%s "$f" 2>/dev/null)
    printf "  %-60s %s\n" "$(basename "$f")" "$(( size / 1048576 )) MB"
  done
done
