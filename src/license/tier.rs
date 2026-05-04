
// ── License Tier ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LicenseTier {
    Trial,
    Starter,
    Professional,
    Enterprise,
}

impl LicenseTier {
    /// Maps an activation tier string to a LicenseTier enum.
    /// Returns Trial when the input is None or unrecognized.
    pub fn from_tier_str(s: Option<&str>) -> Self {
        match s.map(|t| t.to_lowercase()) {
            Some(ref t) if t == "starter" => LicenseTier::Starter,
            Some(ref t) if t == "professional" || t == "pro" => LicenseTier::Professional,
            Some(ref t) if t == "enterprise" => LicenseTier::Enterprise,
            _ => LicenseTier::Trial,
        }
    }

    /// Maximum number of devices allowed. None means unlimited.
    pub fn max_devices(&self) -> Option<u32> {
        match self {
            LicenseTier::Trial => Some(5),
            LicenseTier::Starter => Some(50),
            LicenseTier::Professional => None,
            LicenseTier::Enterprise => None,
        }
    }

    /// Number of days telemetry data is retained.
    pub fn data_retention_days(&self) -> u32 {
        match self {
            LicenseTier::Trial => 7,
            LicenseTier::Starter | LicenseTier::Professional | LicenseTier::Enterprise => 365,
        }
    }

    pub fn opcua_enabled(&self) -> bool {
        matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
    }

    pub fn email_notifications(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn webhook_notifications(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn push_notifications(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn sms_notifications(&self) -> bool {
        matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
    }

    pub fn rest_api_export(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn csv_export(&self) -> bool {
        true
    }

    pub fn analytics(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn oee(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn pdf_reporting(&self) -> bool {
        matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
    }

    pub fn backup_restore(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn auto_discovery(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    pub fn white_label(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    /// Full theme editor (colors, fonts, layouts).
    /// Starter gets basic (logo swap), Professional+ gets full theme control.
    pub fn theme_full(&self) -> bool {
        matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
    }

    /// Per-user screen layouts (each user gets their own layout).
    /// Professional tier and above.
    pub fn per_user_layouts(&self) -> bool {
        matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
    }

    pub fn rbac(&self) -> bool {
        matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
    }

    pub fn multi_site(&self) -> bool {
        matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
    }

    pub fn recipes(&self) -> bool {
        matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
    }

    pub fn sso(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    /// SSO with SAML/OIDC enhanced features and LDAP fallback.
    /// Only available on Enterprise tier.
    pub fn sso_saml(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    pub fn cfr_part_11(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    /// EtherNet/IP (Allen-Bradley) protocol support.
    /// Only available on Enterprise tier.
    pub fn ethernet_ip(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    /// Siemens S7 protocol support.
    /// Only available on Enterprise tier.
    pub fn s7_protocol(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    /// DNP3 (IEEE 1815) protocol support — Enterprise only.
    pub fn dnp3(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    /// BACnet/IP protocol support — Enterprise only.
    pub fn bacnet(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    /// RTSP video integration.
    /// Only available on Enterprise tier.
    pub fn rtsp_video(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }

    /// AI Copilot assistant.
    /// Starter tier and above.
    pub fn ai_copilot(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    /// Embedded MQTT broker.
    /// Starter tier and above.
    pub fn mqtt_broker(&self) -> bool {
        !matches!(self, LicenseTier::Trial)
    }

    /// Returns all feature flags as a JSON object.
    pub fn to_feature_flags(&self) -> serde_json::Value {
        serde_json::json!({
            "max_devices": self.max_devices(),
            "data_retention_days": self.data_retention_days(),
            "opcua_enabled": self.opcua_enabled(),
            "email_notifications": self.email_notifications(),
            "webhook_notifications": self.webhook_notifications(),
            "push_notifications": self.push_notifications(),
            "sms_notifications": self.sms_notifications(),
            "rest_api_export": self.rest_api_export(),
            "csv_export": self.csv_export(),
            "analytics": self.analytics(),
            "oee": self.oee(),
            "pdf_reporting": self.pdf_reporting(),
            "backup_restore": self.backup_restore(),
            "auto_discovery": self.auto_discovery(),
            "white_label": self.white_label(),
            "theme_full": self.theme_full(),
            "per_user_layouts": self.per_user_layouts(),
            "rbac": self.rbac(),
            "multi_site": self.multi_site(),
            "recipes": self.recipes(),
            "sso": self.sso(),
            "sso_saml": self.sso_saml(),
            "cfr_part_11": self.cfr_part_11(),
            "ethernet_ip": self.ethernet_ip(),
            "s7_protocol": self.s7_protocol(),
            "dnp3": self.dnp3(),
            "bacnet": self.bacnet(),
            "rtsp_video": self.rtsp_video(),
            "mqtt_broker": self.mqtt_broker(),
        })
    }
}
