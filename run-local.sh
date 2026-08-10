#!/usr/bin/env bash
# Run voltrus (solar-farm scenario) + gateway locally, in tmux.
#
#   ./run-local.sh            start everything
#   ./run-local.sh stop       kill the tmux session
#
# Defaults avoid clashes with other local services (excon-api :3000,
# nats :1883): voltrus → :3300, mqtt-sim → :1884. Override via env:
#   VOLTRUS_PORT=4000 MQTT_PORT=1883 ./run-local.sh
set -euo pipefail

SCENARIO="${1:-solar-farm}"
if [ "$SCENARIO" = "stop" ]; then
  tmux kill-session -t voltrus-local 2>/dev/null && echo "stopped" || echo "not running"
  exit 0
fi

# Paths
VOLTRUS_DIR="$(cd "$(dirname "$0")" && pwd)"
GATEWAY_DIR="${GATEWAY_DIR:-/Users/macbookpro/Documents/JoyoDigitama/MacTools/gateway}"
SCENARIO_DIR="$VOLTRUS_DIR/scenarios/$SCENARIO"
[ -d "$SCENARIO_DIR" ] || { echo "unknown scenario: $SCENARIO"; exit 1; }

# Ports
VOLTRUS_PORT="${VOLTRUS_PORT:-3300}"
MQTT_PORT="${MQTT_PORT:-1884}"

SESSION="voltrus-local"
BIN="$VOLTRUS_DIR/target/release"
GW_BIN="$GATEWAY_DIR/target/debug/voltrus-gateway"

echo "Building voltrus (release)…"
( cd "$VOLTRUS_DIR"   && cargo build --release --features gateway-modbus,gateway-opcua >/dev/null )
echo "Building gateway (debug)…"
( cd "$GATEWAY_DIR"   && cargo build >/dev/null )

# Per-scenario config with overridden server_port (yaml has 3000; we use 3300).
WORKDIR="$(mktemp -d -t voltrus-local.XXXXXX)"
sed "s/^server_port:.*/server_port: ${VOLTRUS_PORT}/" "$SCENARIO_DIR/voltrus.yaml" > "$WORKDIR/config.yaml"
# ServeDir resolves "static" relative to cwd → symlink the repo build into the workdir.
ln -s "$VOLTRUS_DIR/static" "$WORKDIR/static"

# Fresh session
tmux kill-session -t "$SESSION" 2>/dev/null || true
tmux new-session -d -s "$SESSION" -n info "echo 'voltrus-local — solar farm'; bash"

launch() {  # <window-name> <command...>
  local name="$1"; shift
  tmux new-window -t "$SESSION" -n "$name"
  tmux send-keys -t "$SESSION:$name" "$* 2>&1 | sed 's/^/[$name] /'" C-m
}

launch modbus-inv  "$BIN/modbus-sim --scenario solar-farm --role inverter   --port 5020"
launch modbus-meter "$BIN/modbus-sim --scenario solar-farm --role grid-meter --port 5021"
launch s7-weather  "$BIN/s7-sim     --scenario solar-farm --port 10200"
launch mqtt-weather "$BIN/mqtt-sim  --scenario solar-farm --port ${MQTT_PORT}"

sleep 2
launch voltrus     "cd '$WORKDIR' && '$BIN/voltrus' config.yaml"
launch gateway     "'$GW_BIN' --bind 127.0.0.1:50061"

tmux select-window -t "$SESSION:voltrus"

cat <<EOF

  ✓ running in tmux session: $SESSION

  Dashboard : http://localhost:${VOLTRUS_PORT}
  Gateway   : gRPC 127.0.0.1:50061   (grpcui/grpcurl)
  Workdir   : $WORKDIR

  tmux attach -t $SESSION        # Ctrl+b d to detach
  tmux kill-session -t $SESSION  # stop all
  $0 stop                       # same thing

EOF
