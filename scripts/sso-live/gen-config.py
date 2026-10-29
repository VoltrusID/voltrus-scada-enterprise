#!/usr/bin/env python3
"""Write a Voltrus config.yaml with an `sso:` section for the live servers.

  gen-config.py <out.yaml> <port> <public_url> <db> <idp-metadata.xml>
Environment: DEFAULT_ROLE (default viewer; empty refuses unmapped users),
LDAP_URL, LDAP_CA_FILE, KC_ISSUER.
"""
import os
import sys

out, port, public_url, db, idp_md = sys.argv[1:6]
default_role = os.environ.get("DEFAULT_ROLE", "viewer")
issuer = os.environ.get("KC_ISSUER", "http://127.0.0.1:8081/realms/voltrus")
ldap_url = os.environ.get("LDAP_URL", "ldaps://dc1.localhost:636")
ldap_ca = os.environ.get("LDAP_CA_FILE", "")
ldap_pw = os.environ.get("LDAP_BIND_PASSWORD", "Svc-Pass-1!")
base_dn = os.environ.get("LDAP_BASE_DN", "DC=corp,DC=example")
cookie_secure = os.environ.get("COOKIE_SECURE", "auto")
# https IdP (Keycloak behind a private CA): fetch the SAML metadata by URL.
idp_metadata_url = os.environ.get("IDP_METADATA_URL", "")
idp_ca_file = os.environ.get("IDP_CA_FILE", "")
saml_idp = (f"idp_metadata_url: {idp_metadata_url}" if idp_metadata_url else f"idp_metadata_file: {idp_md}")
ca_line = f'    ca_cert_file: "{idp_ca_file}"\n' if idp_ca_file else ""

cfg = f"""server_port: {port}
server_host: "0.0.0.0"
db_path: "{db}"
persist_interval_secs: 1
cookie_secure: {cookie_secure}
auth_secret: "sso-live-secret-sso-live-secret-0123456789"
oidc_issuer: {issuer}

sso:
  public_url: {public_url}
  saml:
    enabled: true
    label: Keycloak SAML
    {saml_idp}
{ca_line}    username_attribute: username
    email_attribute: email
    groups_attribute: groups
    role_mapping:
      default_role: "{default_role}"
      groups:
        - {{ group: "/scada-admins", role: admin }}
        - {{ group: "/scada-operators/night-shift", role: operator }}
  oidc:
    client_id: voltrus
{ca_line}    client_secret_env: VOLTRUS_OIDC_CLIENT_SECRET
    username_claim: preferred_username
    groups_claim: groups
    role_mapping:
      default_role: "{default_role}"
      groups:
        - {{ group: scada-admins, role: admin }}
        - {{ group: night-shift, role: operator }}
  ldap:
    enabled: true
    label: Samba AD
    url: {ldap_url}
    ca_cert_file: "{ldap_ca}"
    bind_dn: "CN=svc-voltrus,CN=Users,{base_dn}"
    bind_password_env: VOLTRUS_LDAP_BIND_PASSWORD
    base_dn: "{base_dn}"
    user_filter: "(sAMAccountName={{username}})"
    username_attribute: sAMAccountName
    nested_groups: true
    role_mapping:
      default_role: "{default_role}"
      groups:
        - {{ group: "CN=SCADA Admins,OU=Groups,{base_dn}", role: admin }}
        - {{ group: "SCADA Operators", role: operator }}
"""
open(out, "w").write(cfg)
