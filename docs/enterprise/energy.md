# Voltrus Energy: PLN bill shadow

Computes the PLN electricity bill of an Indonesian plant (tariff I-3/TM or
I-4/TT) every hour from its own meter, so the WBP peak premium and the
excess-kVArh charge are visible in Rupiah before PLN's bill arrives. Alarms
warn on a falling power factor and on peak-hour load. Licence: Integrator or
Enterprise (there is no separate `energy` flag yet). Code:
`enterprise/src/energy/`, web client `enterprise/client/src/panels/Energy*.tsx`.
Spec: `product-specs/voltrus-energy.md` §3.1 and §3.2 (MVP).

**The bill is an estimate until checked against a real PLN bill.** The
excess-kVArh rule, the WBP hours and K vary by contract and region. The page
shows "to be confirmed with your PLN bill" until one month's total agrees with
PLN's within 2% (see [Reconcile](#reconcile-with-plns-bill)).

## What it computes

Per meter and hour (`energy_hour`), from cumulative counters:

* `ΔkWh = E(end of hour) − E(start of hour)` × multiplier, the same for kVArh.
* Band: WBP when the hour's local start time is inside a WBP window (default
  18:00-22:00), else LWBP.
* Quality: `ok`, `estimated` (a boundary was interpolated across a gap longer
  than the meter's max gap, default 15 min, or the counter reset in the hour)
  or `gap` (no sample after the hour yet; the hour shows only what its own
  samples show and is computed again when data arrives).

Per site and billing month (calendar month in the site's local time), from the
meters with role **PLN incomer** (sub-meters are computed but not billed):

| Line | I-3 | I-4 |
| --- | --- | --- |
| Energy LWBP | kWh_LWBP × T | kWh_LWBP × T |
| Energy WBP | kWh_WBP × K × T | kWh_WBP × T_WBP (default = T, no K) |
| WBP premium (inside the WBP line) | kWh_WBP × (K − 1) × T | kWh_WBP × (T_WBP − T), 0 by default |
| Excess kVArh (monthly) | max(0, kVArh − f × kWh) × T_kVArh | same |
| cos φ | kWh / √(kWh² + kVArh²) | same |

`f` is `tan(acos(pf_limit))` = 0.6197 at the default PLN limit 0.85, unless the
tariff sets `kvarh_factor`. **Not included: demand charge (biaya beban), PPJ
and PPN**, as in `energy-research/pln-bill-worksheet.xlsx`.

Check against the worksheet's illustration (600,000 kWh, 20% WBP, K 1.4,
cos φ 0.80, kVArh 450,000), in the unit tests (`energy::bill::tests`):

| | factor 0.62 (spec text) | factor tan(acos 0.85) (worksheet cells, default) |
| --- | --- | --- |
| Excess kVArh | 78,000 | 78,153.40 |
| kVArh charge | Rp 86,949,720 | Rp 87,120,717.73 |
| WBP premium | Rp 49,717,440 | Rp 49,717,440 |
| Total (LWBP + WBP + kVArh) | | Rp 758,306,157.73 |

The spec and research round the factor to 0.62; the worksheet uses the exact
value. Set `kvarh_factor: 0.62` on the tariff if a customer's bill uses the
rounded one.

**Projected month-end**: per band, kWh so far ÷ hours with data × hours of
that band in the month; kVArh the same over all hours. Hours without data
(meter added mid-month, outages) are treated as unknown and projected.

**Daily breakdown**: each local day's kWh, kVArh, cos φ, WBP premium and its
own excess-kVArh charge. PLN bills the excess per month, so the days do not add
up to the month's kVArh charge; they show which days cause it.

## Tariff defaults (data, not code)

PLN 2025 rates from the research (§1.2), offered when a site is created and
editable on the site's Setup tab:

| | LWBP | WBP | Excess kVArh |
| --- | --- | --- | --- |
| I-3/TM (> 200 kVA) | Rp 1,035.78/kWh | K × LWBP, K 1.4 (1.4 ≤ K ≤ 2) | Rp 1,114.74 |
| I-4/TT (≥ 30,000 kVA) | Rp 996.74/kWh | Rp 996.74 | Rp 996.74 |

A tariff applies from the first day of its `valid_from` month until a later
one starts (a month before the first tariff uses the first). A rate change is
a new tariff, so old months keep their price.

## Set up

1. Licence: **Data Manager › Admin › License** shows Integrator or Enterprise.
2. The meter on PLN's supply must be a device in Voltrus with **cumulative**
   kWh (import) and, ideally, kVArh registers stored in the historian. The
   built-in PM5560 template does not map the energy counters yet (they are
   64-bit; the Modbus driver reads int16, uint16, int32, float32 and float64
   only): map them yourself as a supported type (many meters also offer
   float32 or float64 energy registers), or as registers combined by an
   expression tag (only tags whose samples are in the historian can be used).
3. **Data Manager › Insights › PLN Bill › + New site**: name, time zone (WIB,
   WITA, WIT), WBP hours, contracted kVA (reference only), the first tariff,
   and the alerts.
4. **Setup › + Add meter**: device and tags (pick from the list or type the
   historian metric name), role **PLN incomer**, multiplier = CT ratio × PT
   ratio × unit (0.001 for Wh/varh counters), rollover value if the register
   can wrap (4294967296 for unsigned 32-bit), and without a kVArh counter a
   kvar tag to integrate (trapezoid rule; less exact). Saving computes the
   hours from the start of the previous month in the background.
5. Open **Bill**. The hourly job then keeps it current (it runs at minute 2 of
   every hour on the active node, recomputing the last 24 h and any `gap` hour
   of the last 7 days). **Recompute month** redoes a whole month, for example
   after fixing the multiplier.

Time zones are fixed offsets in whole hours (Indonesia has no daylight saving):
`Asia/Jakarta`/`Asia/Pontianak`/`WIB` (+7), `Asia/Makassar`/`WITA` (+8),
`Asia/Jayapura`/`WIT` (+9), `UTC`, or `+07:00`. WBP windows are whole hours;
a window may cross midnight (`22:00-02:00`).

## Alerts

Evaluated by the hourly job from the PLN incomer meters, raised as core alarms
(alarm list, WebSocket, and the notification rules of their source key):

| Source key | Alarm | Clears |
| --- | --- | --- |
| `energy:site<id>:pf` | rolling cos φ over the last `pf_window_hours` (24) below `pf_warn` (0.87), warning | cos φ back above `pf_warn` + 0.005 |
| `energy:site<id>:wbp_kw` | the last full hour is WBP and its average load (kWh in that hour) is above `wbp_kw_target` | a WBP hour below target, or the first LWBP hour |
| `energy:site<id>:daily` | not an alarm: once a day after local midnight, "yesterday's kVArh charge Rp X, WBP premium Rp Y, cos φ" (Indonesian or English) | |

To receive them by Telegram, WhatsApp, email or webhook, add a notification
rule (**Data Manager › Notifications**) with the source key as written above.
The daily summary is sent only while such a rule exists for
`energy:site<id>:daily`, at most once per day.

The WBP load alarm works on hourly averages and is raised about two minutes
after the hour: it tells which WBP hours exceeded the target, it does not trip
inside the hour. Live gating on a kW tag needs a flow with a cron trigger (or
a schedule condition on alarms, which the core does not have).

## Reconcile with PLN's bill

When the bill arrives: **Bill**, pick the month, **Enter PLN's figures**: kWh
LWBP, kWh WBP and kVArh (required), and optionally excess kVArh, the Rupiah
lines and the total, all without demand charge, PPJ and PPN. The table then
shows Voltrus, PLN, the difference and the percentage per line. Without a
total, PLN's quantities are priced with the site's tariff. A month whose total
agrees within 2% confirms the site, and the "to be confirmed" note goes away.
If it does not agree, check K, the WBP hours, the multiplier and the kVArh rule
(try `kvarh_factor` 0.62).

## API

Every endpoint needs the licence (403 otherwise). Reads: any signed-in user.
Writes: the configuration roles (admin, demo, integration). Every write is in
the audit log (`energy_*` actions).

| Method and path | What |
| --- | --- |
| `GET /api/v1/energy/defaults` | default tariffs, factor, WBP window, alert settings |
| `GET /api/v1/energy/sites`, `POST` | list (with current tariff and confirmed flag), create (with optional first `tariff`) |
| `GET/PUT/DELETE /api/v1/energy/sites/{id}` | site with tariffs, meters, notification keys; update (re-bands stored hours if WBP or time zone changed); delete with everything |
| `POST /api/v1/energy/sites/{id}/tariffs` | add or replace the tariff of a `valid_from` month |
| `PUT/DELETE /api/v1/energy/tariffs/{id}` | change, delete (a site keeps one) |
| `POST /api/v1/energy/sites/{id}/meters`, `PUT/DELETE /api/v1/energy/meters/{id}` | meters |
| `GET /api/v1/energy/meters/{id}/hours?from=&to=` | hourly rows (UTC seconds, up to 62 days) |
| `GET /api/v1/energy/sites/{id}/bill?month=YYYY-MM` | month to date, projection, tariff, data quality, PLN figures and differences |
| `GET /api/v1/energy/sites/{id}/daily?month=YYYY-MM` | daily breakdown |
| `GET /api/v1/energy/sites/{id}/reconcile` | every reconciled month |
| `PUT/DELETE /api/v1/energy/sites/{id}/reconcile/{month}` | enter or remove PLN's figures |
| `POST /api/v1/energy/sites/{id}/recompute?month=` | recompute a month |

Example:

```sh
curl -b cookies -X POST https://scada.example/api/v1/energy/sites \
  -H 'Content-Type: application/json' \
  -d '{"name":"Pabrik A","timezone":"Asia/Jakarta","tariff":{"valid_from":"2026-11","golongan":"I-3","k_factor":1.4,"rate_lwbp":1035.78,"rate_kvarh":1114.74,"pf_limit":0.85}}'
```

## How the counter values are read

The job reads the historian's full-resolution samples
(`Storage::query_history`, raw rows and packed blocks merged) one hour at a
time, 12 h either side of the range. It does not use `telemetry_hourly`: the
core writes that tier only when raw data expires (after 90 days by default),
so it never holds the current month. Samples are made monotonic (a drop with
the previous reading above 90% of the rollover value and the new one below 10%
is a rollover; any other drop is a reset that counts as zero), each hour is
reduced to its first and last sample, and the counter at each hour boundary is
interpolated between the samples on either side. Energy is conserved across a
gap: its hours share the counter difference (and are marked `estimated`).

## Data

Tables, created at start and replicated by hot standby: `energy_site`,
`energy_tariff`, `energy_meter`, `energy_hour` (meter, UTC hour, kWh, kVArh,
band, quality), `energy_bill` (PLN's figures and the shadow bill at entry),
`energy_notice` (daily summaries sent). The job runs only on the active node.

## Not done yet / unconfirmed

* **Unconfirmed**: the excess-kVArh formula (common practice, not found in the
  tariff regulation), the 18:00-22:00 WBP window and K per contract; the
  tariff values are 2025's and change by regulation.
* No licence flag of its own: follows the Integrator tier until core gets an
  `energy` flag (and the activation worker an add-on list).
* No MCP tools: the core's MCP registry has no extension hook.
* Not modelled: demand charge (biaya beban), PPJ, PPN, minimum charges (jam
  nyala), billing periods that are not calendar months, kVA/kW maximum demand.
* No outlier filter: a corrupt counter reading that jumps up is counted.
* Areas and sub-meter roll-ups (`energy_area`), energy per unit produced
  (MES join), POME and CBAM are later phases.
* The 64-bit counter types and a field-checked PM5560 template are core work.
