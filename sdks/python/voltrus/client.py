"""Voltrus SCADA Python SDK client."""

import json
from typing import Any, Dict, List, Optional, Union

import urllib3

from .models import (
    Alarm,
    AlarmSummary,
    CommandStatus,
    DataPoint,
    Device,
    OeeMetrics,
    Screen,
    TagValue,
    WriteResult,
)


class VoltrusClient:
    """Official Python SDK for Voltrus SCADA.

    Usage:
        client = VoltrusClient("http://localhost:8080", api_key="your-key")
        token = client.login("admin", "password")
        tags = client.get_live_data("screen-id")
    """

    def __init__(
        self,
        base_url: str,
        api_key: Optional[str] = None,
        token: Optional[str] = None,
        timeout: int = 10,
    ) -> None:
        self._base_url = base_url.rstrip("/")
        self._timeout = timeout
        self._token = token

        headers: Dict[str, str] = {"Content-Type": "application/json"}
        if api_key:
            headers["X-API-Key"] = api_key
        if token:
            headers["Authorization"] = f"Bearer {token}"

        self._http = urllib3.PoolManager(headers=headers, timeout=urllib3.Timeout(total=timeout))

    def _url(self, path: str) -> str:
        return f"{self._base_url}{path}"

    def _request(self, method: str, path: str, **kwargs: Any) -> Any:
        url = self._url(path)
        resp = self._http.request(method, url, **kwargs)
        if resp.status >= 400:
            raise Exception(f"Voltrus API error {resp.status}: {resp.data.decode()}")
        if resp.status == 204:
            return None
        return json.loads(resp.data.decode())

    # ── Auth ────────────────────────────────────────────────

    def login(self, username: str, password: str) -> str:
        """Login and store token for subsequent requests."""
        data = self._request("POST", "/api/v1/auth/login", body=json.dumps({"username": username, "password": password}))
        token = data["token"]
        self._token = token
        self._http.headers["Authorization"] = f"Bearer {token}"
        return token

    # ── Tags / Data ────────────────────────────────────────

    def get_live_data(self, screen_id: str) -> List[TagValue]:
        """Read current tag values for a screen."""
        data = self._request("GET", f"/api/v1/screens/{screen_id}/data-sources")
        return [TagValue(**t) for t in data]

    def write_tag(self, tag_id: str, value: Union[float, str, bool]) -> WriteResult:
        """Write a value to a tag. Returns command_id for tracking."""
        data = self._request("POST", f"/api/v1/tags/{tag_id}/write", body=json.dumps({"value": value}))
        return WriteResult(**data)

    def get_command_status(self, command_id: str) -> CommandStatus:
        """Get command status by command_id."""
        data = self._request("GET", f"/api/v1/commands/{command_id}")
        return CommandStatus(**data)

    def get_commands(
        self, tag_id: Optional[str] = None, status: Optional[str] = None, limit: Optional[int] = None
    ) -> List[CommandStatus]:
        """Get command history with optional filters."""
        params: Dict[str, Any] = {}
        if tag_id:
            params["tag"] = tag_id
        if status:
            params["status"] = status
        if limit:
            params["limit"] = limit
        data = self._request("GET", "/api/v1/commands", fields=params)
        return [CommandStatus(**c) for c in data]

    def cancel_command(self, command_id: str) -> None:
        """Cancel a pending command."""
        self._request("POST", f"/api/v1/commands/{command_id}/cancel")

    def get_history(
        self,
        device_id: str,
        tag_name: str,
        from_ts: str,
        to_ts: str,
        limit: Optional[int] = None,
    ) -> List[DataPoint]:
        """Query historical data for a tag."""
        params: Dict[str, Any] = {
            "device_id": device_id,
            "tag_name": tag_name,
            "from": from_ts,
            "to": to_ts,
        }
        if limit:
            params["limit"] = limit
        data = self._request("GET", "/api/v1/history", fields=params)
        return [DataPoint(**d) for d in data]

    # ── Alarms ──────────────────────────────────────────────

    def list_alarms(
        self, state: Optional[str] = None, severity: Optional[str] = None, limit: Optional[int] = None
    ) -> List[Alarm]:
        """List alarms with optional filters."""
        params: Dict[str, Any] = {}
        if state:
            params["state"] = state
        if severity:
            params["severity"] = severity
        if limit:
            params["limit"] = limit
        data = self._request("GET", "/api/v1/alarms", fields=params)
        return [Alarm(**a) for a in data]

    def get_alarm_summary(self) -> AlarmSummary:
        """Get alarm summary counts."""
        data = self._request("GET", "/api/v1/alarms/summary")
        return AlarmSummary(**data)

    def acknowledge_alarm(self, alarm_id: str) -> None:
        """Acknowledge a specific alarm."""
        self._request("POST", f"/api/v1/alarms/{alarm_id}/acknowledge")

    def acknowledge_all_alarms(self) -> None:
        """Acknowledge all active alarms."""
        self._request("POST", "/api/v1/alarms")

    def shelf_alarm(self, alarm_id: str, duration_minutes: int) -> None:
        """Shelf an alarm (temporarily suppress)."""
        self._request(
            "POST",
            f"/api/v1/alarms/{alarm_id}/shelf",
            body=json.dumps({"duration_minutes": duration_minutes}),
        )

    def unshelf_alarm(self, alarm_id: str) -> None:
        """Unshelf an alarm."""
        self._request("POST", f"/api/v1/alarms/{alarm_id}/unshelf")

    # ── Devices ─────────────────────────────────────────────

    def list_modbus_devices(self) -> List[Device]:
        """List Modbus devices."""
        data = self._request("GET", "/api/v1/modbus-devices")
        return [Device(**d) for d in data]

    def list_opcua_devices(self) -> List[Device]:
        """List OPC-UA devices."""
        data = self._request("GET", "/api/v1/opcua/devices")
        return [Device(**d) for d in data]

    def list_s7_devices(self) -> List[Device]:
        """List S7 devices."""
        data = self._request("GET", "/api/v1/s7/devices")
        return [Device(**d) for d in data]

    def list_enip_devices(self) -> List[Device]:
        """List EtherNet/IP devices."""
        data = self._request("GET", "/api/v1/enip/devices")
        return [Device(**d) for d in data]

    def list_dnp3_devices(self) -> List[Device]:
        """List DNP3 devices."""
        data = self._request("GET", "/api/v1/dnp3/devices")
        return [Device(**d) for d in data]

    def list_bacnet_devices(self) -> List[Device]:
        """List BACnet devices."""
        data = self._request("GET", "/api/v1/bacnet/devices")
        return [Device(**d) for d in data]

    # ── OEE ─────────────────────────────────────────────────

    def get_oee_metrics(
        self, device_id: Optional[str] = None, from_ts: Optional[str] = None, to_ts: Optional[str] = None
    ) -> OeeMetrics:
        """Get OEE metrics."""
        params: Dict[str, Any] = {}
        if device_id:
            params["device_id"] = device_id
        if from_ts:
            params["from"] = from_ts
        if to_ts:
            params["to"] = to_ts
        data = self._request("GET", "/api/v1/oee/metrics", fields=params)
        return OeeMetrics(**data)

    # ── Screens ─────────────────────────────────────────────

    def list_screens(self) -> List[Screen]:
        """List all screens."""
        data = self._request("GET", "/api/v1/screens")
        return [Screen(**s) for s in data]

    # ── System ──────────────────────────────────────────────

    def health(self) -> Dict[str, Any]:
        """Health check."""
        return self._request("GET", "/api/v1/health")

    def get_system_info(self) -> Dict[str, Any]:
        """Get system info."""
        return self._request("GET", "/api/v1/system")

    def get_version(self) -> str:
        """Get Voltrus version."""
        data = self._request("GET", "/api/v1/version")
        return data.get("version", "unknown")
