#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="/opt/voltrus-lite"
REPO="dzulfikar08/voltrus-lite-rust-service"
SERVICES=(voltrus-lite modbus-sim modbus-sim-1)
HEALTH_URL="http://localhost:3000/api/health"
HEALTH_TIMEOUT=30

VERSION=${1:-${VERSION:-latest}}

echo "==> Downloading release ${VERSION}..."
gh release download "${VERSION}" --repo "${REPO}" --clobber

echo "==> Stopping services..."
for svc in "${SERVICES[@]}"; do
    sudo systemctl stop "$svc" || true
done
sleep 1

echo "==> Extracting to ${INSTALL_DIR}..."
sudo tar -xzf ~/voltrus-lite-rust-service-linux-x86_64.tar.gz -C "${INSTALL_DIR}/" --overwrite
sudo chmod +x "${INSTALL_DIR}/voltrus-lite-rust-service" "${INSTALL_DIR}/modbus-sim"

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
        sudo systemctl status voltrus-lite --no-pager
        exit 1
    fi
done

echo "==> Deploy complete! Service is healthy."
rm -f ~/voltrus-lite-rust-service-linux-x86_64.tar.gz
