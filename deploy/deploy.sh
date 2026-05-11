#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="/opt/voltrus"
REPO="dzulfikar08/voltrus"
SERVICES=(voltrus modbus-sim modbus-sim-1)
HEALTH_URL="http://localhost:3000/api/health"
HEALTH_TIMEOUT=30

VERSION=${1:-${VERSION:-latest}}

if [ "$VERSION" = "latest" ]; then
  VERSION="$(gh release list --repo "${REPO}" --limit 1 --json tagName --jq '.[0].tagName')"
  echo "Resolved latest to ${VERSION}"
fi

echo "==> Downloading release ${VERSION}..."
ARCHIVE="voltrus-${VERSION}-linux-x86_64.tar.gz"
gh release download "${VERSION}" --repo "${REPO}" --pattern "${ARCHIVE}" --clobber

echo "==> Stopping services..."
for svc in "${SERVICES[@]}"; do
    sudo systemctl stop "$svc" || true
done
sleep 1

echo "==> Extracting to ${INSTALL_DIR}..."
sudo tar -xzf "${ARCHIVE}" -C "${INSTALL_DIR}/" --overwrite
sudo chmod +x "${INSTALL_DIR}/voltrus" "${INSTALL_DIR}/modbus-sim"

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
rm -f "${ARCHIVE}"
