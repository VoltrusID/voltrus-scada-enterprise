# Enterprise operations features

Setup guides for the operations and compliance features of the add-on. Each
one lists what works, the exact screen, API or `config.yaml` key for every
step, how to check it, troubleshooting, and what is not available yet.
Audited and tested end to end on 2026-11-16.

| Feature | Plan | Guide |
| --- | --- | --- |
| 21 CFR Part 11: password policy and expiry, auto-logout, signatures on setpoints, alarms, recipes and mode switches, audit trail, CSV and PDF export | Enterprise | [part-11.md](part-11.md) |
| IEC 62443 security audit mode | Enterprise | [iec-62443.md](iec-62443.md) |
| Hot standby with automatic failover (one licence covers both nodes) | Enterprise | [hot-standby.md](hot-standby.md) |
| Multi-site hub | Integrator, Enterprise | [multi-site.md](multi-site.md) |
| RTSP video cameras, live camera tiles on dashboards and SCADA screens | Enterprise | [video.md](video.md) |
| Scheduled PDF reports by email | Integrator, Enterprise | [reports.md](reports.md) |
| Voltrus Energy: PLN bill shadow (WBP premium, excess kVArh), power factor and peak alarms | Integrator, Enterprise | [energy.md](energy.md) |

Where things are in the web client: the **More actions (⋯)** button in the
header (on phones, **Open menu**) opens **Data Manager**, **System** (SMTP)
and **Audit Log**. Data Manager's sidebar has the groups Data (Hub, Cameras),
Insights (PLN Bill, Reports, E-Signatures) and Admin (Security, Compliance, Branding, License).

The native console (`core/native`) has no screens for these features except
the audit log (**More › Audit log**, with CSV export); hub tags appear in it
like any tag, and the Part 11 rules (password policy, session limits, signing
prompts) apply to it because the server enforces them. Manage everything else in
the web client.

Part 11 in one line: an admin saves **Admin › Compliance › Policy** once; from
then on the server enforces it (until then the core defaults apply). Operators
meet it as the "Electronic signature required" dialog on the actions the admin
chose; API keys, MCP and flows are refused for those actions unless named as an
exception on the same tab. Checklist: [part-11.md](part-11.md#check-it-works).

To try them without a signed licence key, build the stub-licence test node
(`cargo build --features ha-test-node --bin voltrus-ha-test-node` in
`enterprise/`); it reports a valid Enterprise licence and is never shipped.
`cargo test --features ha-test-node --test compliance_hooks` runs the Part 11
checks against such a node.
