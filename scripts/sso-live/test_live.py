#!/usr/bin/env python3
"""HTTP-level SSO checks against the live Keycloak and Samba servers (stdlib only).

Plays a browser: a cookie jar, redirects followed, the Keycloak login form posted, the
SAML auto-submit form posted to the ACS. Real-browser behaviour (SameSite handling) is
covered by the manual browser run described in docs/sso.md; this suite is the part that
repeats.

  test_live.py            all instances run.sh started
Environment: RUN_DIR (default ./.run).
"""
import html
import http.cookiejar
import json
import os
import re
import ssl
import sys
import urllib.error
import urllib.parse
import urllib.request

RUN = os.environ.get("RUN_DIR", os.path.join(os.path.dirname(os.path.abspath(__file__)), ".run"))
CTX = ssl.create_default_context()
CTX.check_hostname = False
CTX.verify_mode = ssl.CERT_NONE

# name -> (public url, admin-log dir, default role expectation for unmapped users)
INSTANCES = {
    "http": ("http://localhost:3100", "http", "viewer"),
    "https": ("https://scada.localhost:8443", "https", "viewer"),
    "strict": ("http://localhost:3102", "strict", None),  # default_role "": unmapped users are refused
    "tlsidp": ("http://localhost:3103", "tlsidp", "viewer"),  # Keycloak over https, private CA
}

KC_USERS = [  # user, password, expected role (None: refused)
    ("alice", "alice-pw", "admin"),
    ("bob", "bob-pw", "operator"),
    ("carol", "carol-pw", "DEFAULT"),
    ("dave", "dave-pw", None),  # disabled at the IdP
    ("alice", "wrong", None),
]
LDAP_USERS = [
    ("alice", "Alice-Pw-1!", "admin"),
    ("bob", "Bob-Pw-1!", "operator"),  # only a member of a group nested inside SCADA Operators
    ("carol", "Carol-Pw-1!", "DEFAULT"),
    ("dave", "Dave-Pw-1!", None),  # disabled in AD
    ("alice", "wrong", None),
    ("alice", "", None),
]

failures = []
passed = 0


def check(name, ok, detail=""):
    global passed
    if ok:
        passed += 1
        print(f"  ok   {name}")
    else:
        failures.append(name)
        print(f"  FAIL {name} {detail}")


class Browser:
    def __init__(self):
        # Chrome treats localhost and 127.0.0.1 as secure contexts and keeps Secure cookies there.
        self.jar = http.cookiejar.CookieJar(http.cookiejar.DefaultCookiePolicy(secure_protocols=("http", "https")))
        self.op = urllib.request.build_opener(
            urllib.request.HTTPCookieProcessor(self.jar), urllib.request.HTTPSHandler(context=CTX))

    def open(self, url, data=None, headers=None, method=None):
        req = urllib.request.Request(url, data=data, headers=headers or {}, method=method)
        try:
            r = self.op.open(req, timeout=30)
            return r.status, r.read().decode("utf-8", "replace"), r.geturl()
        except urllib.error.HTTPError as e:
            return e.code, e.read().decode("utf-8", "replace"), e.geturl()

    def form(self, url, fields):
        return self.open(url, urllib.parse.urlencode(fields).encode(),
                         {"Content-Type": "application/x-www-form-urlencoded"})

    def me(self, base):
        _, body, _ = self.open(base + "/api/v1/auth/me")
        return json.loads(body)

    def cookie(self, name):
        return next((c for c in self.jar if c.name == name), None)


def kc_login(b, page, user, pw):
    """Post the credentials on a Keycloak login page; returns (status, body, url)."""
    status, body, url = page
    m = re.search(r'<form[^>]*id="kc-form-login"[^>]*action="([^"]+)"', body)
    if not m:
        return status, body, url
    return b.form(html.unescape(m.group(1)), {"username": user, "password": pw, "credentialId": ""})


def saml_fields(body):
    m = re.search(r'<form[^>]*action="([^"]+)"', body)
    if not m or "SAMLResponse" not in body:
        return None
    fields = dict((html.unescape(k), html.unescape(v))
                  for k, v in re.findall(r'<input[^>]*name="([^"]+)"[^>]*value="([^"]*)"', body))
    return html.unescape(m.group(1)), fields


def login_saml(b, base, user, pw):
    """SP-initiated SAML. Returns the SAMLResponse form (action, fields) or None."""
    page = b.open(base + "/api/v1/auth/saml/login")
    page = kc_login(b, page, user, pw)
    form = saml_fields(page[1])
    if form:
        action, fields = form
        b.last = b.form(action, fields)
    return form


def login_oidc(b, base, user, pw):
    page = b.open(base + "/api/v1/auth/oidc/login")
    b.last = kc_login(b, page, user, pw)


def login_ldap(b, base, user, pw):
    b.last = b.open(base + "/api/v1/auth/ldap/login", json.dumps({"username": user, "password": pw}).encode(),
                    {"Content-Type": "application/json"})


def expect_role(expected, default):
    return default if expected == "DEFAULT" else expected


def run_logins(name, base, default):
    users = {"saml": KC_USERS, "oidc": KC_USERS, "ldap": LDAP_USERS}
    for kind, login in (("saml", login_saml), ("oidc", login_oidc), ("ldap", login_ldap)):
        for user, pw, role in users[kind]:
            role = expect_role(role, default)
            b = Browser()
            login(b, base, user, pw)
            me = b.me(base)
            label = f"[{name}] {kind} {user}/{'wrong' if 'wrong' in pw or not pw else 'ok'}"
            if role is None:
                check(f"{label} refused", not me.get("authenticated"), str(me))
            else:
                check(f"{label} -> {kind}:{user} role {role}",
                      me.get("authenticated") and me.get("role") == role and me.get("username") == f"{kind}:{user}",
                      str(me))


def run_session(name, base):
    """Logout, then log in again; no cookie survives logout."""
    for kind, login, user, pw in (("ldap", login_ldap, "alice", "Alice-Pw-1!"), ("oidc", login_oidc, "alice", "alice-pw")):
        b = Browser()
        login(b, base, user, pw)
        check(f"[{name}] {kind} session established", b.me(base).get("authenticated"))
        st, _, _ = b.open(base + "/api/v1/auth/logout", b"", {"Content-Type": "application/json"}, "POST")
        check(f"[{name}] {kind} logout", st == 200 and not b.me(base).get("authenticated"))
        # Keycloak's own session lives on: signing in again is immediate for OIDC, a form for LDAP.
        login(b, base, user, pw)
        check(f"[{name}] {kind} login again after logout", b.me(base).get("authenticated"))


def run_saml_hardening(name, base):
    # replay: the same SAMLResponse posted by a second browser is refused
    b = Browser()
    form = login_saml(b, base, "alice", "alice-pw")
    check(f"[{name}] saml baseline login", b.me(base).get("authenticated"))
    if not form:
        return
    b2 = Browser()
    st, body, _ = b2.form(*form)
    check(f"[{name}] saml response replayed in another browser refused", not b2.me(base).get("authenticated"), f"{st}")
    # replay in the same browser (a second use of the assertion)
    b.open(base + "/api/v1/auth/logout", b"", {"Content-Type": "application/json"}, "POST")
    st, body, _ = b.form(*form)
    check(f"[{name}] saml assertion replayed after logout refused", not b.me(base).get("authenticated"), f"{st}")
    # IdP-initiated (unsolicited) response
    b3 = Browser()
    slug = "voltrus-" + urllib.parse.urlsplit(base).netloc.replace(":", "-").replace(".", "-")
    page = b3.open(f"http://127.0.0.1:8081/realms/voltrus/protocol/saml/clients/{slug}")
    page = kc_login(b3, page, "alice", "alice-pw")
    form = saml_fields(page[1])
    check(f"[{name}] keycloak issued an IdP-initiated response", form is not None, page[1][:200])
    if form:
        b3.form(*form)
        check(f"[{name}] saml IdP-initiated response refused", not b3.me(base).get("authenticated"))


def admin_password(logdir):
    log = open(os.path.join(RUN, logdir, "node.log")).read()
    m = re.search(r"GENERATED ADMIN PASSWORD: (\S+)", log)
    return m.group(1) if m else None


def run_account_rules(name, base, logdir):
    pw = admin_password(logdir)
    if not pw:
        print(f"  skip [{name}] account rules: no generated admin password in the log (database was reused)")
        return
    admin = Browser()
    admin.open(base + "/api/v1/auth/login", json.dumps({"username": "admin", "password": pw}).encode(),
               {"Content-Type": "application/json"})
    users = {u["username"]: u["id"] for u in json.loads(admin.open(base + "/api/v1/users")[1])}

    def put(uid, body):
        return admin.open(f"{base}/api/v1/users/{uid}", json.dumps(body).encode(), {"Content-Type": "application/json"}, "PUT")[0]

    # a local account disabled in Voltrus is refused even though the IdP accepts the login
    uid = users["saml:alice"]
    put(uid, {"disabled": True})
    b = Browser()
    login_saml(b, base, "alice", "alice-pw")
    check(f"[{name}] saml login of a Voltrus-disabled account refused", not b.me(base).get("authenticated"))
    put(uid, {"disabled": False})
    b = Browser()
    login_saml(b, base, "alice", "alice-pw")
    check(f"[{name}] saml login after re-enabling works", b.me(base).get("authenticated"))

    # a role changed by hand is overwritten by the directory at the next sign-in
    put(users["ldap:bob"], {"role": "viewer"})
    b = Browser()
    login_ldap(b, base, "bob", "Bob-Pw-1!")
    check(f"[{name}] role edited by hand is re-applied from the directory", b.me(base).get("role") == "operator")

    # an external identity cannot be used with the local login form
    b = Browser()
    st = b.open(base + "/api/v1/auth/login", json.dumps({"username": "saml:alice", "password": ""}).encode(),
                {"Content-Type": "application/json"})[0]
    check(f"[{name}] external account has no usable local password", st in (400, 401, 403), str(st))


def run_metadata(name, base):
    b = Browser()
    st, body, _ = b.open(base + "/api/v1/auth/saml/metadata")
    check(f"[{name}] SP metadata served", st == 200 and f"{base}/api/v1/auth/saml/acs" in body)
    check(f"[{name}] providers list saml, oidc, ldap",
          {p["provider"] for p in json.loads(b.open(base + "/api/v1/auth/providers")[1])["providers"]} >= {"saml", "oidc", "ldap"})


def run_cookies(name, base):
    b = Browser()
    st, body, _ = b.open(base + "/api/v1/auth/saml/login")
    c = b.cookie("voltrus_saml_binding")
    check(f"[{name}] saml binding cookie set", c is not None)
    if base.startswith("https"):
        # Behind a TLS proxy the binding cookie must be SameSite=None; Secure so the
        # cross-site POST from the IdP carries it. Read the raw header.
        req = urllib.request.Request(base + "/api/v1/auth/saml/login")
        opener = urllib.request.build_opener(urllib.request.HTTPSHandler(context=CTX), NoRedirect)
        try:
            opener.open(req)
        except urllib.error.HTTPError as e:
            ck = [v for k, v in e.headers.items() if k.lower() == "set-cookie"]
            check(f"[{name}] binding cookie is SameSite=None; Secure", any("SameSite=None" in v and "Secure" in v for v in ck), str(ck))


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *a, **k):
        return None


NoRedirect = _NoRedirect()


def main():
    only = sys.argv[1:] or list(INSTANCES)
    for name in only:
        base, logdir, default = INSTANCES[name]
        print(f"== {name}: {base}")
        try:
            urllib.request.urlopen(base + "/api/v1/auth/providers", context=CTX, timeout=5)
        except Exception as e:
            print(f"  FAIL [{name}] not reachable: {e}")
            failures.append(f"{name} unreachable")
            continue
        run_metadata(name, base)
        run_logins(name, base, default)
        if name == "tlsidp":
            continue  # same flows as http; what differs is the https fetch of IdP metadata and discovery
        if name != "strict":
            run_cookies(name, base)
            run_session(name, base)
            run_saml_hardening(name, base)
            run_account_rules(name, base, logdir)
    print(f"\n{passed} passed, {len(failures)} failed")
    for f in failures:
        print("  FAILED:", f)
    sys.exit(1 if failures else 0)


main()
