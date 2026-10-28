"""Client for the Voltrus SCADA Enterprise add-on endpoints.

``VoltrusEnterpriseClient`` subclasses the public ``voltrus.VoltrusClient``, so
login, API keys, cookies and every Core method work unchanged. Method names
follow the REST surface, like the public client. Results are plain dicts and
lists exactly as the server sends them.

Against a Core-only server every method here raises ``VoltrusError`` with
``status == 404``. Licence-gated endpoints answer 403 when the licence tier is
too low.
"""

from typing import Any, Dict, List, Optional
from urllib.parse import urlencode

from voltrus import VoltrusClient


class VoltrusEnterpriseClient(VoltrusClient):
    """Public ``VoltrusClient`` plus the Enterprise endpoints.

    Usage:
        client = VoltrusEnterpriseClient("http://localhost:3000")
        client.login("admin", "secret")
        client.get_license_status()
        client.list_hub_sites()
    """

    # ── Licence ─────────────────────────────────────────────
    # get_license_status() is inherited from the public client.

    def activate_license(
        self,
        license_key: Optional[str] = None,
        activation_base64: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Activate online with ``license_key`` or offline with the signed
        activation file (``activation_base64``). Send exactly one.
        Returns ``{"ok": True, "status": {...}}`` (same shape as the status endpoint)."""
        if (license_key is None) == (activation_base64 is None):
            raise ValueError("Pass exactly one of license_key or activation_base64")
        body: Dict[str, Any] = (
            {"license_key": license_key}
            if license_key is not None
            else {"activation_base64": activation_base64}
        )
        return self._request("POST", "/api/license/activate", body=body)

    def request_license(self, license_key: str) -> Dict[str, Any]:
        """Offline step 1. Returns ``{"request_base64": ...}``, the request file
        to upload at activate.voltrus.id."""
        return self._request(
            "POST", "/api/license/request", body={"license_key": license_key}
        )

    def refresh_license(self) -> Dict[str, Any]:
        """Renew a monthly key online. Returns ``{"ok": True, "status": {...}}``."""
        return self._request("POST", "/api/license/refresh")

    # ── Compliance (21 CFR Part 11), admin only ─────────────

    def get_compliance_config(self) -> Dict[str, Any]:
        """Compliance settings (auto logout, password policy, rotation)."""
        return self._request("GET", "/api/v1/compliance/config")

    def update_compliance_config(self, **changes: Any) -> Dict[str, Any]:
        """Update compliance settings. Only the keyword arguments you pass change:
        auto_logout_minutes, password_min_length, password_require_uppercase,
        password_require_number, password_require_special,
        password_rotation_days, authority_to_change."""
        return self._request("PUT", "/api/v1/compliance/config", body=changes)

    def sign_record(
        self, resource_type: str, resource_id: str, reason: str, meaning: str
    ) -> Dict[str, Any]:
        """Sign a resource as the logged-in user. ``reason`` and ``meaning`` must
        not be blank. Returns ``{"ok", "id", "signature_hash", "created_at"}``."""
        return self._request(
            "POST",
            "/api/v1/compliance/sign",
            body={
                "resource_type": resource_type,
                "resource_id": resource_id,
                "reason": reason,
                "meaning": meaning,
            },
        )

    def list_signatures(
        self,
        resource_type: Optional[str] = None,
        resource_id: Optional[str] = None,
    ) -> List[Dict[str, Any]]:
        """Electronic signatures, optionally filtered by resource."""
        return self._request(
            "GET",
            "/api/v1/compliance/signatures",
            params={"resource_type": resource_type, "resource_id": resource_id},
        )

    # ── Security audit (IEC 62443), admin only ──────────────

    def list_security_events(
        self,
        event_type: Optional[str] = None,
        severity: Optional[str] = None,
        from_ts: Optional[int] = None,
        to_ts: Optional[int] = None,
        limit: Optional[int] = None,
        offset: Optional[int] = None,
    ) -> Dict[str, Any]:
        """Security events. Returns ``{"events": [...], "count": n}``. The server
        defaults to 100 events and caps ``limit`` at 1000."""
        return self._request(
            "GET",
            "/api/v1/security/events",
            params={
                "event_type": event_type,
                "severity": severity,
                "from": from_ts,
                "to": to_ts,
                "limit": limit,
                "offset": offset,
            },
        )

    def export_security_events_csv(
        self, from_ts: Optional[int] = None, to_ts: Optional[int] = None
    ) -> str:
        """Security events as CSV text."""
        return self._request(
            "GET",
            "/api/v1/security/events/export",
            params={"from": from_ts, "to": to_ts},
            raw=True,
        )

    def get_security_thresholds(self) -> Dict[str, Any]:
        """Alert thresholds: failed_login, suspicious_401 and rate_limit, each as
        ``*_threshold`` and ``*_window_secs``."""
        return self._request("GET", "/api/v1/security/thresholds")

    def update_security_thresholds(self, thresholds: Dict[str, int]) -> Dict[str, Any]:
        """Replace all six threshold values. Each threshold must be greater than 0."""
        return self._request("PUT", "/api/v1/security/thresholds", body=thresholds)

    # ── Multi-site hub ──────────────────────────────────────

    def list_hub_sites(self) -> List[Dict[str, Any]]:
        """Remote sites. The API key is never returned."""
        return self._request("GET", "/api/v1/hub/sites")

    def create_hub_site(
        self, name: str, url: str, api_key: str, enabled: bool = True
    ) -> Dict[str, Any]:
        """Add a remote site. ``url`` must start with http:// or https://."""
        return self._request(
            "POST",
            "/api/v1/hub/sites",
            body={"name": name, "url": url, "api_key": api_key, "enabled": enabled},
        )

    def update_hub_site(self, site_id: int, **changes: Any) -> None:
        """Update a site. Keyword arguments: name, url, api_key, enabled."""
        self._request("PUT", f"/api/v1/hub/sites/{site_id}", body=changes)

    def delete_hub_site(self, site_id: int) -> None:
        """Delete a site and stop its connection."""
        self._request("DELETE", f"/api/v1/hub/sites/{site_id}")

    def start_hub_site(self, site_id: int) -> Dict[str, Any]:
        """Start the connection to a site. The site must be enabled."""
        return self._request("POST", f"/api/v1/hub/sites/{site_id}/start")

    def stop_hub_site(self, site_id: int) -> Dict[str, Any]:
        """Stop the connection to a site."""
        return self._request("POST", f"/api/v1/hub/sites/{site_id}/stop")

    def get_hub_status(self) -> Dict[str, Any]:
        """Every site with its connection state: ``{"sites": [...], "total": n}``."""
        return self._request("GET", "/api/v1/hub/status")

    # ── Cameras (RTSP video), admin only ────────────────────

    def list_cameras(self) -> List[Dict[str, Any]]:
        """Configured cameras, each with ``has_snapshots``."""
        return self._request("GET", "/api/v1/cameras")

    def create_camera(
        self,
        name: str,
        url: str,
        enabled: bool = True,
        snapshot_on_alarm: bool = True,
        retention_hours: Optional[int] = None,
    ) -> Dict[str, Any]:
        """Add a camera. ``url`` must start with rtsp://. Returns ``{"ok", "id"}``."""
        body: Dict[str, Any] = {
            "name": name,
            "url": url,
            "enabled": enabled,
            "snapshot_on_alarm": snapshot_on_alarm,
        }
        if retention_hours is not None:
            body["retention_hours"] = retention_hours
        return self._request("POST", "/api/v1/cameras", body=body)

    def update_camera(self, camera_id: int, **changes: Any) -> Dict[str, Any]:
        """Update a camera. Keyword arguments: name, url, enabled,
        snapshot_on_alarm, retention_hours."""
        return self._request("PUT", f"/api/v1/cameras/{camera_id}", body=changes)

    def delete_camera(self, camera_id: int) -> Dict[str, Any]:
        """Delete a camera."""
        return self._request("DELETE", f"/api/v1/cameras/{camera_id}")

    def get_camera_stream(self, camera_id: int) -> Dict[str, Any]:
        """Stream details: camera_id, name, stream_url, enabled, protocol."""
        return self._request("GET", f"/api/v1/cameras/{camera_id}/stream")

    def trigger_snapshot(self, camera_id: int) -> Dict[str, Any]:
        """Capture a snapshot now. The camera must be enabled."""
        return self._request("POST", f"/api/v1/cameras/{camera_id}/snapshot")

    def list_snapshots(self, camera_id: int) -> List[Dict[str, Any]]:
        """Latest 100 snapshots of a camera."""
        return self._request("GET", f"/api/v1/cameras/{camera_id}/snapshots")

    def delete_snapshot(self, camera_id: int, snapshot_id: int) -> Dict[str, Any]:
        """Delete one snapshot."""
        return self._request(
            "DELETE", f"/api/v1/cameras/{camera_id}/snapshots/{snapshot_id}"
        )

    # ── Scheduled reports ───────────────────────────────────

    def list_reports(self) -> List[Dict[str, Any]]:
        """Configured reports."""
        return self._request("GET", "/api/v1/reports")

    def create_report(self, config: Dict[str, Any]) -> Dict[str, Any]:
        """Create a report. Required: ``name``, ``schedule`` ("daily", "weekly" or
        "monthly") and ``email_recipients`` (a JSON array of addresses as text,
        for example ``'["ops@example.com"]'``). Optional: ``device_ids`` ("all"
        or a JSON array as text) and ``enabled``. Returns the created report."""
        return self._request("POST", "/api/v1/reports", body=config)

    def update_report(self, report_id: int, config: Dict[str, Any]) -> Dict[str, Any]:
        """Update a report. Only the keys you pass change. Returns the report."""
        return self._request("PUT", f"/api/v1/reports/{report_id}", body=config)

    def delete_report(self, report_id: int) -> None:
        """Delete a report."""
        self._request("DELETE", f"/api/v1/reports/{report_id}")

    def generate_report(self, report_id: int) -> str:
        """Generate a report now. Returns the HTML report document."""
        return self._request("POST", f"/api/v1/reports/{report_id}/generate", raw=True)

    # ── Branding ────────────────────────────────────────────

    def get_branding(self) -> Dict[str, Any]:
        """Branding: title, subtitle, logo, theme_level ("none", "basic", "full").
        Public endpoint; returns the defaults without a white-label licence."""
        return self._request("GET", "/api/v1/settings/branding")

    def update_branding(
        self,
        title: Optional[str] = None,
        subtitle: Optional[str] = None,
        logo: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Update branding. Only the values you pass change."""
        body = {
            k: v
            for k, v in {"title": title, "subtitle": subtitle, "logo": logo}.items()
            if v is not None
        }
        return self._request("PUT", "/api/v1/settings/branding", body=body)

    # ── Hot standby ─────────────────────────────────────────

    def get_ha_status(self) -> Dict[str, Any]:
        """Role (``primary``, ``standby``, ``fenced``), epoch, peer and sync state,
        replication lag and the last failover of this node."""
        return self._request("GET", "/api/v1/ha/status")

    def force_ha_sync(self, wait_secs: Optional[int] = None) -> Dict[str, Any]:
        """Run a full resynchronisation of the standby from a fresh snapshot of the
        primary (admin only). Returns ``{"status": "synced", "report": {...}}``, or
        ``{"status": "sync_in_progress"}`` when it is still running after
        ``wait_secs`` (default 30). 400 when HA is not enabled, 409 without a peer."""
        return self._request("POST", "/api/v1/ha/force-sync" + _query(wait_secs=wait_secs))

    def force_ha_failover(
        self, wait_secs: Optional[int] = None, force: bool = False
    ) -> Dict[str, Any]:
        """Switch the roles of the pair (admin only). On the primary: a controlled
        switchover. On a linked standby: asks the primary to hand over. On a standby
        without a link: takes over; ``force`` skips the data and reachability
        checks. Returns once the roles have changed:
        ``{"status": "failover_complete", "new_role", "epoch", "duration_ms"}``.
        409 with the reason when it cannot."""
        return self._request(
            "POST",
            "/api/v1/ha/force-failover"
            + _query(wait_secs=wait_secs, force="true" if force else None),
        )

    # ── SSO ─────────────────────────────────────────────────

    def list_sso_providers(self) -> List[Dict[str, Any]]:
        """SSO providers (OIDC, SAML, LDAP) with their ``enabled`` flag, from
        ``GET /api/v1/auth/providers``. The local provider is left out. A Core-only
        server returns an empty list."""
        data = self._request("GET", "/api/v1/auth/providers")
        return [p for p in data.get("providers", []) if p.get("provider") != "local"]


def _query(**params: Any) -> str:
    """``?a=1&b=2`` for the parameters that are set, else an empty string."""
    given = {k: v for k, v in params.items() if v is not None}
    return "?" + urlencode(given) if given else ""
