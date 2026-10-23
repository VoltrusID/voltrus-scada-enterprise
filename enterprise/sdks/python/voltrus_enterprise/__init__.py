"""Voltrus SCADA Enterprise SDK extension.

Adds the Enterprise add-on endpoints to the public ``voltrus`` client.
"""

from voltrus import VoltrusError

from .client import VoltrusEnterpriseClient

__all__ = ["VoltrusEnterpriseClient", "VoltrusError"]

__version__ = "0.47.1"
