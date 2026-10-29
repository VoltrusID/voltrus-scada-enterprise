#!/usr/bin/env python3
"""Provision the `voltrus` realm in a local Keycloak (stdlib only).

  keycloak-setup.py realm                 recreate realm, users, groups, OIDC client
  keycloak-setup.py saml <public_url>     import Voltrus' SP metadata as a SAML client

Environment: KC_URL (default http://127.0.0.1:8081), KC_ADMIN / KC_ADMIN_PASSWORD,
OIDC_SECRET (confidential client secret).
"""
import json
import os
import ssl
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

KC = os.environ.get("KC_URL", "http://127.0.0.1:8081")
REALM = "voltrus"
OIDC_SECRET = os.environ.get("OIDC_SECRET", "oidc-live-secret")
INSECURE = ssl.create_default_context()
INSECURE.check_hostname = False
INSECURE.verify_mode = ssl.CERT_NONE

USERS = [
    # username, password, enabled, groups
    ("alice", "alice-pw", True, ["scada-admins"]),
    ("bob", "bob-pw", True, ["scada-operators/night-shift"]),
    ("carol", "carol-pw", True, []),
    ("dave", "dave-pw", False, ["scada-admins"]),
]


def call(method, path, body=None, token=None, raw=False, ctype="application/json"):
    data = None
    if body is not None:
        data = body if isinstance(body, (bytes, str)) else json.dumps(body)
        data = data.encode() if isinstance(data, str) else data
    req = urllib.request.Request(KC + path, data=data, method=method)
    if body is not None:
        req.add_header("Content-Type", ctype)
    if token:
        req.add_header("Authorization", "Bearer " + token)
    try:
        with urllib.request.urlopen(req, context=INSECURE) as r:
            payload = r.read()
            if raw:
                return r.status, payload, r.headers
            return r.status, (json.loads(payload) if payload else None), r.headers
    except urllib.error.HTTPError as e:
        sys.exit(f"{method} {path} -> {e.code}: {e.read().decode()[:400]}")


def admin_token():
    form = urllib.parse.urlencode({
        "grant_type": "password", "client_id": "admin-cli",
        "username": os.environ.get("KC_ADMIN", "admin"),
        "password": os.environ.get("KC_ADMIN_PASSWORD", "admin"),
    }).encode()
    deadline = time.time() + 180
    while True:
        try:
            req = urllib.request.Request(KC + "/realms/master/protocol/openid-connect/token", data=form)
            with urllib.request.urlopen(req, context=INSECURE) as r:
                return json.load(r)["access_token"]
        except Exception as e:  # not up yet
            if time.time() > deadline:
                sys.exit(f"keycloak not ready: {e}")
            time.sleep(2)


def location_id(headers):
    return headers["Location"].rsplit("/", 1)[1]


def setup_realm(tok):
    call("DELETE", f"/admin/realms/{REALM}", token=tok) if realm_exists(tok) else None
    call("POST", "/admin/realms", {
        "realm": REALM, "enabled": True, "sslRequired": "none",
        "registrationAllowed": False, "loginWithEmailAllowed": True,
        "bruteForceProtected": False,
    }, tok)
    base = f"/admin/realms/{REALM}"

    # Groups: scada-admins, scada-operators with a child, scada-viewers.
    gid = {}
    for name in ("scada-admins", "scada-operators", "scada-viewers"):
        _, _, h = call("POST", base + "/groups", {"name": name}, tok)
        gid[name] = location_id(h)
    _, _, h = call("POST", f"{base}/groups/{gid['scada-operators']}/children", {"name": "night-shift"}, tok)
    gid["scada-operators/night-shift"] = location_id(h)

    for username, password, enabled, groups in USERS:
        _, _, h = call("POST", base + "/users", {
            "username": username, "enabled": enabled, "emailVerified": True,
            "email": f"{username}@corp.example", "firstName": username.title(), "lastName": "Live",
            "credentials": [{"type": "password", "value": password, "temporary": False}],
        }, tok)
        uid = location_id(h)
        for g in groups:
            call("PUT", f"{base}/users/{uid}/groups/{gid[g]}", token=tok)

    # OIDC confidential client with a group-membership claim named `groups`.
    redirects = [f"{u}/api/v1/auth/oidc/callback" for u in public_urls()]
    call("POST", base + "/clients", {
        "clientId": "voltrus", "protocol": "openid-connect", "enabled": True,
        "publicClient": False, "secret": OIDC_SECRET, "standardFlowEnabled": True,
        "directAccessGrantsEnabled": False, "redirectUris": redirects,
        "attributes": {"pkce.code.challenge.method": "S256", "post.logout.redirect.uris": "+"},
        "protocolMappers": [{
            "name": "groups", "protocol": "openid-connect",
            "protocolMapper": "oidc-group-membership-mapper",
            "config": {"claim.name": "groups", "full.path": "false",
                       "id.token.claim": "true", "access.token.claim": "true",
                       "userinfo.token.claim": "true"},
        }],
    }, tok)
    print(f"realm {REALM} ready: users {[u[0] for u in USERS]}; OIDC client voltrus")


def realm_exists(tok):
    try:
        urllib.request.urlopen(urllib.request.Request(
            f"{KC}/admin/realms/{REALM}", headers={"Authorization": "Bearer " + tok}), context=INSECURE)
        return True
    except urllib.error.HTTPError:
        return False


def public_urls():
    return [u for u in os.environ.get("VOLTRUS_PUBLIC_URLS", "http://localhost:3100,https://scada.localhost:8443,http://localhost:3102,http://localhost:3103").split(",") if u]


def register_saml(tok, public_url):
    metadata_url = public_url.rstrip("/") + "/api/v1/auth/saml/metadata"
    with urllib.request.urlopen(metadata_url, context=INSECURE) as r:
        sp_metadata = r.read()
    # This is Keycloak's own SP-metadata import: a rejection here is a metadata bug.
    _, client, _ = call("POST", f"/admin/realms/{REALM}/client-description-converter",
                        sp_metadata, tok, ctype="application/xml")
    client["enabled"] = True
    client.setdefault("attributes", {})["saml.force.post.binding"] = "true"
    client["attributes"]["saml_name_id_format"] = "username"
    # Keycloak's importer does not honour AuthnRequestsSigned="false": it keeps
    # "Client signature required" on, and then rejects every unsigned request.
    # Voltrus signs requests only with sign_authn_requests: true.
    # IdP-initiated sign-in URL, to prove Voltrus refuses unsolicited responses.
    client["attributes"]["saml_idp_initiated_sso_url_name"] = "voltrus-" + urllib.parse.urlsplit(public_url).netloc.replace(":", "-").replace(".", "-")
    client["attributes"]["saml.client.signature"] = os.environ.get("SAML_CLIENT_SIGNATURE", "false")
    client["protocolMappers"] = [
        {"name": "username", "protocol": "saml", "protocolMapper": "saml-user-property-mapper",
         "config": {"user.attribute": "username", "attribute.name": "username",
                    "attribute.nameformat": "Basic", "friendly.name": "username"}},
        {"name": "email", "protocol": "saml", "protocolMapper": "saml-user-property-mapper",
         "config": {"user.attribute": "email", "attribute.name": "email",
                    "attribute.nameformat": "Basic", "friendly.name": "email"}},
        {"name": "groups", "protocol": "saml", "protocolMapper": "saml-group-membership-mapper",
         "config": {"attribute.name": "groups", "attribute.nameformat": "Basic",
                    "full.path": "true", "single": "false"}},
    ]
    existing = call("GET", f"/admin/realms/{REALM}/clients?clientId={urllib.parse.quote(client['clientId'])}", token=tok)[1]
    for c in existing:
        call("DELETE", f"/admin/realms/{REALM}/clients/{c['id']}", token=tok)
    call("POST", f"/admin/realms/{REALM}/clients", client, tok)
    print(f"SAML client {client['clientId']} imported from {metadata_url}")


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    tok = admin_token()
    if sys.argv[1] == "realm":
        setup_realm(tok)
    elif sys.argv[1] == "saml":
        register_saml(tok, sys.argv[2])
    else:
        sys.exit(__doc__)


main()
