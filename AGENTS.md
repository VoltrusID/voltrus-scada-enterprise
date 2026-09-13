# AGENTS.md — Voltrus Server

## Public / Private Boundary (read before pushing anything anywhere)

Exactly **one** repo in the Voltrus family is PUBLIC. Everything else is proprietary.

| Location | Visibility | Contents |
|----------|-----------|----------|
| `github.com/VoltrusID/voltrus-gateway` | **PUBLIC** | SDKs only: `sdk/typescript/`, `sdk/python/` |
| this repo (`voltrus/`) | private | Rust SCADA server + frontend. Never publish any of it. |
| `../MacTools/` | private | Desktop apps. Its `gateway/sdks/` is a **different product's SDK** (v0.1.x) — NOT the public SDK. |
| `../voltrus-remote/` | private | Control plane / headscale. |

Local clone of the public repo: `~/Documents/JoyoDigitama/voltrus-sdk` — **outside** this workspace.

## SDK Source of Truth

- `sdks/` in this repo is canonical. It is generated from the server's OpenAPI spec by `sdks/generate.sh` (needs docker; `VOLTRUS_BIN` runs the local server binary to dump the spec).
- The public repo is downstream: after regenerating, sync `sdks/` → `~/Documents/JoyoDigitama/voltrus-sdk/sdk/`. Edits made directly in the public clone get lost on the next sync.

## Publish Flow (sync to public repo)

```bash
SRC=$HOME/Documents/JoyoDigitama/voltrus/voltrus   # note: repo is nested inside the workspace dir of the same name
DST=$HOME/Documents/JoyoDigitama/voltrus-sdk
rm -rf "$DST/sdk/typescript" "$DST/sdk/python"
cp -R "$SRC/sdks/typescript" "$DST/sdk/typescript"
cp -R "$SRC/sdks/python" "$DST/sdk/python"
# strip build artifacts
rm -rf "$DST/sdk/typescript/node_modules" "$DST/sdk/typescript/dist"
       find "$DST" -name '__pycache__' -type d -exec rm -rf {} +
find "$DST" -name '*.pyc' -delete
find "$DST" -name '*.egg-info' -type d -exec rm -rf {} +
```

Then, in `$DST`:

1. **Leak scan must return nothing:**
   `grep -rnE '(JoyoDigitama|MacTools|mactools|dzulfikar|headscale|control-api|cargo|crate|ghp_|gho_|sk-|BEGIN [A-Z ]*KEY)' . --exclude-dir=.git`
2. **Manifests**: `sdk/typescript/package.json` (`repository` block) and `sdk/python/pyproject.toml` (`Repository` URL) must point at `https://github.com/VoltrusID/voltrus-gateway`. Keep both `version` fields in lockstep (e.g. both `0.47.0`).
3. Commit, `git push` (branch `main`).

Note: `generate.sh` regenerates `package.json`, `.gitignore`, and the generated README content — after regenerating, re-apply the `repository` block and verify the hand-written parts of the READMEs/examples survived.

## NEVER put in the public repo

- Anything from `src/`, `client/`, `config.yaml`, `scripts/`, `deploy*`, `scenarios/` — any server code or config
- `openapi.json` — dumped from the internal binary; the SDK code already encodes the public API surface
- Git history from this repo — the public repo keeps its own fresh history; old internal commits can leak through copied history
- Anything from `MacTools/gateway/sdks/` — different product

## Fine to keep

- Generic doc examples: `http://192.168.1.100:3000`, `192.168.1.0/24`, `vt_...` placeholder keys
- Product name "Voltrus", `voltrus.com` links, MIT LICENSE ("Copyright Voltrus")

## Accounts

- GitHub org **`VoltrusID`** — public product org (created 2026-09-13). The public SDK repo lives here.
- `Opex-Technologies-Pte-Ltd` (org) and `dzulfikar08` (personal) — company/internal. Do not attach the public SDK repo to them.
- The repo is named `voltrus-gateway` by owner decision but currently contains **only** `sdk/`. It is public from the instant anything is pushed — never push internal files there "temporarily".
