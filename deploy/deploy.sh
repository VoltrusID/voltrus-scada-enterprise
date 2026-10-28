#!/usr/bin/env bash
set -euo pipefail

# Deploy an official Voltrus release (open core plus Enterprise) to this host.
#
# release.sh publishes releases to the R2 bucket voltrus-releases and the download worker
# serves them at dl-voltrus-scada.voltrus.id; there are no releases to fetch from GitHub
# with `gh`. "latest" follows the worker's /latest redirect (highest version in D1). A
# version (v0.47.1 or 0.47.1) is read from voltrus/<tag>/<platform>/ in the bucket.
#
# Usage: deploy.sh [latest|VERSION]     (or VERSION=... deploy.sh)

INSTALL_DIR="/opt/voltrus"
DOWNLOAD_BASE="${VOLTRUS_DOWNLOAD_BASE:-https://dl-voltrus-scada.voltrus.id}"
PLATFORM="linux-x86_64"
SERVICES=(voltrus modbus-sim modbus-sim-1)
HEALTH_URL="http://localhost:3000/api/health"
HEALTH_TIMEOUT=30

VERSION=${1:-${VERSION:-latest}}

if [ "$VERSION" = "latest" ]; then
  URL="$(curl -fsS -o /dev/null -w '%{redirect_url}' "${DOWNLOAD_BASE}/latest?platform=${PLATFORM}")" \
    || { echo "ERROR: could not resolve the latest ${PLATFORM} release from ${DOWNLOAD_BASE}" >&2; exit 1; }
  [ -n "$URL" ] || { echo "ERROR: ${DOWNLOAD_BASE} has no ${PLATFORM} release" >&2; exit 1; }
  ARCHIVE="$(basename "$(printf '%s' "$URL" | sed 's/%2F/\//g')")"
  VERSION="$(printf '%s' "$ARCHIVE" | sed -E 's/^voltrus-(v[^-]+)-.*/\1/')"
  echo "Resolved latest to ${VERSION}"
else
  case "$VERSION" in v*) ;; *) VERSION="v${VERSION}" ;; esac
  ARCHIVE="voltrus-${VERSION}-${PLATFORM}.tar.gz"
  URL="${DOWNLOAD_BASE}/releases/voltrus%2F${VERSION}%2F${PLATFORM}%2F${ARCHIVE}"
fi

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

echo "==> Downloading ${ARCHIVE}..."
curl -fL --retry 3 -o "${WORKDIR}/${ARCHIVE}" "$URL"

echo "==> Stopping services..."
for svc in "${SERVICES[@]}"; do
    sudo systemctl stop "$svc" || true
done
sleep 1

echo "==> Extracting to ${INSTALL_DIR}..."
sudo tar -xzf "${WORKDIR}/${ARCHIVE}" -C "${INSTALL_DIR}/" --overwrite
sudo chmod +x "${INSTALL_DIR}/voltrus" "${INSTALL_DIR}/modbus-sim"
# The web UI is inside the binary. A static/ folder from an older release would override it
# and keep serving the old UI, so remove it.
sudo rm -rf "${INSTALL_DIR}/static"

echo "==> Starting services..."
for svc in "${SERVICES[@]}"; do
    sudo systemctl start "$svc"
done

echo "==> Running health check..."
ELAPSED=0
until curl -sf "${HEALTH_URL}" > /dev/null 2>&1; do
    sleep 1
    ELAPSED=$((ELAPSED + 1))
    if [ "$ELAPSED" -ge "$HEALTH_TIMEOUT" ]; then
        echo "ERROR: Health check failed after ${HEALTH_TIMEOUT}s"
        sudo systemctl status voltrus --no-pager
        exit 1
    fi
done

echo "==> Deploy complete! Service is healthy."
