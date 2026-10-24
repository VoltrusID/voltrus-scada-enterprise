# Voltrus SCADA Enterprise

The paid add-on for [Voltrus SCADA](https://github.com/VoltrusID/voltrus-scada),
and the repository that builds the official Voltrus binaries.

Voltrus is open core. The code is split in two parts with two licences:

| Part | Where | Licence | Production use |
|------|-------|---------|----------------|
| **Core** | [`core/`](core), a git submodule of [VoltrusID/voltrus-scada](https://github.com/VoltrusID/voltrus-scada) | [AGPL-3.0-or-later](core/LICENSE) | Free |
| **Enterprise** | [`enterprise/`](enterprise) | [Voltrus Enterprise License](enterprise/LICENSE) (source available) | Needs a paid subscription and a licence key |

Core is the SCADA itself: protocols, dashboards, alarms, historian and the REST
API. Enterprise adds single sign-on, 21 CFR Part 11 controls, IEC 62443 audit,
hot standby, multi-site hub, RTSP video, white-label, scheduled reports and
priority support. The official binary is Core plus the Enterprise crate. Without
a valid licence key it runs as Community and the Core features keep working.

## What "source available" means here

You can read all the Enterprise code, build it, and run it to evaluate it or to
develop against it. Running it in production needs a paid licence key. You may
not remove or bypass the licence check, forge keys, or redistribute the
Enterprise code. The full terms are in [`enterprise/LICENSE`](enterprise/LICENSE).

The build always enforces licence keys: activations are Ed25519 signed, checked
offline against the public keys compiled into the binary, and bound to the
machine fingerprint. See [`enterprise/src/licensing/`](enterprise/src/licensing).

## Pricing and licence keys

Plans and prices: <https://voltrus.id/voltrus-scada>. Questions or a key
request: support@voltrus.id.

## Layout

```
core/         submodule: VoltrusID/voltrus-scada (AGPL-3.0-or-later)
enterprise/   Rust crate voltrus-enterprise, depends on ../core
  client/     Enterprise web client module
  sdks/       SDK extensions (TypeScript, Python) for the Enterprise endpoints
worker/       licence activation worker (Cloudflare Worker)
scripts/      official build and local release scripts
docs/         build notes, plans and specs
```

Build notes: [`docs/official-build.md`](docs/official-build.md).

```sh
git clone --recurse-submodules https://github.com/VoltrusID/voltrus-scada-enterprise
cd voltrus-scada-enterprise
scripts/build-official.sh
```

## Trademarks

"Voltrus" and the Voltrus logo are trademarks of JoyoDigitama. Neither licence
grants any trademark rights. Read the [trademark policy](core/TRADEMARKS.md)
before you name a fork, a product or a domain.

## Contributing and DCO

Contributions are welcome. Every commit must carry a `Signed-off-by` line
(`git commit -s`), which confirms the
[Developer Certificate of Origin](https://developercertificate.org). Because the
project is open core, contributions also grant JoyoDigitama a licence to use them
in the Enterprise add-on. Section 6 of [`enterprise/LICENSE`](enterprise/LICENSE)
and [`core/CONTRIBUTING.md`](core/CONTRIBUTING.md) have the details.

## Security

Report vulnerabilities privately to support@voltrus.id. See
[`core/SECURITY.md`](core/SECURITY.md).
