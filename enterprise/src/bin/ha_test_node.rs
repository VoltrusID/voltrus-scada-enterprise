//! A node for the hot standby integration test (`tests/ha_pair.rs`).
//!
//! It is the real add-on with one difference: the licence resolves to a valid
//! Enterprise key without a signature, so a pair can run on a developer machine
//! or in CI without a signed activation. Built only with the `ha-test-node`
//! feature, which no release script enables.

use std::sync::Arc;

use voltrus::api::AppState;
use voltrus::edition::{BoxFuture, Extension, ExtensionRoutes, ExternalIdentity, StartContext};
use voltrus::license::LicenseStatus;
use voltrus::models::AppConfig;
use voltrus::storage::Storage;
use voltrus_enterprise::Enterprise;

struct TestNode(Enterprise);

impl Extension for TestNode {
    fn name(&self) -> &'static str {
        "enterprise (ha test node)"
    }

    fn resolve_license<'a>(&'a self, _storage: &'a Storage) -> BoxFuture<'a, LicenseStatus> {
        Box::pin(async {
            LicenseStatus::Valid {
                license_key: "ha-test-node".into(),
                expires_at: None,
                days_remaining: None,
                tier: Some("enterprise".into()),
                grace: false,
                legacy_white_label: false,
                site: None,
                support_until: None,
            }
        })
    }

    fn cli_help(&self) -> &'static str {
        self.0.cli_help()
    }

    fn handle_cli<'a>(&'a self, args: &'a [String]) -> BoxFuture<'a, bool> {
        self.0.handle_cli(args)
    }

    fn resolve_node_role<'a>(
        &'a self,
        config: &'a AppConfig,
        storage: &'a Storage,
        license: &'a LicenseStatus,
    ) -> BoxFuture<'a, anyhow::Result<voltrus::node_role::Start>> {
        self.0.resolve_node_role(config, storage, license)
    }

    fn start<'a>(&'a self, ctx: StartContext) -> BoxFuture<'a, anyhow::Result<()>> {
        self.0.start(ctx)
    }

    fn routes(&self, state: &AppState) -> ExtensionRoutes {
        self.0.routes(state)
    }

    fn authenticate_bearer<'a>(
        &'a self,
        token: &'a str,
    ) -> BoxFuture<'a, Option<ExternalIdentity>> {
        self.0.authenticate_bearer(token)
    }

    fn auth_providers(&self) -> Vec<serde_json::Value> {
        self.0.auth_providers()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    voltrus::run(Arc::new(TestNode(Enterprise))).await
}
