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
openssl/libxml2, and the LDAP client against an in-process LDAP server with TLS and
StartTLS. They cannot cover your IdP's quirks. Manual checks:

OpenLDAP (a test directory with one user):

```sh
docker run -d --name ldap -p 1389:389 -p 1636:636 \
  -e LDAP_ADMIN_PASSWORD=admin -e LDAP_USERS=alice -e LDAP_PASSWORDS=alice-pw \
  -e LDAP_ROOT=dc=corp,dc=example bitnami/openldap
# sso.ldap: url: ldap://localhost:1389, allow_plain_ldap: true, bind_dn: cn=admin,dc=corp,dc=example,
# bind_password: admin, base_dn: ou=users,dc=corp,dc=example, user_filter: (uid={username})
```

Samba AD DC (`nowsci/samba-domain`) for Active Directory behaviour: `nested_groups`,
`sAMAccountName`, `ldaps://` with its generated CA.

Keycloak for SAML and OIDC:

```sh
docker run -p 8081:8080 -e KC_BOOTSTRAP_ADMIN_USERNAME=admin -e KC_BOOTSTRAP_ADMIN_PASSWORD=admin \
  quay.io/keycloak/keycloak start-dev
```

Create a realm, a client with the SP metadata (SAML) or the callback URL (OIDC), and
set `idp_metadata_url: http://...` is refused (https only): export the realm's IdP
metadata to a file and use `idp_metadata_file` for local tests.

Regenerating the signature fixtures (`enterprise/src/sso/saml/fixtures/generate.sh`)
needs `openssl` and `xmllint`.
