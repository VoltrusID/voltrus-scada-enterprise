//! Voltrus Enterprise add-on.
//!
//! Plugs into the open core through [`voltrus::edition::Extension`]: licensing,
//! SSO, 21 CFR Part 11, IEC 62443 audit mode, hot standby, multi-site hub, RTSP
//! video, white-label branding, scheduled PDF reports and Telegram/WhatsApp alarms.
//!
//! The official `voltrus` binary is this crate's `main`, which passes
//! [`Enterprise`] to `voltrus::run`.

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

pub mod ai;
pub mod audit;
pub mod branding;
pub mod chat;
pub mod compliance;
pub mod ha;
pub mod hub;
pub mod licensing;
pub mod reports;
mod routes;
mod runtime;
pub mod sso;
pub mod video;

pub use runtime::{Runtime, rt};

use voltrus::api::AppState;
use voltrus::edition::{BoxFuture, Extension, ExtensionRoutes, ExternalIdentity, StartContext};
use voltrus::license::LicenseStatus;
use voltrus::storage::Storage;

/// The Enterprise add-on, installed with `voltrus::run(Arc::new(Enterprise::default()))`.
#[derive(Default)]
pub struct Enterprise;

impl Extension for Enterprise {
    fn name(&self) -> &'static str {
        "enterprise"
    }

    fn resolve_license<'a>(&'a self, storage: &'a Storage) -> BoxFuture<'a, LicenseStatus> {
        Box::pin(licensing::resolve_license_status(storage))
    }

    fn resolve_node_role<'a>(
        &'a self,
        config: &'a voltrus::models::AppConfig,
        storage: &'a Storage,
        license: &'a LicenseStatus,
    ) -> BoxFuture<'a, anyhow::Result<voltrus::node_role::Start>> {
        Box::pin(ha::resolve_node_role(config, storage, license))
    }

    fn cli_help(&self) -> &'static str {
        static HELP: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
            format!("{}{}", licensing::cli::HELP, ha::witness::HELP)
        });
        HELP.as_str()
    }

    fn handle_cli<'a>(&'a self, args: &'a [String]) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            ha::witness::handle_cli(args).await || licensing::cli::handle(args).await
        })
    }

    fn start<'a>(&'a self, ctx: StartContext) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            runtime::start(ctx).await;
            Ok(())
        })
    }

    fn routes(&self, _state: &AppState) -> ExtensionRoutes {
        routes::routes()
    }

    fn authenticate_bearer<'a>(
        &'a self,
        token: &'a str,
    ) -> BoxFuture<'a, Option<ExternalIdentity>> {
        Box::pin(sso::authenticate_bearer(token))
    }

    fn auth_providers(&self) -> Vec<serde_json::Value> {
        sso::auth_providers()
    }
}
