use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

use voltrus::cache::DeviceCache;
use voltrus::edition::StartContext;
use voltrus::license::{LicenseStatus, LicenseTier};
use voltrus::models::AppConfig;
use voltrus::notification::email::EmailNotifier;
use voltrus::poller_manager::PollerManager;
use voltrus::storage::Storage;
use voltrus::ws_manager::WsManager;

use crate::ha::HaState;
use crate::hub::HubManager;
use crate::sms;
use crate::sso::oidc::{JwksManager, OidcConfig};
use crate::video::StreamProxy;
use crate::{licensing, reports};

/// State shared by the add-on's handlers and background tasks.
///
/// Created once in [`start`] and read through [`rt`]. A process-wide value keeps
/// handlers simple: they take the core `AppState` and read add-on state here.
pub struct Runtime {
    pub config: AppConfig,
    pub storage: Storage,
    pub cache: DeviceCache,
    pub ws_manager: Arc<WsManager>,
    pub poller_manager: PollerManager,
    pub license_status: Arc<RwLock<LicenseStatus>>,
    pub email_notifier: Option<EmailNotifier>,
    pub hub: HubManager,
    pub stream_proxy: Arc<StreamProxy>,
    pub ha_state: HaState,
    pub oidc: OidcConfig,
    pub jwks: JwksManager,
    hub_started: AtomicBool,
    scheduler_started: AtomicBool,
    ha_started: AtomicBool,
}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// The add-on runtime. Panics if called before the extension started.
pub fn rt() -> &'static Runtime {
    RUNTIME.get().expect("enterprise runtime is not started")
}

impl Runtime {
    /// Tier currently unlocked by the licence (live, follows activation and renewal).
    pub fn tier(&self) -> LicenseTier {
        self.license_status
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .tier()
    }

    /// Start the add-on services the current tier allows. Safe to call again after
    /// an activation or renewal: each service starts at most once.
    pub async fn apply_license_effects(&self) {
        let tier = self.tier();

        if tier.multi_site()
            && voltrus::node_role::is_active()
            && !self.hub_started.swap(true, Ordering::SeqCst)
        {
            match self.storage.get_enabled_hub_sites().await {
                Ok(sites) => {
                    for site in &sites {
                        self.hub
                            .start_site(site.clone(), self.cache.clone(), self.storage.clone())
                            .await;
                        tracing::info!(
                            "Started hub connector for site '{}' (id={})",
                            site.name,
                            site.id
                        );
                    }
                    if !sites.is_empty() {
                        tracing::info!("Started {} hub site connector(s)", sites.len());
                    }
                }
                Err(e) => tracing::warn!("Failed to load hub sites: {}", e),
            }
        }

        if tier.pdf_reporting() && !self.scheduler_started.swap(true, Ordering::SeqCst) {
            if let Some(ref notifier) = self.email_notifier {
                reports::scheduler::start_report_scheduler(
                    self.storage.clone(),
                    self.cache.clone(),
                    notifier.clone(),
                );
                tracing::info!("PDF report scheduler started");
            } else {
                tracing::info!(
                    "PDF reporting enabled but SMTP not configured: report scheduler not started"
                );
            }
        }

        if tier.ha() && self.ha_state.is_enabled() && !self.ha_started.swap(true, Ordering::SeqCst) {
            self.ha_state.start();
            tracing::info!(
                "HA enabled, node_id={}, role={}",
                self.config.ha.node_id,
                self.config.ha.role
            );
        }
    }

    /// Stop the services only the active node of a pair runs. They start again
    /// through [`Runtime::apply_license_effects`] when the node takes over.
    pub async fn pause_active_services(&self) {
        self.hub.stop_all().await;
        self.hub_started.store(false, Ordering::SeqCst);
    }
}

pub(crate) async fn start(ctx: StartContext) {
    voltrus::edition::register_sms_sender(Arc::new(sms::CarrierSms));
    voltrus::edition::register_hosted_ai(Arc::new(crate::ai::VoltrusAi));

    let oidc = OidcConfig {
        enabled: ctx.config.oidc_enabled,
        issuer: ctx.config.oidc_issuer.clone(),
        audience: ctx.config.oidc_audience.clone(),
        jwks_url: ctx.config.oidc_jwks_url.clone(),
        role_claim: ctx.config.oidc_role_claim.clone(),
        admin_role: ctx.config.oidc_admin_role.clone(),
    };
    let jwks = JwksManager::new(&oidc.jwks_url);
    if oidc.enabled {
        crate::sso::oidc::spawn_jwks_refresh(jwks.clone());
        tracing::info!("OIDC enabled, issuer: {}", oidc.issuer);
    }

    let ha_state = HaState::build(&ctx.config.ha, ctx.storage.clone(), ctx.poller_manager.clone())
        .unwrap_or_else(|e| {
            tracing::error!("Hot standby could not be set up: {e:#}");
            HaState::disabled()
        });

    let runtime = Runtime {
        config: ctx.config,
        storage: ctx.storage.clone(),
        cache: ctx.cache,
        ws_manager: ctx.ws_manager,
        poller_manager: ctx.poller_manager,
        license_status: ctx.license_status.clone(),
        email_notifier: ctx.email_notifier,
        hub: HubManager::new(),
        stream_proxy: Arc::new(StreamProxy::new("snapshots")),
        ha_state,
        oidc,
        jwks,
        hub_started: AtomicBool::new(false),
        scheduler_started: AtomicBool::new(false),
        ha_started: AtomicBool::new(false),
    };
    if RUNTIME.set(runtime).is_err() {
        tracing::warn!("enterprise runtime was already started");
        return;
    }

    licensing::online::spawn_renewal_task(ctx.storage, ctx.license_status);
    rt().apply_license_effects().await;
}
