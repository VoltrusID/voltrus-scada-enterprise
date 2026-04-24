#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="/opt/voltrus-lite"
BINARY_NAME="voltrus-lite-rust-service"
SERVICE_NAME="voltrus-lite"
REPO="DzulfikarAs/voltrus-lite-rust-service"
HEALTH_URL="http://localhost:3000/api/health"
HEALTH_TIMEOUT=30

echo "==> Fetching latest release from GitHub..."
LATEST_URL=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"browser_download_url".*linux-x86_64.tar.gz' \
    | head -1 \
    | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_URL" ]; then
    echo "ERROR: Could not find latest release binary"
    exit 1
fi

echo "==> Latest release: ${LATEST_URL}"

TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

echo "==> Downloading..."
curl -sL "$LATEST_URL" -o "${TMPDIR}/release.tar.gz"

echo "==> Extracting..."
tar -xzf "${TMPDIR}/release.tar.gz" -C "${TMPDIR}"

if [ ! -f "${TMPDIR}/${BINARY_NAME}" ]; then
    echo "ERROR: Binary not found in release archive"
    exit 1
fi

echo "==> Stopping service gracefully..."
sudo systemctl stop "${SERVICE_NAME}" || true
sudo systemctl stop modbus-sim || true
sleep 1

echo "==> Replacing binary..."
cp "${TMPDIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo "==> Updating static files..."
rm -rf "${INSTALL_DIR}/static"
if [ -d "${TMPDIR}/static" ]; then
    cp -r "${TMPDIR}/static" "${INSTALL_DIR}/static"
fi

echo "==> Starting service..."
sudo systemctl start "${SERVICE_NAME}"

echo "==> Running health check..."
ELAPSED=0
until curl -sf "${HEALTH_URL}" > /dev/null 2>&1; do
    sleep 1
    ELAPSED=$((ELAPSED + 1))
    if [ "$ELAPSED" -ge "$HEALTH_TIMEOUT" ]; then
        echo "ERROR: Health check failed after ${HEALTH_TIMEOUT}s"
        sudo systemctl status "${SERVICE_NAME}" --no-pager
        exit 1
    fi
done

echo "==> Deploy complete! Service is healthy."
sudo systemctl status "${SERVICE_NAME}" --no-pager
