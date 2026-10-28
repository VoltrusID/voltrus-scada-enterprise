# Hot standby

Two Voltrus nodes, one active. The active node (primary) polls the devices,
writes to them, stores history and evaluates alarms. The other (standby) keeps a
copy of everything and takes over when the primary is gone. Licence: one
Enterprise licence covers both nodes. Code: `enterprise/src/ha/`.

## Configuration

Both nodes use the same file apart from `node_id`, `bind_address`, `peer_address`
and `role`. `auth_secret` must be identical too, or users have to sign in again
after a failover. `config.yaml` itself (ports, MQTT broker, networks, retention
settings) is not replicated: keep it the same by hand.

```yaml
ha:
  enabled: true
  role: primary            # primary | standby | auto. Only used on the very first start
  node_id: node-a          # unique per node
  bind_address: 0.0.0.0:7901
  peer_address: 10.0.0.2:7901
  heartbeat_interval_ms: 1000
  failover_after_missed: 3 # failover after heartbeat * missed of silence (3 s)
  sync_interval_ms: 200    # how often changes are shipped
  shared_secret: "<openssl rand -hex 32>"   # or env VOLTRUS_HA_SECRET, 32+ characters
  standby_polling: false   # true: the standby reads devices (live values), never writes or stores
  tls: { enabled: false, cert_path: "", key_path: "" }   # optional certificate for the link
  witness:                 # optional, see below
    kind: arbiter          # arbiter | tcp | http
    address: 10.0.0.3:7902
    timeout_ms: 1500
```

After the first start the role is recorded in the database (`ha_node`) and the
pair decides it. `role` is not consulted again. `auto` probes the peer on the
first start: it follows a primary, otherwise it takes the active role (lower
`node_id` wins if both start at once).

## What replicates

Everything in the database except the machine-bound licence activation and the
HA bookkeeping tables: devices, tags, alarms and their history, screens, flows,
users and settings, audit and security logs, telemetry and the tables the
historian adds later. Tables created by a later migration are picked up
automatically (a check every 60 s installs the capture triggers).

Mechanism: logical row replication, not WAL or page shipping. SQLite triggers on
every table append `(table, key)` to `ha_changelog` in the same transaction as
the write. The shipper reads the log, fetches the current row of each key and
sends full row images (or deletes) in one read snapshot; the standby applies them
with `INSERT OR REPLACE` in one transaction together with its new position.
Replay is idempotent and repeated changes to one row collapse into one image.
This was chosen over WAL shipping because SQLite offers no way to apply WAL
frames under a live connection, and over hooking each write function because
the trigger approach covers every table, including the packed-block table
and the raw-row deletes of a compaction, as ordinary row changes. Bulk deletes
travel as key ranges.

On connect the standby takes a full sync (a consistent snapshot read on a
second connection, streamed in chunks, then incremental from the snapshot point)
unless it is in the same epoch and inside the retained log. The standby is
read-only: every HTTP write except sign-in, `/api/v1/ha/*` and the licence
endpoints answers `503 standby_read_only`, and its history, alarm, retention,
egress, OEE, command queue and flow jobs are idle.

### Data loss on failover

Changes are shipped every `sync_interval_ms` (200 ms) and acknowledged. A crash
of the primary loses what was committed after the last acknowledged batch:
at most `sync_interval_ms` plus one network round trip and one apply, so well
under a second on a LAN, and only if the standby was connected and caught up.
Samples are written to the database every `persist_interval_secs`, so values
that were polled but not yet persisted are lost in any failover. The history
also has a gap of about the failover time, because neither node polls then.
A controlled switchover loses nothing. If a standby was disconnected longer
than the log holds (4 million changes) it takes a full sync instead.

## Failover safety

* **Epoch.** Every new primary starts a new epoch, persisted before the node acts.
  A node that sees a higher epoch stops acting at once. Two primaries in the same
  epoch are ordered by `node_id` (the lower keeps the role).
* **Fencing.** The loser stops polling and writing, then takes a full sync from the
  winner before it follows (its data may hold changes the winner never received).
* **Boot.** A node that was primary asks its peer and its witness before it acts, so
  a restarted old primary never comes up beside the primary that replaced it.
* **Device writes** are refused at the lowest level (Modbus, S7, EtherNet/IP,
  BACnet, Voltrus PLC), whatever path asked, whenever the node is not active.
* **Witness.** Without one, a standby takes over whenever the primary is silent: a
  broken link between two healthy nodes then produces two primaries until the link
  heals and the epochs sort it out. With `kind: arbiter` (a process
  `voltrus --ha-witness` on a third machine) the active node holds a lease it
  renews with every heartbeat and stops acting if it cannot; the standby may take
  over only by acquiring the lease, which the arbiter grants after the holder's
  lease ran out. That prevents two active nodes under any partition. `tcp` and
  `http` check that a host that must be reachable from the real network (the
  gateway, a PLC) answers: the standby promotes only if it does, an active node that
  lost its peer and the witness stops. They protect against an isolated node but
  cannot tell a dead peer from a broken link.
* **Link.** TLS 1.3 (rustls, no C library). Identity comes from the shared secret:
  both ends prove it with an HMAC bound to the TLS exporter secret of the session,
  so a relay that terminates TLS fails and a recorded handshake is useless. The
  secret must be long and random. A configured certificate is optional; by default
  each node generates one at start.

## Operations

* `GET /api/v1/ha/status`: role, epoch, peer, sync state, lag, pending changes,
  last failover and its reason, witness state.
* `POST /api/v1/ha/force-sync` (admin): full resync now. 200 with the report, 202
  while it runs after `wait_secs`.
* `POST /api/v1/ha/force-failover` (admin): on the primary a controlled switchover
  (stop, ship the rest, the standby takes over, the old primary follows without a
  full sync); on a linked standby it asks the primary to do that; on an unlinked
  standby it takes over (`?force=true` skips the data and reachability checks).
* Upgrade the standby first, then the primary. The link refuses a different
  protocol version, and a standby that cannot apply a change reports `error`.

Roles: `primary` (acts), `standby` (follows), `fenced` (wants to be primary but
may not act: lost its lease, or both its peer and witness; resumes by itself).
