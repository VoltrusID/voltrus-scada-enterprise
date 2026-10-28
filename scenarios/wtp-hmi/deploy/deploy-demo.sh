#!/usr/bin/env bash
# Deploy the WTP showcase to demo.voltrus.id (shared voltrus VPS, SSH host "voltrus-vps").
#
#   scenarios/wtp-hmi/deploy/deploy-demo.sh            build + deploy
#   scenarios/wtp-hmi/deploy/deploy-demo.sh --no-build deploy the last build
#
# Installs to /opt/voltrus-scada as user voltrus-scada: the server on 127.0.0.1:3310 and the
# plant simulator on 127.0.0.1:5030. Public access is the host's Cloudflare tunnel
# (/etc/cloudflared/config.yml ingress demo.voltrus.id -> http://127.0.0.1:3310), set up once.
# config.yaml and data/ on the VPS are never overwritten. The web UI is embedded in the binary
# (core/client is built first, then the release build embeds core/static), so no static/ is deployed.
set -euo pipefail

HOST="${VPS_HOST:-voltrus-vps}"
REPO="$(cd "$(dirname "$0")/../../.." && pwd)"
DEPLOY="$REPO/scenarios/wtp-hmi/deploy"
SIM="$REPO/core/scenarios/wtp-hmi"
TARGET="x86_64-unknown-linux-musl"
BIN="$REPO/enterprise/target/$TARGET/release/voltrus"

if [ "${1:-}" != "--no-build" ]; then
  (cd "$REPO/core/client" && npm ci && VOLTRUS_ENTERPRISE_CLIENT="$REPO/enterprise/client/src/index.ts" npm run build)
  (cd "$REPO/enterprise" && ulimit -n 65536 && cargo zigbuild --release --locked --target "$TARGET" --bin voltrus)
fi
[ -x "$BIN" ] || { echo "missing $BIN (build first)"; exit 1; }

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
cp "$BIN" "$SIM/plant-sim.py" "$SIM/seed.py" "$DEPLOY/config.yaml" \
   "$DEPLOY/voltrus-scada.service" "$DEPLOY/voltrus-scada-plant.service" "$STAGE/"
tar -czf "$STAGE.tgz" -C "$STAGE" .
scp -q "$STAGE.tgz" "$HOST:/tmp/voltrus-scada.tgz"
rm -f "$STAGE.tgz"

ssh "$HOST" 'bash -s' <<'REMOTE'
set -euo pipefail
D=/opt/voltrus-scada
id voltrus-scada >/dev/null 2>&1 || sudo /usr/sbin/useradd --system --home-dir "$D" --shell /usr/sbin/nologin voltrus-scada
sudo mkdir -p "$D/data"
rm -rf /tmp/voltrus-scada && mkdir /tmp/voltrus-scada && tar -xzf /tmp/voltrus-scada.tgz -C /tmp/voltrus-scada
cd /tmp/voltrus-scada
sudo install -m 755 voltrus "$D/voltrus.new"
sudo install -m 644 plant-sim.py seed.py "$D/"
[ -f "$D/config.yaml" ] || sudo install -m 640 config.yaml "$D/config.yaml"
sudo install -m 644 voltrus-scada.service voltrus-scada-plant.service /etc/systemd/system/
sudo chown -R voltrus-scada:voltrus-scada "$D"
sudo systemctl daemon-reload
sudo systemctl stop voltrus-scada 2>/dev/null || true
sudo mv "$D/voltrus.new" "$D/voltrus"
# The web UI is inside the binary; a static/ folder from an earlier deploy would override it.
sudo rm -rf "$D/static"
sudo systemctl enable --now voltrus-scada-plant >/dev/null
sudo systemctl restart voltrus-scada-plant
sudo systemctl enable --now voltrus-scada >/dev/null
sudo systemctl restart voltrus-scada
for i in $(seq 1 30); do
  curl -sf http://127.0.0.1:3310/api/v1/health >/dev/null && { echo "voltrus-scada healthy"; break; }
  sleep 1
  [ "$i" = 30 ] && { sudo journalctl -u voltrus-scada -n 30 --no-pager; exit 1; }
done
rm -rf /tmp/voltrus-scada /tmp/voltrus-scada.tgz
REMOTE
