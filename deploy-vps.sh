#!/usr/bin/env bash
set -euo pipefail

# ── Voltrus local deploy script ──
# Builds the Linux x86_64 binary locally on Mac, then deploys to VPS via SSH.
#
# Usage:
#   ./deploy-vps.sh                  # build + deploy
#   ./deploy-vps.sh --build-only     # build only, skip deploy
#   ./deploy-vps.sh --deploy-only    # skip build, deploy existing artifact
#   ./deploy-vps.sh --frontend-only  # build frontend + deploy only (no Rust rebuild)
#
# Requires:
#   - zig, cargo-zigbuild (for cross-compilation)
#   - cloudflared (for SSH tunnel)
#   - SSH config with Host "vps-voltrus" already set up

VERSION="$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"
TAG="v${VERSION}"
FD_LIMIT=65536
TARGET="x86_64-unknown-linux-musl"
BUILD_DIR="target/release-artifacts/linux-x86_64"

VPS_HOST="vps-voltrus"
VPS_INSTALL_DIR="/opt/voltrus"
VPS_SERVICES=(voltrus modbus-sim modbus-sim-1)
VPS_HEALTH_URL="http://localhost:3000/api/health"
VPS_HEALTH_TIMEOUT=30

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; exit 1; }
step()  { echo -e "${CYAN}[STEP]${NC} $*"; }

# ── Parse args ──
BUILD_RUST=true
BUILD_FRONTEND=true
DEPLOY=true
case "${1:-}" in
  --build-only)    DEPLOY=false ;;
  --deploy-only)   BUILD_RUST=false; BUILD_FRONTEND=false ;;
  --frontend-only) BUILD_RUST=false ;;
  "") ;;
  *) error "Unknown argument: $1. Use --build-only, --deploy-only, or --frontend-only" ;;
esac

# ── Preflight ──
if [ "$DEPLOY" = true ]; then
  ssh -o ConnectTimeout=10 "$VPS_HOST" "echo ok" >/dev/null 2>&1 \
    || error "Cannot reach VPS via 'ssh $VPS_HOST'. Check SSH config and cloudflared."
  info "VPS SSH connection OK"
fi

if [ "$BUILD_RUST" = true ]; then
  command -v zig >/dev/null 2>&1 || error "zig not found. Install: brew install zig"
  command -v cargo zigbuild >/dev/null 2>&1 || error "cargo-zigbuild not found. Install: cargo install cargo-zigbuild"
fi

if [ "$BUILD_FRONTEND" = true ]; then
  command -v bun >/dev/null 2>&1 || error "bun not found. Install: curl -fsSL https://bun.sh/install | bash"
fi

info "Voltrus ${TAG}"

# ── Build frontend ──
if [ "$BUILD_FRONTEND" = true ]; then
  step "Building frontend..."
  (
    cd client
    bun install --frozen-lockfile 2>/dev/null || bun install
    bun run build
    mkdir -p ../static
    cp -r dist/* ../static/
  )
  info "Frontend built"
fi

# ── Build Rust binary ──
if [ "$BUILD_RUST" = true ]; then
  step "Cross-compiling for Linux x86_64 (musl)..."
  rustup target add "$TARGET" 2>/dev/null || true
  bash -c "ulimit -n ${FD_LIMIT} && cargo zigbuild --release --target ${TARGET} --bin voltrus --bin modbus-sim"

  [ -f "target/${TARGET}/release/voltrus" ] || error "Build failed: voltrus binary not found"

  mkdir -p "$BUILD_DIR"
  cp "target/${TARGET}/release/voltrus" "$BUILD_DIR/"
  cp "target/${TARGET}/release/modbus-sim" "$BUILD_DIR/" 2>/dev/null || warn "modbus-sim not found (may need feature flag)"
  cp config.yaml "$BUILD_DIR/config.yaml.example"
  [ -f deploy/voltrus.service ] && cp deploy/voltrus.service "$BUILD_DIR/"
  rm -rf "$BUILD_DIR/static"
  cp -r static "$BUILD_DIR/static"

  local_size=$(stat -f%z "$BUILD_DIR/voltrus" 2>/dev/null || stat -c%s "$BUILD_DIR/voltrus" 2>/dev/null)
  info "Binary size: $(( local_size / 1048576 )) MB"
fi

# ── Deploy ──
if [ "$DEPLOY" = true ]; then
  step "Deploying to VPS..."

  ARCHIVE="/tmp/voltrus-deploy-${TAG}.tar.gz"
  info "Packing build artifacts..."
  COPYFILE_DISABLE=1 tar --no-xattrs -czf "$ARCHIVE" -C "$BUILD_DIR" .

  ARCHIVE_SIZE=$(stat -f%z "$ARCHIVE" 2>/dev/null || stat -c%s "$ARCHIVE" 2>/dev/null)
  info "Archive size: $(( ARCHIVE_SIZE / 1048576 )) MB"

  info "Uploading archive..."
  scp "$ARCHIVE" "${VPS_HOST}:/tmp/voltrus-deploy.tar.gz"

  # Execute remote deploy
  info "Running remote deploy..."
  ssh "$VPS_HOST" "bash -s" <<REMOTE_SCRIPT
set -euo pipefail

INSTALL_DIR="${VPS_INSTALL_DIR}"
SERVICES=(${VPS_SERVICES[*]})
HEALTH_URL="${VPS_HEALTH_URL}"
HEALTH_TIMEOUT=${VPS_HEALTH_TIMEOUT}

echo "==> Stopping services..."
for svc in "\${SERVICES[@]}"; do
  sudo systemctl stop "\$svc" || true
done
sleep 1

echo "==> Preparing install directory..."
sudo mkdir -p "\${INSTALL_DIR}"

echo "==> Unpacking archive..."
rm -rf /tmp/voltrus-deploy
mkdir -p /tmp/voltrus-deploy
tar -xzf /tmp/voltrus-deploy.tar.gz -C /tmp/voltrus-deploy 2>/dev/null

echo "==> Installing binary..."
sudo mv /tmp/voltrus-deploy/voltrus "\${INSTALL_DIR}/voltrus"
sudo chmod +x "\${INSTALL_DIR}/voltrus"

if [ -f /tmp/voltrus-deploy/modbus-sim ]; then
  sudo mv /tmp/voltrus-deploy/modbus-sim "\${INSTALL_DIR}/modbus-sim"
  sudo chmod +x "\${INSTALL_DIR}/modbus-sim"
fi

echo "==> Installing static files..."
sudo rm -rf "\${INSTALL_DIR}/static"
sudo mv /tmp/voltrus-deploy/static "\${INSTALL_DIR}/static"

echo "==> Fixing ownership..."
sudo chown -R voltrus:voltrus "\${INSTALL_DIR}/static"
sudo chown voltrus:voltrus "\${INSTALL_DIR}" "\${INSTALL_DIR}/voltrus" "\${INSTALL_DIR}/modbus-sim"
sudo chown -R voltrus:voltrus "\${INSTALL_DIR}/data" 2>/dev/null || true

echo "==> Cleaning up temp files..."
rm -rf /tmp/voltrus-deploy /tmp/voltrus-deploy.tar.gz

echo "==> Starting services..."
for svc in "\${SERVICES[@]}"; do
  sudo systemctl start "\$svc"
done

echo "==> Running health check..."
ELAPSED=0
until curl -sf "\${HEALTH_URL}" > /dev/null 2>&1; do
  sleep 1
  ELAPSED=\$((ELAPSED + 1))
  if [ "\$ELAPSED" -ge "\$HEALTH_TIMEOUT" ]; then
    echo "ERROR: Health check failed after \${HEALTH_TIMEOUT}s"
    sudo systemctl status voltrus --no-pager
    exit 1
  fi
done
echo "==> Health check passed!"

REMOTE_SCRIPT

  rm -f "$ARCHIVE"
  step "Deploy complete! Voltrus ${TAG} is live on VPS."
fi

echo ""
info "Done!"
