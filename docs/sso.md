# Single sign-on

Enterprise licence required (`sso` for OIDC, `sso_saml` for SAML and LDAP). Every
login route checks the licence at request time. A server without the licence
answers 403 on them and does not list the providers on the login page.

Four ways in, all ending in the same session local login creates (the
`voltrus_token` and `voltrus_refresh` cookies), the same security events
(`login_success`, `login_failed`) and the same account model:

| Provider | Flow | Routes |
|----------|------|--------|
| SAML 2.0 | SP-initiated, HTTP-Redirect request, HTTP-POST response | `GET /api/v1/auth/saml/login`, `POST /api/v1/auth/saml/acs`, `GET /api/v1/auth/saml/metadata` |
| OpenID Connect | Authorization code with PKCE | `GET /api/v1/auth/oidc/login`, `GET /api/v1/auth/oidc/callback` |
| LDAP / Active Directory | Password form on the login page | `POST /api/v1/auth/ldap/login` |
| OIDC bearer tokens | `Authorization: Bearer` for API clients | any authenticated route |

Azure AD / Entra ID, Okta, Keycloak and ADFS are used through SAML or OIDC;
Active Directory directly through LDAP.

## Accounts and roles

External users become local accounts named `<provider>:<id>` (`saml:alice@corp.example`,
`ldap:alice`, `oidc:alice`). The prefix means an external identity can never
collide with, or take over, a local account such as `admin`. They have no usable
password. Disable one in Users like any other account.

The role comes from the groups the provider reports and `role_mapping`, and is
re-applied at every sign-in, so a change in the directory reaches Voltrus the
next time the user logs in (a role changed by hand in Users is overwritten then).
When several groups match, the highest role wins: `admin` > `operator` > `user` >
`demo` > `viewer`.

```yaml
role_mapping:
  default_role: viewer      # users matching nothing; "" refuses their login instead
  groups:
    - { group: "CN=SCADA Admins,OU=Groups,DC=corp,DC=example", role: admin }
    - { group: "scada-operators", role: operator }
```

`group` is a name, an LDAP group DN or an IdP group value. Matching ignores case
and, for DNs, spaces around `,` and `=`. LDAP groups are matched by full DN and by CN.

## Configuration

Everything is the `sso:` section of the server's config file (`config.yaml`).
Secrets can come from the environment (`*_env` fields) so they stay out of the file.
The server must be reachable by browsers at `public_url`; redirect and ACS URLs are
built from it.

```yaml
sso:
  public_url: https://scada.example.com

  saml:
    enabled: true
    label: "Azure AD"
    idp_metadata_url: https://login.microsoftonline.com/<tenant-id>/federationmetadata/2007-06/federationmetadata.xml?appid=<app-id>
    groups_attribute: http://schemas.microsoft.com/ws/2008/06/identity/claims/groups
    username_attribute: http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress
    role_mapping:
      default_role: viewer
      groups:
        - { group: "<group object id>", role: admin }

  oidc:
    client_id: voltrus
    client_secret_env: VOLTRUS_OIDC_CLIENT_SECRET
    username_claim: preferred_username
    groups_claim: groups
    role_mapping:
      groups:
        - { group: scada-admins, role: admin }

  ldap:
    enabled: true
    url: ldaps://dc1.corp.example:636
    ca_cert_file: /etc/voltrus/corp-ca.pem
    bind_dn: CN=svc-voltrus,OU=Service,DC=corp,DC=example
    bind_password_env: VOLTRUS_LDAP_BIND_PASSWORD
    base_dn: DC=corp,DC=example
    user_filter: (sAMAccountName={username})
    username_attribute: sAMAccountName
    nested_groups: true
    role_mapping:
      groups:
        - { group: "CN=SCADA Admins,OU=Groups,DC=corp,DC=example", role: admin }

# OIDC browser sign-in uses the issuer from the top level, shared with bearer validation:
oidc_issuer: https://login.example.com/realms/voltrus
oidc_enabled: true      # only for Authorization: Bearer tokens
oidc_jwks_url: https://login.example.com/realms/voltrus/protocol/openid-connect/certs
```

A provider with an invalid configuration is left out and the reason is logged
(`SAML sign-in disabled: ...`); the others keep working. Check the server log at
startup.

### SAML

Register Voltrus at the IdP with the SP metadata from
`https://<public_url>/api/v1/auth/saml/metadata`, or by hand: entity ID
`<public_url>/api/v1/auth/saml/metadata`, ACS (HTTP-POST)
`<public_url>/api/v1/auth/saml/acs`. Both can be overridden with `sp_entity_id`
and `acs_url`.

IdP trust comes from one of: `idp_metadata_url` (https, re-fetched every
`metadata_refresh_hours`, default 12, and once on an unknown signing key, so
certificate rollover needs no restart), `idp_metadata_file`, `idp_metadata_xml`,
or the explicit `idp_entity_id` + `idp_sso_url` + `idp_certificate`
(`idp_certificates` for several). Only signing certificates pinned this way can
vouch for a login; keys inside the response are ignored.

What is enforced on every response:

* The Response or the Assertion carries a valid XML signature (exclusive
  canonicalization, RSA-SHA256/384/512; SHA-1 only with `allow_sha1: true`).
  Claims are read only from the element the verified signature is attached to.
  Documents with a DTD, duplicate IDs, several assertions, encrypted assertions,
  or signatures anywhere unexpected are refused.
* Issuer is the configured IdP; `Destination` and every `Recipient` are our ACS URL;
  our entity ID is in the `AudienceRestriction`; `NotBefore`/`NotOnOrAfter` hold
  within `clock_skew_secs` (default 60).
* `InResponseTo` names a login this server started (kept 10 minutes, one use),
  and the response comes back in the browser that started it (a cookie; enforced
  when `cookie_secure` is on, which is the default).
* Each assertion ID is accepted once (replay protection).
* IdP-initiated logins (no `InResponseTo`) are refused unless `allow_idp_initiated: true`.

Not supported: encrypted assertions (turn encryption off at the IdP; the channel
is TLS), ECDSA signatures, SAML logout, artifact binding. `sign_authn_requests: true`
with `sp_private_key`/`sp_private_key_file` signs the AuthnRequest (RSA-SHA256);
publish `sp_certificate` so the IdP can verify it.

Typical attribute settings: Azure AD groups claim
`http://schemas.microsoft.com/ws/2008/06/identity/claims/groups` (object IDs),
Okta a `groups` attribute statement, Keycloak the `Role` attribute or a group list mapper.

### IdPs behind a private CA

`idp_metadata_url`, OIDC discovery, the token endpoint and the JWKS are fetched over
HTTPS with the built-in public roots. A Keycloak, ADFS or other IdP whose certificate
comes from a private CA needs that CA: `sso.saml.ca_cert_file` (or inline `ca_cert`,
PEM) for the SAML metadata, `sso.oidc.ca_cert_file` / `ca_cert` for OIDC (browser
sign-in and bearer-token keys). The CA is added to the public roots; host names are
still verified. `SSL_CERT_FILE` and the system store are not consulted.

### OIDC browser sign-in

Create a confidential client at the provider with redirect URI
`<public_url>/api/v1/auth/oidc/callback` (override with `redirect_uri`). Setting
`client_id` turns the button on. Voltrus discovers the endpoints from
`<oidc_issuer>/.well-known/openid-configuration`, sends a PKCE (S256) code
challenge, a `state` bound to the browser by cookie and a `nonce`, and validates the
ID token (signature from the provider's JWKS with key rotation, issuer, audience =
`client_id`, expiry, nonce). Without `client_secret` it runs as a public client with
PKCE only. `client_auth: basic` sends the secret in a header instead of the body.

Azure AD: use the v2.0 issuer `https://login.microsoftonline.com/<tenant>/v2.0`,
and app roles (`groups_claim: roles`) rather than groups when users belong to many
groups (Azure replaces the list with a link above 200).

### LDAP and Active Directory

Passwords are verified by binding as the user; nothing is compared locally.

* **Search mode** (set `bind_dn`): bind as a service account, find the user with
  `user_filter` under `base_dn`, bind as the found DN with the typed password.
* **Direct mode** (set `bind_dn_format` instead, e.g. `{username}@corp.example`
  or `uid={username},ou=people,dc=corp,dc=example`): bind as the user, then read
  their entry.

Transport: `ldaps://` or `ldap://` with `starttls: true`, certificate and host name
verified against the system roots or, when set, only the CA in `ca_cert_file`/`ca_cert`.
Plain `ldap://` is refused unless `allow_plain_ldap: true`, which sends passwords
in clear text on the network. There is no option to skip certificate verification.

Groups: the `memberOf` attribute (`member_of_attribute`), plus, with
`nested_groups: true`, an Active Directory transitive-membership search
(`1.2.840.113556.1.4.1941`) for groups reached through other groups. For other
directories set `group_filter`, e.g. `(&(objectClass=groupOfNames)(member={user_dn}))`
with `group_base_dn`. Usernames are escaped in filters and DNs; empty passwords
are refused before any bind (many servers treat them as anonymous success).
Login attempts share the rate limits of local login.

## Testing against real servers

The automated tests cover signature verification against documents signed by
openssl/libxml2, the LDAP client against an in-process LDAP server with TLS and
StartTLS, and the OIDC and SAML clients against stubs. They cannot cover an IdP's
quirks, so `scripts/sso-live/` runs the real thing:

```sh
scripts/sso-live/run.sh up     # Keycloak, Samba AD DC, Caddy, four Voltrus instances
scripts/sso-live/run.sh test   # HTTP-level suite (python3, stdlib only), ~30 s
scripts/sso-live/run.sh down   # remove containers, volumes, images and ./.run
```

Needs docker with compose, python3, openssl, curl and a Rust toolchain; the first
`up` builds `voltrus-ha-test-node`, the real Enterprise add-on with a stub Enterprise
licence (feature `ha-test-node`, never in a release build). `BIN=` uses a prebuilt one.
`STATIC_DIR=` (default `core/static` when built) serves the web UI for manual browser
runs. The images are pulled on `up` (about 1.5 GB) and deleted by `down`.

What `up` creates:

* **Keycloak** (dev mode, `127.0.0.1:8081`), realm `voltrus`: users `alice` (group
  `scada-admins`), `bob` (child group `scada-operators/night-shift`), `carol` (no
  group), `dave` (disabled); a confidential OIDC client `voltrus` with a `groups`
  claim; one SAML client per Voltrus instance, created by posting
  `/api/v1/auth/saml/metadata` to Keycloak's client importer, so a metadata Keycloak
  rejects fails `up`.
* **Samba AD DC** (`ldaps://dc1.localhost:636`, realm `corp.example`): users alice,
  bob, carol, dave (disabled) and `svc-voltrus`; groups `SCADA Admins`,
  `SCADA Operators` and, nested inside it, `SCADA Night Shift` (bob's only group). Its
  autogenerated server certificate has no subjectAltName and no verifying client can
  accept it, so the script replaces it with one from a throwaway CA (`.run/ldap-ca.pem`).
  Do the same on a real DC: use a certificate with a SAN, from your AD CS.
* **Voltrus instances**: `http` (`http://localhost:3100`), `https`
  (`https://scada.localhost:8443` through Caddy with its internal CA, so
  `cookie_secure: auto` turns on and the SAML binding cookie is `SameSite=None;
  Secure`), `strict` (`default_role: ""`, unmapped users refused) and `tlsidp`
  (Keycloak reached as `https://kc.localhost:8444` through Caddy, with
  `idp_metadata_url` and OIDC discovery trusting Caddy's CA through `ca_cert_file`).

The suite logs in through SAML (SP-initiated), OIDC and LDAP as every user, checks the
role, the refusals (disabled user, wrong or empty password, unmapped user with
`default_role: ""`, a Voltrus-disabled account), logout and logging in again, a role
edited by hand being re-applied, and SAML replay and IdP-initiated responses being
refused.

Things the real servers showed:

* Keycloak's SAML client importer keeps "Client signature required" on even though the
  metadata says `AuthnRequestsSigned="false"`; Keycloak then rejects every request with
  `SigAlg was null` ("Invalid requester" in the browser). Turn it off, or set
  `sign_authn_requests: true` and publish `sp_certificate`.
* Keycloak's group-membership mapper emits `/scada-operators/night-shift` with
  `full.path: true` (SAML here) and only `night-shift` with `false` (OIDC here): map the
  value the token carries. Only the direct group is listed, not its parents.
* There is no SAML logout: Voltrus logout ends the Voltrus session, the IdP session
  lives on, and the next SAML or OIDC sign-in is immediate without a password prompt.
* A real browser run (managed Chromium, the web UI, the three login paths, the two
  transports) matched the HTTP suite; the only thing the suite cannot show is browser
  cookie policy, which the `https` instance exercises when you sign in by hand.

Other servers, by hand. OpenLDAP (a test directory with one user):

```sh
docker run -d --name ldap -p 1389:389 -p 1636:636 \
  -e LDAP_ADMIN_PASSWORD=admin -e LDAP_USERS=alice -e LDAP_PASSWORDS=alice-pw \
  -e LDAP_ROOT=dc=corp,dc=example bitnami/openldap
# sso.ldap: url: ldap://localhost:1389, allow_plain_ldap: true, bind_dn: cn=admin,dc=corp,dc=example,
# bind_password: admin, base_dn: ou=users,dc=corp,dc=example, user_filter: (uid={username})
```

`idp_metadata_url: http://...` is refused (https only), as is a non-loopback http
`oidc_issuer` (loopback http is allowed); for an IdP without a public certificate use `ca_cert_file` (see IdPs behind a private CA) or
`idp_metadata_file`.

Regenerating the signature fixtures (`enterprise/src/sso/saml/fixtures/generate.sh`)
needs `openssl` and `xmllint`.
