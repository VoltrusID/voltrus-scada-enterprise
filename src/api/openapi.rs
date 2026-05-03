use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        // System
        crate::api::system::get_health,
        crate::api::system::get_live,
        crate::api::system::get_system,
        crate::api::system::get_version,
        // Auth
        crate::api::auth::login,
        crate::api::auth::refresh_token,
        crate::api::auth::logout,
        crate::api::auth::auth_me,
        crate::api::auth::get_csrf_token,
        crate::api::auth::list_auth_providers,
        crate::api::auth::saml_metadata,
        crate::api::auth::saml_acs,
        crate::api::auth::ldap_login,
        crate::api::auth::change_password,
        crate::api::auth::seed,
        // Settings
        crate::api::settings::get_branding,
        crate::api::settings::get_alarm_cooldown,
        crate::api::settings::update_branding,
        crate::api::settings::update_alarm_cooldown,
        // License
        crate::api::license::get_license_status,
        // Alarms
        crate::api::alarms::active_alarm_count,
        crate::api::alarms::list_alarms,
        crate::api::alarms::acknowledge_alarm,
        crate::api::alarms::acknowledge_all,
        crate::api::alarms::shelf_alarm,
        crate::api::alarms::unshelf_alarm,
        crate::api::alarms::list_alarm_audit,
        crate::api::alarms::list_alarm_analytics,
        crate::api::alarms::get_alarm_summary,
        // Commands
        crate::api::commands::write_tag,
        crate::api::commands::get_command_status,
        crate::api::commands::list_commands,
        crate::api::commands::cancel_command,
        crate::api::commands::get_command_audit,
        crate::api::commands::update_write_policy,
        // Data
        crate::api::data::get_data,
        crate::api::data::get_history,
        crate::api::data::export_csv,
        crate::api::data::export_data,
        // Modbus devices
        crate::api::modbus_devices::list_modbus_devices,
        crate::api::modbus_devices::create_modbus_device,
        crate::api::modbus_devices::update_modbus_device,
        crate::api::modbus_devices::delete_modbus_device,
        crate::api::modbus_devices::scan_modbus_registers,
        crate::api::modbus_devices::scan_network,
        crate::api::modbus_devices::replace_registers,
        crate::api::modbus_devices::restart_poller,
        crate::api::modbus_devices::get_poller_statuses,
        // OPC-UA devices
        crate::api::opcua_devices::list_devices,
        crate::api::opcua_devices::create_device,
        crate::api::opcua_devices::update_device,
        crate::api::opcua_devices::delete_device,
        crate::api::opcua_devices::list_nodes,
        crate::api::opcua_devices::add_node,
        crate::api::opcua_devices::remove_node,
        crate::api::opcua_devices::browse_device,
        crate::api::opcua_devices::start_device,
        crate::api::opcua_devices::stop_device,
        // S7 devices
        crate::api::s7_devices::list_devices,
        crate::api::s7_devices::create_device,
        crate::api::s7_devices::update_device,
        crate::api::s7_devices::delete_device,
        crate::api::s7_devices::list_tags,
        crate::api::s7_devices::add_tag,
        crate::api::s7_devices::remove_tag,
        crate::api::s7_devices::browse_device,
        crate::api::s7_devices::start_device,
        crate::api::s7_devices::stop_device,
        // EtherNet/IP devices
        crate::api::enip_devices::list_devices,
        crate::api::enip_devices::create_device,
        crate::api::enip_devices::update_device,
        crate::api::enip_devices::delete_device,
        crate::api::enip_devices::list_tags,
        crate::api::enip_devices::add_tag,
        crate::api::enip_devices::remove_tag,
        crate::api::enip_devices::browse_device,
        crate::api::enip_devices::start_device,
        crate::api::enip_devices::stop_device,
    ),
    components(schemas(
        crate::models::HealthResponse,
        crate::models::SystemResponse,
        crate::models::ProcessInfo,
        crate::models::LiveResponse,
        crate::models::LoginResponse,
        crate::models::BrandingSettings,
        crate::models::LoginRequest,
        crate::models::ChangePasswordRequest,
        crate::api::auth::SamlAcsRequest,
        crate::api::auth::LdapLoginRequest,
        // Alarm schemas
        crate::models::Alarm,
        crate::models::AlarmAuditEntry,
        crate::models::AlarmSummary,
        crate::models::AlarmAnalyticsEntry,
        crate::models::ShelfAlarmRequest,
        crate::api::alarms::AlarmListQuery,
        // Command schemas
        crate::models::Command,
        crate::models::CommandAuditEntry,
        crate::models::WriteRequest,
        crate::models::CommandQuery,
        crate::models::WritePolicyUpdate,
        // Data schemas
        crate::models::DataSourceData,
        crate::models::HistoryResponse,
        crate::models::DataSourceSummary,
        crate::models::DataQuery,
        crate::models::ExportResponse,
        crate::models::ExportMeta,
        crate::models::TelemetryExportRow,
        // Modbus schemas
        crate::models::ModbusDevice,
        crate::models::ScanRegisterRequest,
        crate::models::ScannedRegister,
        crate::models::ModbusRegisterInput,
        crate::models::PollerStatus,
        // OPC-UA schemas
        crate::models::OpcUaDevice,
        crate::models::OpcUaNode,
        crate::models::CreateOpcUaDeviceRequest,
        crate::models::UpdateOpcUaDeviceRequest,
        crate::models::CreateOpcUaNodeRequest,
        crate::models::OpcUaSecurityPolicy,
        crate::models::OpcUaSecurityMode,
        // S7 schemas
        crate::models::S7TagRow,
        crate::models::CreateS7DeviceRequest,
        crate::models::UpdateS7DeviceRequest,
        crate::models::CreateS7TagRequest,
        // EtherNet/IP schemas
        crate::models::EnipTag,
        crate::models::CreateEnipDeviceRequest,
        crate::models::UpdateEnipDeviceRequest,
        crate::models::CreateEnipTagRequest,
    )),
    tags(
        (name = "Auth", description = "Authentication and user management"),
        (name = "Data", description = "Real-time data and history queries"),
        (name = "Devices", description = "Device management (Modbus, OPC-UA, S7, EtherNet/IP)"),
        (name = "Alarms", description = "Alarm management and analytics"),
        (name = "Commands", description = "Command acknowledgment and control"),
        (name = "System", description = "System health, info, and memory"),
        (name = "Compression", description = "Data compression stats and config"),
        (name = "Screens", description = "Dashboard screen management"),
        (name = "Layouts", description = "Screen layout management"),
        (name = "Data Sources", description = "Data source configuration"),
        (name = "Flows", description = "Data flow engine rules"),
        (name = "Notifications", description = "Notification settings and push subscriptions"),
        (name = "Stream", description = "Server-Sent Events real-time stream"),
        (name = "OEE", description = "Overall Equipment Effectiveness metrics"),
        (name = "Analytics", description = "Data aggregation and comparison"),
        (name = "Hub", description = "Multi-site hub management"),
        (name = "Reports", description = "Report generation and scheduling"),
        (name = "Recipes", description = "Recipe and parameter management"),
        (name = "RBAC", description = "Role-based access control"),
        (name = "Compliance", description = "FDA 21 CFR Part 11 compliance"),
        (name = "Security", description = "IEC 62443 security audit events"),
        (name = "Annotations", description = "Data point annotations"),
        (name = "Shift Handover", description = "Shift handover logs and reports"),
        (name = "Video", description = "RTSP camera management"),
        (name = "API Keys", description = "API key management"),
        (name = "Settings", description = "Application settings"),
        (name = "License", description = "License status"),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
            );
            components.add_security_scheme(
                "cookie_auth",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("voltrus_token"))),
            );
        } else {
            let components = utoipa::openapi::ComponentsBuilder::new()
                .security_scheme(
                    "api_key",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
                )
                .security_scheme(
                    "cookie_auth",
                    SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("voltrus_token"))),
                )
                .build();
            openapi.components = Some(components);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_generates_valid_json() {
        let spec = ApiDoc::openapi();
        let json = serde_json::to_string_pretty(&spec)
            .expect("OpenAPI spec should serialize to JSON");
        assert!(!json.is_empty());
        assert!(json.contains("\"openapi\""));
        assert!(json.contains("\"paths\""));
        assert!(json.contains("\"components\""));
        assert!(json.contains("\"securitySchemes\""));
        assert!(json.contains("\"api_key\""));
        assert!(json.contains("\"cookie_auth\""));
    }

    #[test]
    fn test_openapi_spec_has_tags() {
        let spec = ApiDoc::openapi();
        let tags = spec.tags.as_ref().expect("should have tags");
        let tag_names: Vec<&str> = tags.iter().map(|t| t.name.as_str()).collect();
        assert!(tag_names.contains(&"Auth"));
        assert!(tag_names.contains(&"Data"));
        assert!(tag_names.contains(&"Devices"));
        assert!(tag_names.contains(&"Alarms"));
        assert!(tag_names.contains(&"Commands"));
        assert!(tag_names.contains(&"System"));
    }

    #[test]
    fn test_openapi_spec_has_paths() {
        let spec = ApiDoc::openapi();
        let paths = &spec.paths;
        assert!(paths.paths.len() > 20, "Should have at least 20 paths, got {}", paths.paths.len());
        assert!(paths.paths.contains_key("/api/v1/health"), "Should have /api/v1/health");
        assert!(paths.paths.contains_key("/api/v1/data"), "Should have /api/v1/data");
        assert!(paths.paths.contains_key("/api/v1/auth/login"), "Should have /api/v1/auth/login");
    }
}
