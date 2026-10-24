# AGENTS.md: Voltrus SCADA Enterprise

This repository is public and source available. Core is a git submodule of the
public AGPL repository `VoltrusID/voltrus-scada`. Enterprise (`enterprise/`) is
under the Voltrus Enterprise License. Read `docs/official-build.md` for the
layout and build.

## Never commit

- Licence signing private keys, Worker or CI secrets, `.env` files, API tokens.
- Databases (`*.db`, `*.db-wal`, `*.db-shm`), backup archives, build output
  (`target/`, `dist/`, `node_modules/`), customer data or customer licence keys.

`.gitignore` blocks the common ones. CI runs a secret scan on every push.

## Where changes go

- Core feature or fix: a pull request on `VoltrusID/voltrus-scada`, then bump the
  `core` submodule here.
- Enterprise feature: `enterprise/` and `enterprise/client/`.
- Commits need a `Signed-off-by` line (DCO).
