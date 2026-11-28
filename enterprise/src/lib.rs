//! Voltrus Enterprise add-on.
//!
//! Plugs into the open core through [`voltrus::edition::Extension`]: licensing,
//! SSO, 21 CFR Part 11, IEC 62443 audit mode, hot standby, multi-site hub, RTSP
//! video, white-label branding, scheduled PDF reports, Telegram/WhatsApp alarms and
//! the push relay connection for the phone app, and Voltrus Energy (PLN bill shadowing).
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
pub mod energy;
pub mod ha;
pub mod hub;
pub mod licensing;
pub mod mobile;
pub mod reports;
mod routes;
mod runtime;
pub mod sso;
pub mod video;

pub use runtime::{Runtime, rt, try_rt};

use voltrus::api::AppState;
use voltrus::edition::{
    ActionContext, BoxFuture, Extension, ExtensionRoutes, ExternalIdentity, PasswordChangeKind,
    PasswordCheck, SessionPolicy, SignatureRecord, SignatureRequirement, StartContext,
};
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
        static HELP: std::sync::LazyLock<String> =
            std::sync::LazyLock::new(|| format!("{}{}", licensing::cli::HELP, ha::witness::HELP));
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

    fn routes(&self, state: &AppState) -> ExtensionRoutes {
        routes::routes(state)
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

    // 21 CFR Part 11: answered from the Compliance settings (compliance::hooks).

    fn validate_new_password<'a>(
        &'a self,
        check: &'a PasswordCheck,
    ) -> BoxFuture<'a, Result<(), String>> {
        Box::pin(compliance::hooks::validate_new_password(check))
    }

    fn password_changed<'a>(
        &'a self,
        username: &'a str,
        password_hash: &'a str,
        kind: PasswordChangeKind,
    ) -> BoxFuture<'a, ()> {
        Box::pin(compliance::hooks::password_changed(
            username,
            password_hash,
            kind,
        ))
    }

    fn password_expired<'a>(&'a self, user: &'a voltrus::models::User) -> BoxFuture<'a, bool> {
        Box::pin(compliance::hooks::password_expired(user))
    }

    fn session_policy<'a>(
        &'a self,
        _username: &'a str,
        _role: &'a str,
    ) -> BoxFuture<'a, SessionPolicy> {
        Box::pin(compliance::hooks::session_policy())
    }

    fn signature_required<'a>(
        &'a self,
        ctx: &'a ActionContext,
    ) -> BoxFuture<'a, Option<SignatureRequirement>> {
        Box::pin(compliance::hooks::signature_required(ctx))
    }

    fn record_signature<'a>(
        &'a self,
        record: &'a SignatureRecord,
    ) -> BoxFuture<'a, anyhow::Result<Option<String>>> {
        Box::pin(compliance::hooks::record_signature(record))
    }
}
