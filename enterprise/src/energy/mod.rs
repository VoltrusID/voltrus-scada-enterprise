//! Voltrus Energy: PLN electricity bill shadowing (spec `product-specs/voltrus-energy.md`, 3.1 and 3.2).
//!
//! * [`tariff`]: sites, PLN tariffs (data, with the 2025 rates as defaults), WBP windows, local time.
//! * [`counter`]: hourly kWh/kVArh from cumulative counters (rollover, resets, gaps).
//! * [`job`]: the hourly job writing `energy_hour`, the PF / WBP-load alarms and the daily summary.
//! * [`bill`], [`report`]: the shadow bill, month-end projection, daily breakdown, reconciliation.
//! * [`handlers`]: REST API under `/api/v1/energy`.
//!
//! Licensing: the core has no `energy` flag yet, so the feature follows the
//! Integrator tier (spec §6 option A requires Integrator).

pub mod bill;
pub mod counter;
pub mod handlers;
pub mod job;
pub mod report;
pub mod store;
pub mod tariff;

#[cfg(test)]
mod tests;

use voltrus::license::LicenseTier;

pub(crate) use store::ensure_schema;

/// Whether the licence covers Voltrus Energy: Integrator or Enterprise.
pub fn licensed(tier: LicenseTier) -> bool {
    tier != LicenseTier::Community
}
