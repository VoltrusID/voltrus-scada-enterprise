# Enterprise operations features

Setup guides for the operations and compliance features of the add-on. Each
one lists what works, the exact screen, API or `config.yaml` key for every
step, how to check it, troubleshooting, and what is not available yet.
Audited and tested end to end on 2026-11-16.

| Feature | Plan | Guide |
| --- | --- | --- |
| 21 CFR Part 11: electronic signatures, audit trail, CSV and PDF export | Enterprise | [part-11.md](part-11.md) |
| IEC 62443 security audit mode | Enterprise | [iec-62443.md](iec-62443.md) |
| Hot standby with automatic failover (one licence covers both nodes) | Enterprise | [hot-standby.md](hot-standby.md) |
| Multi-site hub | Integrator, Enterprise | [multi-site.md](multi-site.md) |
| RTSP video cameras | Enterprise | [video.md](video.md) |
| Scheduled PDF reports by email | Integrator, Enterprise | [reports.md](reports.md) |

Where things are in the web client: the **More actions (⋯)** button in the
header (on phones, **Open menu**) opens **Data Manager**, **System** (SMTP)
and **Audit Log**. Data Manager's sidebar has the groups Data (Hub, Cameras),
Insights (Reports, E-Signatures) and Admin (Security, Compliance, Branding, License).

The native console (`core/native`) has no screens for these features except
the audit log (**More › Audit log**, with CSV export); hub tags appear in it
like any tag. Manage everything else in the web client.

To try them without a signed licence key, build the stub-licence test node
(`cargo build --features ha-test-node --bin voltrus-ha-test-node` in
`enterprise/`); it reports a valid Enterprise licence and is never shipped.
