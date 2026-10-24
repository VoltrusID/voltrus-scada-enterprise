#!/usr/bin/env bash
set -euo pipefail

# ── Voltrus cross-compile release script (official build: open core + Enterprise) ──
# Builds all targets via cargo-zigbuild and uploads to Cloudflare R2 + D1.
#
# Layout: the `voltrus` binary is built from enterprise/ (it links the open core in
# the core/ submodule), `modbus-sim` from core/, and the web client from core/client
# with the enterprise client module (output core/static). The packaged archives
# have the same names and layout as before the open-core split.
#
# Usage:
#   ./release.sh                  # build all + upload
#   ./release.sh --build-only     # build only, skip upload (alias: --skip-upload)
#   ./release.sh --upload-only    # skip build, upload existing artifacts
#   ./release.sh --dry-run        # build all, then print the upload plan; nothing remote runs
#   ./release.sh --host           # native glibc build of the host (linux-x86_64 layout) to check the
#                                 #   packaging; never uploads (implies --skip-upload)
#   ./release.sh --target linux-x86_64 [...]   # limit to one platform (also: macos-arm64,
#                                              #   linux-arm64, windows-x86_64)
#
# Environment: BUILD_DIR (default target/release-artifacts) moves the packaged output;
# CARGO_TARGET_DIR, if set, is used for both cargo invocations; DESCRIPTION is the
# release note stored in D1.
#
# Upload: R2 bucket voltrus-releases, keys voltrus/<tag>/<platform>/<archive> and
# voltrus/latest/<platform>/<archive>, then D1 voltrus-releases-db (release_files, releases)
# once every object is in. dl-voltrus-scada.voltrus.id serves the highest D1 version.
# CI builds the archives on a tag (.github/workflows/release.yml) and attaches them to the
# GitHub release; download them into BUILD_DIR and run --upload-only to publish.
#
# Requires: node + npm, zig, cargo-zigbuild, rustup targets
# Upload requires: wrangler (via worker/license-activate/node_modules)

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

VERSION="$(grep '^version' enterprise/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"
TAG="v${VERSION}"
R2_BUCKET="voltrus-releases"
BUILD_DIR="${BUILD_DIR:-target/release-artifacts}"
ENTERPRISE_TARGET_DIR="${CARGO_TARGET_DIR:-enterprise/target}"
CORE_TARGET_DIR="${CARGO_TARGET_DIR:-core/target}"
FD_LIMIT=65536
# D1 version number sorts like semver (0.47.1 -> 47001, 1.0.0 -> 1000000): /latest and /downloads
# serve the highest number. Concatenated digits sorted 1.0.0 below 0.47.0.
IFS=. read -r _V_MAJ _V_MIN _V_PAT <<< "${VERSION%%-*}"
VERSION_NUM=$(( 10#${_V_MAJ:-0} * 1000000 + 10#${_V_MIN:-0} * 1000 + 10#${_V_PAT:-0} ))
# Object layout the download worker serves: voltrus/<tag>/<platform>/<file> and voltrus/latest/<platform>/<file>.
R2_PREFIX="voltrus"

file_size() { stat -c%s "$1" 2>/dev/null || stat -f%z "$1"; }
DESCRIPTION="${DESCRIPTION:-Bug fixes and improvements.}"
WORKER_DIR="$ROOT/worker/license-activate"

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
BLUE='\033[0;34m'
NC='\033[0m'

info()  { echo -e "${BLUE}[*]${NC} $*"; }
ok()    { echo -e "${GREEN}[✓]${NC} $*"; }
warn()  { echo -e "${YELLOW}[!]${NC} $*"; }
step()  { echo -e "${BLUE}[*]${NC} $*"; }
error() { echo -e "${RED}[✗]${NC} $*" >&2; exit 1; }

# ── Parse args ──
BUILD=true
UPLOAD=true
DRY_RUN=false
HOST_BUILD=false
TARGET_FILTER=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --build-only|--skip-upload) UPLOAD=false ;;
    --upload-only) BUILD=false ;;
    --dry-run) DRY_RUN=true ;;
    --host) HOST_BUILD=true; UPLOAD=false ;;
    --target)
      shift
      TARGET_FILTER="${1:-}"
      [ -z "$TARGET_FILTER" ] && error "--target requires an argument"
      ;;
    *) error "Unknown argument: $1. Use --build-only, --skip-upload, --upload-only, --dry-run or --target <platform>" ;;
  esac
  shift
done
[ "$DRY_RUN" = true ] && [ "$BUILD" = false ] && info "Dry run with --upload-only: only the upload plan is printed"

if [ "$HOST_BUILD" = true ]; then
  TARGETS=("linux-x86_64|x86_64-unknown-linux-gnu|")
elif [ -n "$TARGET_FILTER" ]; then
  FILTERED=()
  for entry in "${TARGETS[@]}"; do
    IFS='|' read -r name _ _ <<< "$entry"
    [ "$name" = "$TARGET_FILTER" ] && FILTERED+=("$entry")
  done
  [ ${#FILTERED[@]} -gt 0 ] || error "Unknown target '$TARGET_FILTER'. Valid: macos-arm64 linux-arm64 linux-x86_64 windows-x86_64"
  TARGETS=("${FILTERED[@]}")
fi

# ── Preflight ──
if [ "$BUILD" = true ]; then
  command -v node >/dev/null 2>&1 || error "node not found"
  command -v npm >/dev/null 2>&1 || error "npm not found"
  [ "$HOST_BUILD" = true ] || command -v zig >/dev/null 2>&1 || error "zig not found. Install: brew install zig"
  command -v cargo >/dev/null 2>&1 || error "cargo not found"
  [ "$HOST_BUILD" = true ] || command cargo zigbuild --help >/dev/null 2>&1 || error "cargo-zigbuild not found. Install: cargo install cargo-zigbuild"
  [ -f core/Cargo.toml ] || git submodule update --init core
  [ -f core/Cargo.toml ] || error "core submodule missing: run git submodule update --init core"
fi

if [ "$UPLOAD" = true ] && [ "$DRY_RUN" = false ]; then
  if [ ! -d "$WORKER_DIR/node_modules" ]; then
    info "Installing wrangler dependencies..."
    (cd "$WORKER_DIR" && npm install)
  fi
  ok "Wrangler ready"
fi

info "Voltrus ${TAG}"
[ "$DRY_RUN" = true ] && info "Dry run: nothing is uploaded and no remote command runs"

# ── Build frontend (core client + enterprise client module -> core/static) ──
build_frontend() {
  info "Building frontend..."
  (
    cd core/client
    npm ci
    VOLTRUS_ENTERPRISE_CLIENT="$ROOT/enterprise/client/src/index.ts" npm run build
  )
  [ -f core/static/index.html ] || error "Frontend build did not produce core/static/index.html"
}

# ── Build one target ──
build_target() {
  local name="$1"
  local target="$2"
  local ext="$3"

  info "Building ${name} (${target})..."

  # Ensure target is installed
  rustup target add "$target" 2>/dev/null || true

  local bin_voltrus="${ENTERPRISE_TARGET_DIR}/${target}/release/voltrus${ext}"
  local bin_modbus="${CORE_TARGET_DIR}/${target}/release/modbus-sim${ext}"

  # Build (native for macOS, zigbuild for cross). The official `voltrus` binary comes
  # from the enterprise crate, modbus-sim from core.
  local build_cmd="cargo zigbuild"
  { [[ "$target" == *"-apple-"* ]] || [ "$HOST_BUILD" = true ]; } && build_cmd="cargo build"
  bash -c "ulimit -n ${FD_LIMIT} && cd enterprise && ${build_cmd} --release --locked --target ${target} --bin voltrus"
  bash -c "ulimit -n ${FD_LIMIT} && ${build_cmd} --release --locked --manifest-path core/Cargo.toml --target ${target} --bin modbus-sim"

  # Verify
  [ -f "$bin_voltrus" ] || error "Build failed: $bin_voltrus not found"
  [ -f "$bin_modbus" ] || warn "modbus-sim not found for ${name} (may need feature flag)"

  # Package
  local pkg_dir="${BUILD_DIR}/${name}"
  rm -rf "$pkg_dir"
  mkdir -p "$pkg_dir"
  cp "$bin_voltrus" "$pkg_dir/"
  [ -f "$bin_modbus" ] && cp "$bin_modbus" "$pkg_dir/"
  cp core/config.yaml "$pkg_dir/config.yaml.example"
  [ -f core/deploy/voltrus.service ] && cp core/deploy/voltrus.service "$pkg_dir/"
  cp -r core/static "$pkg_dir/static"

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
  size=$(file_size "$archive")
  ok "  -> $(basename "$archive") ($(( size / 1048576 )) MB)"
}

# ── Upload to R2 via wrangler ──
upload_to_r2() {
  local file="$1"
  local key="$2"
  if [ "$DRY_RUN" = true ]; then
    info "[dry-run] would upload $(basename "$file") to ${R2_BUCKET}/${key}"
    return
  fi
  info "Uploading $(basename "$file") to R2..."
  (cd "$WORKER_DIR" && npx wrangler r2 object put "${R2_BUCKET}/${key}" --file="$file" --remote 2>&1) | tail -3
  ok "  -> Uploaded: $key"
}

# ── D1 statement (printed, not run, in a dry run) ──
d1_execute() {
  local sql="$1"
  if [ "$DRY_RUN" = true ]; then
    info "[dry-run] would run on voltrus-releases-db: ${sql}"
    return
  fi
  (cd "$WORKER_DIR" && npx wrangler d1 execute voltrus-releases-db --remote --command "$sql" 2>&1) | tail -1
}

# ── Insert release record into D1 ──
insert_d1_release() {
  local platform="$1"
  local filename="$2"
  local r2_key="$3"
  local size="$4"

  info "Inserting $platform into D1..."
  d1_execute "INSERT OR REPLACE INTO release_files (version, platform, filename, r2_key, size) VALUES ($VERSION_NUM, '$platform', '$filename', '$r2_key', $size);"
  ok "  -> D1 record: $platform"
}

insert_d1_release_meta() {
  local filename="$1"
  local size="$2"
  local pub_date
  pub_date=$(date -u +"%Y-%m-%dT%H:%M:%S.000Z")
  local title="Voltrus ${TAG}"
  local desc_esc="${DESCRIPTION//\'/\'\'}"

  info "Updating D1 releases table..."
  d1_execute "INSERT OR REPLACE INTO releases (version, short_version_string, title, description, length, pub_date) VALUES ($VERSION_NUM, '$VERSION', '$title', '$desc_esc', $size, '$pub_date');"
  ok "  -> D1 releases record updated"
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
  step "Uploading to R2..."

  archive_for() {
    if [[ "$1" == *"windows"* ]]; then
      echo "${BUILD_DIR}/voltrus-${TAG}-${1}.zip"
    else
      echo "${BUILD_DIR}/voltrus-${TAG}-${1}.tar.gz"
    fi
  }

  for entry in "${TARGETS[@]}"; do
    IFS='|' read -r name target ext <<< "$entry"
    archive="$(archive_for "$name")"
    [ -f "$archive" ] || error "Archive not found: $archive"
    upload_to_r2 "$archive" "${R2_PREFIX}/${TAG}/${name}/$(basename "$archive")"
    upload_to_r2 "$archive" "${R2_PREFIX}/latest/${name}/$(basename "$archive")"
  done

  # The download worker serves the highest D1 version, so the records go in only after every
  # object is in R2: a failed upload never leaves a half-published release.
  step "Updating D1..."
  total_size=0
  for entry in "${TARGETS[@]}"; do
    IFS='|' read -r name target ext <<< "$entry"
    archive="$(archive_for "$name")"
    bytes=$(file_size "$archive")
    total_size=$((total_size + bytes))
    local_filename="$(basename "$archive")"
    insert_d1_release "$name" "$local_filename" "${R2_PREFIX}/latest/${name}/${local_filename}" "$bytes"
  done
  insert_d1_release_meta "voltrus-${TAG}" "$total_size"

  if [ "$DRY_RUN" = true ]; then ok "Dry run finished: upload plan printed, nothing sent"; else ok "All uploads complete!"; fi
fi

echo ""
info "Done! Artifacts in ${BUILD_DIR}/:"
ls -lh "${BUILD_DIR}"/*.tar.gz "${BUILD_DIR}"/*.zip 2>/dev/null | awk '{printf "  %-50s %s\n", $NF, $5}' || true

if [ "$UPLOAD" = true ] && [ "$DRY_RUN" = false ]; then
  echo ""
  echo -e "  ${YELLOW}Deploy worker if updated:${NC}"
  echo -e "    cd $WORKER_DIR && npx wrangler deploy"
fi
