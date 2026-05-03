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
        // Screens
        crate::api::screens::list_screens,
        crate::api::screens::create_screen,
        crate::api::screens::update_screen,
        crate::api::screens::delete_screen,
        crate::api::screens::list_screen_sensors,
        crate::api::screens::add_screen_sensor,
        crate::api::screens::update_screen_sensor,
        crate::api::screens::remove_screen_sensor,
        // Layouts
        crate::api::layouts::get_layouts,
        crate::api::layouts::save_layouts,
        crate::api::layouts::get_twin_layouts,
        crate::api::layouts::save_twin_layouts,
        crate::api::layouts::get_twin_connections,
        crate::api::layouts::save_twin_connections,
        crate::api::layouts::save_twin_camera,
        crate::api::layouts::get_canvas_layouts,
        crate::api::layouts::save_canvas_layouts,
        crate::api::layouts::get_scada_equipment,
        crate::api::layouts::save_scada_equipment,
        crate::api::layouts::get_scada_connections,
        crate::api::layouts::save_scada_connections,
        // Data Sources
        crate::api::data_sources::list_data_sources,
        crate::api::data_sources::create_data_source,
        crate::api::data_sources::update_data_source,
        crate::api::data_sources::delete_data_source,
        crate::api::data_sources::reorder_data_sources,
        // Flows
        crate::api::flows::list_flows,
        crate::api::flows::create_flow,
        crate::api::flows::get_flow,
        crate::api::flows::update_flow,
        crate::api::flows::delete_flow,
        crate::api::flows::start_flow,
        crate::api::flows::stop_flow,
        crate::api::flows::test_flow,
        crate::api::flows::list_flow_runs,
        // Notifications
        crate::api::notifications::list_notification_settings,
        crate::api::notifications::create_notification_setting,
        crate::api::notifications::update_notification_setting,
        crate::api::notifications::delete_notification_setting,
        crate::api::notifications::test_notification,
        // Push
        crate::api::push::subscribe,
        crate::api::push::unsubscribe,
        crate::api::push::vapid_public_key,
        // Stream
        crate::api::stream::stream,
        // Shortcuts
        crate::api::shortcuts::get_shortcuts,
        crate::api::shortcuts::save_shortcuts,
        crate::api::shortcuts::get_nav_shortcuts,
        crate::api::shortcuts::save_nav_shortcuts,
        // API Keys
        crate::api::api_keys::create_api_key,
        crate::api::api_keys::list_api_keys,
        crate::api::api_keys::revoke_api_key,
        // User Notifications
        crate::api::user_notifications::list_notifications,
        crate::api::user_notifications::mark_read,
        crate::api::user_notifications::dismiss_notification,
        // OEE
        crate::api::oee::get_oee_metrics,
        crate::api::oee::list_downtime,
        crate::api::oee::create_downtime,
        crate::api::oee::close_downtime,
        crate::api::oee::list_shifts,
        crate::api::oee::create_shift,
        crate::api::oee::delete_shift,
        // Analytics
        crate::api::analytics::get_aggregate,
        crate::api::analytics::get_compare,
        crate::api::analytics::get_spc,
        crate::api::analytics::get_histogram,
        crate::api::analytics::get_run_chart,
        crate::api::analytics::list_custom_metrics,
        crate::api::analytics::create_custom_metric,
        crate::api::analytics::update_custom_metric,
        crate::api::analytics::delete_custom_metric,
        crate::api::analytics::evaluate_all_custom_metrics,
        // Hub
        crate::api::hub::list_sites,
        crate::api::hub::create_site,
        crate::api::hub::update_site,
        crate::api::hub::delete_site,
        crate::api::hub::start_site_connection,
        crate::api::hub::stop_site_connection,
        crate::api::hub::get_hub_status,
        // Reports
        crate::api::reports::list_reports,
        crate::api::reports::create_report,
        crate::api::reports::update_report,
        crate::api::reports::delete_report,
        crate::api::reports::generate_report,
        // Recipes
        crate::api::recipes::list_recipes,
        crate::api::recipes::create_recipe,
        crate::api::recipes::get_recipe,
        crate::api::recipes::update_recipe,
        crate::api::recipes::delete_recipe,
        crate::api::recipes::load_recipe,
        // RBAC
        crate::api::rbac::list_roles,
        crate::api::rbac::create_role,
        crate::api::rbac::update_role,
        crate::api::rbac::delete_role,
        crate::api::rbac::assign_user_role,
        // Compliance
        crate::api::compliance::get_compliance_config,
        crate::api::compliance::update_compliance_config,
        crate::api::compliance::create_electronic_signature,
        crate::api::compliance::list_electronic_signatures,
        // Security
        crate::api::security::list_security_events,
        crate::api::security::export_security_csv,
        crate::api::security::get_security_thresholds,
        crate::api::security::update_security_thresholds,
        // Annotations
        crate::api::annotations::create_annotation,
        crate::api::annotations::list_annotations,
        crate::api::annotations::get_annotation,
        crate::api::annotations::update_annotation,
        crate::api::annotations::delete_annotation,
        // Shift Handover
        crate::api::shift_handover::create_shift_handover,
        crate::api::shift_handover::list_shift_handovers,
        crate::api::shift_handover::get_shift_handover,
        crate::api::shift_handover::auto_shift_report,
        // Video
        crate::api::video::list_cameras,
        crate::api::video::create_camera,
        crate::api::video::update_camera,
        crate::api::video::delete_camera,
        crate::api::video::trigger_snapshot,
        crate::api::video::list_snapshots,
        crate::api::video::delete_snapshot,
        crate::api::video::get_stream_info,
        // Compression
        crate::api::compression::get_compression_stats,
        crate::api::compression::get_compression_config,
        crate::api::compression::get_system_memory,
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
        // Screen schemas
        crate::models::Device,
        crate::models::DataSource,
        crate::models::ScreenDataSource,
        crate::models::ScreenLayoutRow,
        // Layout schemas
        crate::models::TwinLayout,
        crate::models::TwinConnection,
        crate::models::CanvasLayoutItem,
        crate::models::ScadaEquipment,
        crate::models::ScadaConnection,
        // Flow schemas
        crate::models::Flow,
        crate::models::FlowRun,
        // Notification schemas
        crate::models::NotificationSetting,
        crate::models::CreateNotificationSettingRequest,
        crate::models::UpdateNotificationSettingRequest,
        crate::models::TestNotificationRequest,
        crate::models::PushSubscription,
        crate::models::PushSubscribeRequest,
        crate::models::UserNotification,
        // API Key schemas
        crate::api::api_keys::CreateApiKeyRequest,
        // OEE schemas
        crate::models::OeeMetrics,
        crate::models::DowntimeEvent,
        crate::models::CreateDowntimeEventRequest,
        crate::models::CloseDowntimeEventRequest,
        crate::models::ShiftConfig,
        crate::models::CreateShiftRequest,
        // Analytics schemas
        crate::analytics::AggregatedMetric,
        crate::analytics::ComparisonResult,
        crate::models::SpcResult,
        crate::models::SpcPoint,
        crate::models::ControlLimits,
        crate::models::HistogramResult,
        crate::models::HistogramBin,
        crate::models::CurvePoint,
        crate::models::DistributionStats,
        crate::models::CapabilityIndices,
        crate::models::RunChartResult,
        crate::models::RunChartPoint,
        crate::models::Signal,
        crate::models::CustomMetric,
        crate::models::CustomMetricValue,
        crate::models::CreateCustomMetricRequest,
        crate::models::UpdateCustomMetricRequest,
        // Hub schemas
        crate::models::HubSite,
        crate::models::CreateHubSiteRequest,
        crate::models::UpdateHubSiteRequest,
        // Report schemas
        crate::models::Report,
        crate::models::CreateReportRequest,
        crate::models::UpdateReportRequest,
        // Recipe schemas
        crate::models::Recipe,
        crate::models::CreateRecipeRequest,
        crate::api::recipes::UpdateRecipeRequest,
        // RBAC schemas
        crate::models::Role,
        crate::models::CreateRoleRequest,
        crate::models::UpdateRoleRequest,
        crate::models::AssignRoleRequest,
        // Compliance schemas
        crate::compliance::ComplianceConfig,
        crate::compliance::UpdateComplianceConfigRequest,
        crate::compliance::CreateSignatureRequest,
        crate::compliance::ElectronicSignature,
        // Security schemas
        crate::models::SecurityEvent,
        crate::models::SecurityThresholds,
        // Annotation schemas
        crate::models::Annotation,
        crate::models::CreateAnnotationRequest,
        crate::models::UpdateAnnotationRequest,
        // Shift Handover schemas
        crate::models::ShiftHandover,
        crate::models::CreateShiftHandoverRequest,
        // Video schemas
        crate::models::Camera,
        crate::models::CameraSnapshot,
        crate::models::CreateCameraRequest,
        crate::models::UpdateCameraRequest,
        // Compression schemas
        crate::api::compression::CompressionStatsResponse,
        crate::api::compression::CompressionConfigResponse,
        crate::api::compression::MemoryStatsResponse,
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
