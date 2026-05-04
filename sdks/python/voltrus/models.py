"""Pydantic models for Voltrus SDK."""

from datetime import datetime
from typing import Optional, Union

from pydantic import BaseModel


class TagValue(BaseModel):
    id: str
    name: str
    value: Optional[Union[float, str, bool]] = None
    quality: str  # 'good' | 'bad' | 'unknown'
    timestamp: str
    device_id: str


class WriteResult(BaseModel):
    command_id: str
    status: str  # 'pending' | 'sent' | 'acked' | 'failed'
    message: str


class CommandStatus(BaseModel):
    command_id: str
    tag_id: str
    value: Union[float, str, bool]
    status: str  # 'pending' | 'sent' | 'acked' | 'failed' | 'cancelled'
    created_at: str
    updated_at: str
    performed_by: str
    plc_response: Optional[str] = None


class Alarm(BaseModel):
    id: str
    alarm_id: str
    source_key: str
    source_name: str
    severity: str  # 'critical' | 'warning' | 'info'
    message: str
    state: str  # 'active' | 'acknowledged' | 'shelved'
    triggered_at: str
    acknowledged_at: Optional[str] = None
    acknowledged_by: Optional[str] = None


class AlarmSummary(BaseModel):
    active: int
    acknowledged: int
    shelved: int
    critical: int
    warning: int
    info: int


class Device(BaseModel):
    id: str
    name: str
    protocol: str  # 'modbus' | 'opcua' | 's7' | 'enip' | 'dnp3' | 'bacnet' | 'mqtt'
    enabled: bool
    online: bool
    ip: Optional[str] = None
    port: Optional[int] = None


class Screen(BaseModel):
    id: str
    name: str
    created_at: str


class OeeMetrics(BaseModel):
    availability: float
    performance: float
    quality: float
    oee: float
    period_start: str
    period_end: str


class DataPoint(BaseModel):
    timestamp: str
    value: Union[float, str, bool]
    quality: str
