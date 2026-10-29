#!/usr/bin/env bash
# Run Voltrus Enterprise SSO against real servers: Keycloak (SAML + OIDC) and a Samba
# Active Directory DC (LDAPS), over plain http and behind a local TLS proxy.
#
#   ./run.sh up      start Keycloak, Samba, Caddy and three Voltrus instances
#   ./run.sh test    HTTP-level suite against them (python3, stdlib only)
#   ./run.sh down    stop everything, delete containers, volumes and ./.run
#
# Needs docker (with compose), python3, openssl, curl and a Rust toolchain. The server is
# the `voltrus-ha-test-node` binary (the real Enterprise add-on with a stub Enterprise
# licence, built only with the `ha-test-node` feature). Set BIN to use a prebuilt one.
# Instances (all on the same Keycloak/Samba):
#   http    http://localhost:3100              cookie_secure auto, plain http
#   https   https://scada.localhost:8443       Caddy (internal CA) -> :3101, SameSite=None cookies
#   strict  http://localhost:3102              role_mapping.default_role "": unmapped users refused
#   tlsidp  http://localhost:3103              Keycloak over https (https://kc.localhost:8444, private CA):
#                                              idp_metadata_url and OIDC discovery with ca_cert_file
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../.." && pwd)
RUN="$HERE/.run"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/enterprise/target}
BIN=${BIN:-$CARGO_TARGET_DIR/debug/voltrus-ha-test-node}
KC=${KC_URL:-http://127.0.0.1:8081}
COMPOSE=(docker compose -f "$HERE/docker-compose.yml")
export VOLTRUS_OIDC_CLIENT_SECRET=${OIDC_SECRET:-oidc-live-secret}
export OIDC_SECRET=$VOLTRUS_OIDC_CLIENT_SECRET
export VOLTRUS_LDAP_BIND_PASSWORD='Svc-Pass-1!'
export LDAP_CA_FILE="$RUN/ldap-ca.pem"

stop_instances() {
  for d in http https strict tlsidp; do
    [ -f "$RUN/$d/pid" ] && kill "$(cat "$RUN/$d/pid")" 2>/dev/null || true
    rm -f "$RUN/$d/pid"
  done
  docker rm -f voltrus-sso-live-caddy >/dev/null 2>&1 || true
}

start_instance() { # name port public_url default_role  (env passes through to gen-config.py)
  local name=$1 port=$2 url=$3 role=$4
  mkdir -p "$RUN/$name"
  rm -f "$RUN/$name"/data.db*
  DEFAULT_ROLE=$role python3 "$HERE/gen-config.py" "$RUN/$name/config.yaml" "$port" "$url" data.db "$RUN/idp-metadata.xml"
  # A static/ folder next to the config serves the web UI, needed only for browser runs.
  local static=${STATIC_DIR:-$ROOT/core/static}
  [ -f "$static/index.html" ] && ln -sfn "$static" "$RUN/$name/static"
  (cd "$RUN/$name"; RUST_LOG=info nohup "$BIN" config.yaml > node.log 2>&1 < /dev/null & echo $! > pid)
  for _ in $(seq 60); do curl -fs "http://127.0.0.1:$port/api/v1/auth/providers" >/dev/null 2>&1 && return; sleep 1; done
  echo "instance $name did not start; see $RUN/$name/node.log" >&2; exit 1
}

case "${1:-}" in
  up)
    mkdir -p "$RUN"
    [ -x "$BIN" ] || (cd "$ROOT/enterprise" && VOLTRUS_SKIP_UI_EMBED=1 cargo build --features ha-test-node --bin voltrus-ha-test-node)
    "${COMPOSE[@]}" up -d keycloak samba
    python3 "$HERE/keycloak-setup.py" realm
    "$HERE/samba-setup.sh"
    curl -fs "$KC/realms/voltrus/protocol/saml/descriptor" -o "$RUN/idp-metadata.xml"
    stop_instances
    docker run -d --name voltrus-sso-live-caddy --network host -v "$HERE/Caddyfile:/etc/caddy/Caddyfile:ro" \
      caddy:2-alpine >/dev/null
    for _ in $(seq 30); do
      docker exec voltrus-sso-live-caddy cat /data/caddy/pki/authorities/local/root.crt > "$RUN/caddy-root.pem" 2>/dev/null \
        && [ -s "$RUN/caddy-root.pem" ] && break
      sleep 1
    done
    start_instance http 3100 http://localhost:3100 viewer
    start_instance https 3101 https://scada.localhost:8443 viewer
    start_instance strict 3102 http://localhost:3102 ""
    KC_ISSUER=https://kc.localhost:8444/realms/voltrus \
      IDP_METADATA_URL=https://kc.localhost:8444/realms/voltrus/protocol/saml/descriptor \
      IDP_CA_FILE="$RUN/caddy-root.pem" start_instance tlsidp 3103 http://localhost:3103 viewer
    for u in http://localhost:3100 https://scada.localhost:8443 http://localhost:3102 http://localhost:3103; do
      python3 "$HERE/keycloak-setup.py" saml "$u"
    done
    echo "up. admin passwords are in $RUN/*/node.log; browse http://localhost:3100 or https://scada.localhost:8443 (accept the Caddy CA)"
    ;;
  test)
    shift
    RUN_DIR="$RUN" python3 "$HERE/test_live.py" "$@"
    ;;
  down)
    stop_instances
    "${COMPOSE[@]}" down -v --rmi all 2>&1 | tail -3 || true
    docker rmi caddy:2-alpine >/dev/null 2>&1 || true
    rm -rf "$RUN"
    ;;
  *)
    sed -n '2,/^set -euo/p' "$0" | sed 's/^# \{0,1\}//;/^set -euo/d'
    exit 2
    ;;
esac
