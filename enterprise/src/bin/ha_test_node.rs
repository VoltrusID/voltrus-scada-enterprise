//! A node for the hot standby integration test (`tests/ha_pair.rs`).
//!
//! It is the real add-on with one difference: the licence resolves to a valid
//! Enterprise key without a signature, so a pair can run on a developer machine
//! or in CI without a signed activation. Built only with the `ha-test-node`
//! feature, which no release script enables.

use std::sync::Arc;

use voltrus::api::AppState;
use voltrus::edition::{
    ActionContext, BoxFuture, Extension, ExtensionRoutes, ExternalIdentity, PasswordChangeKind,
    PasswordCheck, SessionPolicy, SignatureRecord, SignatureRequirement, StartContext,
};
use voltrus::license::LicenseStatus;
use voltrus::models::AppConfig;
use voltrus::storage::Storage;
use voltrus_enterprise::Enterprise;

struct TestNode(Enterprise);

impl Extension for TestNode {
    fn name(&self) -> &'static str {
        "enterprise (ha test node)"
    }

    fn resolve_license<'a>(&'a self, storage: &'a Storage) -> BoxFuture<'a, LicenseStatus> {
        // `VOLTRUS_TEST_LICENSE=real`: the real licence check, with the machine named by
        // `VOLTRUS_TEST_MACHINE` and activation files signed by the test key (see
        // `licensing::activation::test_support`), for the pair licensing tests.
        if std::env::var("VOLTRUS_TEST_LICENSE").as_deref() == Ok("real") {
            return self.0.resolve_license(storage);
        }
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

    fn validate_new_password<'a>(
        &'a self,
        check: &'a PasswordCheck,
    ) -> BoxFuture<'a, Result<(), String>> {
        self.0.validate_new_password(check)
    }

    fn password_changed<'a>(
        &'a self,
        username: &'a str,
        password_hash: &'a str,
        kind: PasswordChangeKind,
    ) -> BoxFuture<'a, ()> {
        self.0.password_changed(username, password_hash, kind)
    }

    fn password_expired<'a>(&'a self, user: &'a voltrus::models::User) -> BoxFuture<'a, bool> {
        self.0.password_expired(user)
    }

    fn session_policy<'a>(
        &'a self,
        username: &'a str,
        role: &'a str,
    ) -> BoxFuture<'a, SessionPolicy> {
        self.0.session_policy(username, role)
    }

    fn signature_required<'a>(
        &'a self,
        ctx: &'a ActionContext,
    ) -> BoxFuture<'a, Option<SignatureRequirement>> {
        self.0.signature_required(ctx)
    }

    fn record_signature<'a>(
        &'a self,
        record: &'a SignatureRecord,
    ) -> BoxFuture<'a, anyhow::Result<Option<String>>> {
        self.0.record_signature(record)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    voltrus::run(Arc::new(TestNode(Enterprise))).await
}
