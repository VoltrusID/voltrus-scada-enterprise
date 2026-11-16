# Single sign-on

Sign in to Voltrus with your company accounts: SAML 2.0, OpenID Connect (OIDC) or
LDAP / Active Directory. Needs an **Enterprise** licence (feature flags `sso` for
OIDC, `sso_saml` for SAML and LDAP; see [licensing.md](licensing.md)).

Contents: [How it works](#how-it-works) ·
[Before you start](#before-you-start) ·
[Okta](#okta) · [Microsoft Entra ID](#microsoft-entra-id-azure-ad) ·
[Keycloak](#keycloak) · [Kanidm and other OIDC providers](#kanidm-and-other-oidc-providers) ·
[Active Directory over LDAP](#active-directory-over-ldap) ·
[Roles](#roles-and-group-mapping) · [Signing out](#signing-out) ·
[When the IdP is down](#when-the-identity-provider-is-down) ·
[Reference](#configuration-reference) · [Troubleshooting](#troubleshooting) ·
[Testing against real servers](#testing-against-real-servers)

## How it works

| Provider | Flow | Routes |
|----------|------|--------|
| SAML 2.0 | SP-initiated, HTTP-Redirect request, HTTP-POST response | `GET /api/v1/auth/saml/login`, `POST /api/v1/auth/saml/acs`, `GET /api/v1/auth/saml/metadata` |
| OpenID Connect | Authorization code with PKCE | `GET /api/v1/auth/oidc/login`, `GET /api/v1/auth/oidc/callback` |
| LDAP / Active Directory | Password form on the login page | `POST /api/v1/auth/ldap/login` |
| OIDC bearer tokens | `Authorization: Bearer` for API clients | any authenticated route |

* SSO is configured in the `sso:` section of the server's config file (the
  `config.yaml` you start Voltrus with: `voltrus /etc/voltrus/config.yaml`). There is
  no SSO settings page in the web UI. The file is read at startup: **restart the
  service after every change** (`sudo systemctl restart voltrus`).
* Each configured provider adds a button to the web login page, under the password
  form: **Sign in with &lt;label&gt;** for SAML and OIDC, and a
  username/password form titled with the LDAP label.
* The first sign-in creates the account (just-in-time provisioning). External users
  are named `<provider>:<id>` (`saml:alice@corp.example`, `oidc:alice`,
  `ldap:alice`), so an external identity can never collide with or take over a local
  account such as `admin`. They have no usable Voltrus password.
* The role comes from the user's groups at the IdP at **every** sign-in
  ([Roles](#roles-and-group-mapping)).
* Every sign-in ends in the same session cookies (`voltrus_token`,
  `voltrus_refresh`) and the same security events (`login_success`, `login_failed`,
  `login_disabled`) as a local login.
* The licence is checked on every request. Without an Enterprise licence the SSO
  routes answer 403 ("Single sign-on requires an Enterprise licence") and the login
  page shows no SSO buttons; local accounts keep working.
* The native desktop console (`voltrus-native`) signs in with local accounts only.
  It lists the server's SSO methods on its sign-in card and offers **Open the web
  sign-in**; SSO users use the web client.

## Before you start

1. **Licence.** Activate an Enterprise key ([licensing.md](licensing.md)) and check:

   ```sh
   curl -s https://scada.example.com/api/license/status
   # "tier":"enterprise", "feature_flags": {... "sso":true, "sso_saml":true ...}
   ```
2. **Public URL.** Browsers must reach Voltrus at one fixed address, ideally HTTPS
   behind your reverse proxy. Put it in `sso.public_url`; the callback and ACS URLs
   are built from it:

   | Value | Built from `public_url` |
   |-------|-------------------------|
   | OIDC redirect URI | `<public_url>/api/v1/auth/oidc/callback` |
   | SAML entity ID (and metadata URL) | `<public_url>/api/v1/auth/saml/metadata` |
   | SAML ACS (Assertion Consumer Service, HTTP-POST) | `<public_url>/api/v1/auth/saml/acs` |

3. **Keep a local admin.** The built-in `admin` account (and any local account) keeps
   signing in with its password whatever happens to the IdP. Do not delete it; store
   its password like any break-glass credential ([When the IdP is down](#when-the-identity-provider-is-down)).
4. **Secrets in the environment.** Every secret has an `*_env` variant
   (`client_secret_env`, `bind_password_env`, `sp_private_key_env`). With systemd:
   `sudo systemctl edit voltrus`, then

   ```ini
   [Service]
   Environment=VOLTRUS_OIDC_CLIENT_SECRET=...
   Environment=VOLTRUS_LDAP_BIND_PASSWORD=...
   ```

After a restart the log says which providers are on, and why one is off:

```
INFO  OIDC browser sign-in enabled, redirect https://scada.example.com/api/v1/auth/oidc/callback
INFO  SAML sign-in enabled, ACS https://scada.example.com/api/v1/auth/saml/acs
INFO  LDAP sign-in enabled, directory ldaps://dc1.corp.example:636
ERROR SAML sign-in disabled: <reason>      # a configuration error; the others keep working
```

`curl -s https://scada.example.com/api/v1/auth/providers` lists every provider with
`"enabled": true|false` (true only when it is configured **and** licensed).

## Okta

### Okta with OIDC (recommended)

In the Okta Admin Console:

1. **Applications › Applications › Create App Integration**, sign-in method
   **OIDC - OpenID Connect**, application type **Web Application**, **Next**.
2. **App integration name**: `Voltrus SCADA`. **Grant type**: Authorization Code.
   **Sign-in redirect URIs**: `https://scada.example.com/api/v1/auth/oidc/callback`.
   **Assignments**: the groups allowed to use Voltrus. **Save**.
3. On the **General** tab copy the **Client ID** and **Client secret**.
4. **Sign On** tab › **OpenID Connect ID Token** › **Edit**: **Groups claim type**
   Filter, **Groups claim filter** `groups` *Matches regex* `.*` (or *Starts with*
   `scada-` to send only the relevant groups). **Save**.
5. Create the groups (`scada-admins`, `scada-operators`) under
   **Directory › Groups** and add people.

Voltrus `config.yaml` (the issuer is your Okta org URL; with a custom authorization
server use `https://<org>.okta.com/oauth2/<server-id>` and add the groups claim
there instead):

```yaml
oidc_issuer: https://example.okta.com

sso:
  public_url: https://scada.example.com
  oidc:
    label: Okta
    client_id: 0oa1b2c3d4e5f6g7h8i9
    client_secret_env: VOLTRUS_OIDC_CLIENT_SECRET
    username_claim: preferred_username     # the Okta login (email)
    groups_claim: groups
    role_mapping:
      default_role: viewer
      groups:
        - { group: scada-admins, role: admin }
        - { group: scada-operators, role: operator }
```

### Okta with SAML

1. **Applications › Applications › Create App Integration › SAML 2.0**, name
   `Voltrus SCADA`, **Next**.
2. **Single sign-on URL**: `https://scada.example.com/api/v1/auth/saml/acs` (keep
   "Use this for Recipient URL and Destination URL" ticked).
   **Audience URI (SP Entity ID)**: `https://scada.example.com/api/v1/auth/saml/metadata`.
   **Name ID format**: EmailAddress. **Application username**: Email.
3. **Attribute Statements**: `email` = `user.email`.
   **Group Attribute Statements**: name `groups`, filter *Matches regex* `.*`.
   **Next**, **Finish**.
4. **Sign On** tab: copy the **Metadata URL**. **Assignments**: assign the groups.

```yaml
sso:
  public_url: https://scada.example.com
  saml:
    enabled: true
    label: Okta
    idp_metadata_url: https://example.okta.com/app/exk1a2b3c4/sso/saml/metadata
    groups_attribute: groups
    email_attribute: email
    role_mapping:
      default_role: viewer
      groups:
        - { group: scada-admins, role: admin }
        - { group: scada-operators, role: operator }
```

With no `username_attribute`, the user name is the SAML NameID (the email here).

## Microsoft Entra ID (Azure AD)

### Entra ID with OIDC (recommended)

In the Microsoft Entra admin center:

1. **Identity › Applications › App registrations › New registration**. Name
   `Voltrus SCADA`, **Accounts in this organizational directory only**, **Redirect
   URI**: platform **Web**, `https://scada.example.com/api/v1/auth/oidc/callback`.
   **Register**.
2. On **Overview** copy the **Application (client) ID** and the **Directory (tenant) ID**.
3. **Certificates & secrets › Client secrets › New client secret**; copy the
   **Value** (shown once).
4. Roles, one of:
   * **App roles** (best when people are in many groups): **App roles › Create app
     role**, e.g. value `scada-admins`, allowed member types Users/Groups. Then
     **Identity › Applications › Enterprise applications › Voltrus SCADA › Users and
     groups › Add user/group** and assign people or groups to the role. Use
     `groups_claim: roles`.
   * **Groups**: **Token configuration › Add groups claim**, **Security groups**, ID
     token **Group ID**. The token then carries group **object IDs**; map those. Above
     200 groups Entra replaces the list with a link, so prefer app roles.

```yaml
oidc_issuer: https://login.microsoftonline.com/<tenant-id>/v2.0

sso:
  public_url: https://scada.example.com
  oidc:
    label: Microsoft
    client_id: <application-client-id>
    client_secret_env: VOLTRUS_OIDC_CLIENT_SECRET
    username_claim: preferred_username      # user@corp.example
    groups_claim: roles                     # or "groups" for group object IDs
    role_mapping:
      default_role: ""                      # refuse people without a role
      groups:
        - { group: scada-admins, role: admin }
        - { group: scada-operators, role: operator }
```

Use the v2.0 issuer exactly as above; it must match the `iss` in the ID token.

### Entra ID with SAML

1. **Identity › Applications › Enterprise applications › New application › Create
   your own application**, name `Voltrus SCADA`, *Integrate any other application you
   don't find in the gallery (Non-gallery)*, **Create**.
2. **Single sign-on › SAML**. **Basic SAML Configuration › Edit**:
   **Identifier (Entity ID)** `https://scada.example.com/api/v1/auth/saml/metadata`,
   **Reply URL (Assertion Consumer Service URL)** `https://scada.example.com/api/v1/auth/saml/acs`.
   **Save**.
3. **Attributes & Claims › Edit › Add a group claim**: **Security groups**, source
   attribute **Group ID**. **Save**.
4. **SAML Certificates**: copy the **App Federation Metadata Url**. Leave signing at
   the default (sign the assertion, SHA-256).
5. **Users and groups**: assign who may sign in.

```yaml
sso:
  public_url: https://scada.example.com
  saml:
    enabled: true
    label: Microsoft
    idp_metadata_url: https://login.microsoftonline.com/<tenant-id>/federationmetadata/2007-06/federationmetadata.xml?appid=<app-id>
    groups_attribute: http://schemas.microsoft.com/ws/2008/06/identity/claims/groups
    username_attribute: http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress
    role_mapping:
      default_role: viewer
      groups:
        - { group: "<group object id>", role: admin }
```

Turn token encryption off (the default): Voltrus refuses encrypted assertions.

## Keycloak

Verified end to end with Keycloak 26 (OIDC and SAML) by `scripts/sso-live/`.

### Keycloak with OIDC

In the Keycloak admin console, realm `voltrus` (any realm works):

1. **Clients › Create client**: type **OpenID Connect**, **Client ID** `voltrus`,
   **Next**. **Client authentication** On, **Standard flow** ticked, **Next**.
   **Valid redirect URIs** `https://scada.example.com/api/v1/auth/oidc/callback`.
   **Save**.
2. **Credentials** tab: copy the **Client secret**.
3. **Client scopes** tab › `voltrus-dedicated` › **Configure a new mapper** (or
   **Add mapper › By configuration**) › **Group Membership**: name `groups`, **Token
   Claim Name** `groups`, **Full group path** Off, **Add to ID token** On. **Save**.
4. **Groups**: create `scada-admins`, `scada-operators`; **Users** › a user ›
   **Groups › Join Group**.

```yaml
oidc_issuer: https://sso.example.com/realms/voltrus

sso:
  public_url: https://scada.example.com
  oidc:
    label: Keycloak
    client_id: voltrus
    client_secret_env: VOLTRUS_OIDC_CLIENT_SECRET
    username_claim: preferred_username
    groups_claim: groups
    role_mapping:
      default_role: viewer
      groups:
        - { group: scada-admins, role: admin }
        - { group: night-shift, role: operator }   # subgroup: with Full group path off only its own name is sent
```

### Keycloak with SAML

1. Download the SP metadata: `curl -o voltrus-sp.xml https://scada.example.com/api/v1/auth/saml/metadata`
   (needs the Enterprise licence and `sso.saml.enabled: true` with any IdP metadata
   already set; or enter the values by hand).
2. **Clients › Import client**, **Resource file** `voltrus-sp.xml`, **Save**.
3. Client **Keys** tab: **Client signature required** Off (Keycloak keeps it on even
   though the metadata says requests are unsigned, and then rejects every login with
   "Invalid requester"). Or set `sign_authn_requests: true` with an SP key and
   certificate.
4. **Client scopes** tab › the dedicated scope › **Add mapper › By configuration ›
   Group list**: name `groups`, **Group attribute name** `groups`, **Full group
   path** On.

```yaml
sso:
  public_url: https://scada.example.com
  saml:
    enabled: true
    label: Keycloak
    idp_metadata_url: https://sso.example.com/realms/voltrus/protocol/saml/descriptor
    username_attribute: username       # optional; default is the NameID
    groups_attribute: groups
    role_mapping:
      default_role: viewer
      groups:
        - { group: /scada-admins, role: admin }
        - { group: /scada-operators/night-shift, role: operator }
```

## Kanidm and other OIDC providers

Any provider with OpenID Connect discovery
(`<issuer>/.well-known/openid-configuration`), the authorization code flow and
RS256/RS384/RS512 or ES256 ID tokens works the same way: create a confidential
client with the redirect URI, set `oidc_issuer`, `client_id`, the secret, the
username claim and the groups claim. Voltrus always sends PKCE (S256), a `state`
bound to the browser and a `nonce`.

Kanidm (from its documentation; the generic OIDC path is the one verified with
Keycloak):

```sh
kanidm system oauth2 create voltrus "Voltrus SCADA" https://scada.example.com
kanidm system oauth2 add-redirect-url voltrus https://scada.example.com/api/v1/auth/oidc/callback
kanidm group create scada_admins
kanidm group add-members scada_admins alice
kanidm system oauth2 update-scope-map voltrus scada_admins openid profile email groups
kanidm system oauth2 show-basic-secret voltrus
```

```yaml
oidc_issuer: https://idm.example.com/oauth2/openid/voltrus

sso:
  public_url: https://scada.example.com
  oidc:
    label: Kanidm
    client_id: voltrus
    client_secret_env: VOLTRUS_OIDC_CLIENT_SECRET
    username_claim: preferred_username
    groups_claim: groups
    role_mapping:
      default_role: viewer
      groups:
        - { group: scada_admins@idm.example.com, role: admin }   # Kanidm sends name@domain
```

Kanidm signs ID tokens with ES256 by default, which Voltrus accepts. A provider on a
private CA needs `sso.oidc.ca_cert_file` ([Private CA](#idps-behind-a-private-ca)).
A public client (no secret) works too: leave `client_secret` empty and Voltrus
relies on PKCE alone.

## Active Directory over LDAP

Passwords are checked by binding to the directory as the user; nothing is compared
locally. Use LDAPS or StartTLS: plain `ldap://` is refused unless you set
`allow_plain_ldap: true`.

1. Create a service account with read access, e.g. `CN=svc-voltrus,OU=Service,DC=corp,DC=example`.
2. Make sure the domain controller's certificate has the host name you will use in a
   **subjectAltName** (AD CS "Domain Controller Authentication" or "Kerberos
   Authentication" templates do). Export the issuing CA as PEM to
   `/etc/voltrus/corp-ca.pem`.
3. Create groups such as `SCADA Admins` and `SCADA Operators` (nesting is fine with
   `nested_groups: true`).

```yaml
sso:
  public_url: https://scada.example.com
  ldap:
    enabled: true
    label: Corp AD                       # heading of the form on the login page
    url: ldaps://dc1.corp.example:636    # or ldap://dc1.corp.example:389 with starttls: true
    ca_cert_file: /etc/voltrus/corp-ca.pem
    bind_dn: CN=svc-voltrus,OU=Service,DC=corp,DC=example
    bind_password_env: VOLTRUS_LDAP_BIND_PASSWORD
    base_dn: DC=corp,DC=example
    user_filter: (sAMAccountName={username})
    username_attribute: sAMAccountName
    nested_groups: true
    role_mapping:
      default_role: viewer
      groups:
        - { group: "CN=SCADA Admins,OU=Groups,DC=corp,DC=example", role: admin }
        - { group: "SCADA Operators", role: operator }   # a CN works too
```

On the login page people type their AD user name (`alice`, not `CORP\alice`) and
password into the **Corp AD** form, then press **Sign in with Corp AD**.

* **Search mode** (`bind_dn` set): bind as the service account, find the user with
  `user_filter` under `base_dn`, bind as the found DN with the typed password.
* **Direct mode** (`bind_dn_format` instead, e.g. `{username}@corp.example` or
  `uid={username},ou=people,dc=corp,dc=example`): bind as the user, then read
  their entry. No service account needed.
* Groups come from `memberOf` (`member_of_attribute`) plus, with
  `nested_groups: true`, the AD transitive-membership search
  (`1.2.840.113556.1.4.1941`). For OpenLDAP and others set `group_filter`, e.g.
  `(&(objectClass=groupOfNames)(member={user_dn}))`, with `group_base_dn`.
* Usernames are escaped in filters and DNs; empty passwords are refused before any
  bind. Attempts share the rate limits of local login.

## Roles and group mapping

```yaml
role_mapping:
  default_role: viewer      # people matching no group; "" refuses their sign-in instead
  groups:
    - { group: "CN=SCADA Admins,OU=Groups,DC=corp,DC=example", role: admin }
    - { group: "scada-operators", role: operator }
```

* Roles: `admin`, `operator`, `user`, `demo`, `viewer`. When several groups match,
  the highest wins: `admin` > `operator` > `user` > `demo` > `viewer`.
* `group` is a group name, an LDAP group DN or whatever value the IdP puts in the
  groups claim/attribute (Entra: object IDs or app role values). Matching ignores
  case and, for DNs, spaces around `,` and `=`. LDAP groups match by full DN and by CN.
* The role is re-applied at every sign-in. A role changed by hand in
  **More actions (⋯) › Data Manager › Admin › Users** is overwritten at the
  user's next sign-in: change the group at the IdP instead.
* To find the exact group value an IdP sends, use the IdP's token preview
  (Keycloak: client **Client scopes › Evaluate › Generated ID token**; Okta:
  **Security › API › Authorization Servers › Token Preview**; Entra: decode the ID
  token at jwt.ms). For LDAP, read the user's `memberOf` with `ldapsearch`.

**Taking access away.** Remove the person from the IdP groups (with
`default_role: ""` they are then refused) or disable them at the IdP. In Voltrus,
deleting an SSO account does not block it: the next sign-in creates it again. Disable
it instead. The Users page has no disable switch yet; use the API as an admin:

```sh
# find the id
curl -s -b cookies.txt https://scada.example.com/api/v1/users
# disable (sessions stop at once, new sign-ins are refused); "false" re-enables
curl -s -b cookies.txt -X PUT -H 'Content-Type: application/json' \
  -d '{"disabled": true}' https://scada.example.com/api/v1/users/<id>
```

## Signing out

**User menu (top right) › Logout** ends the Voltrus session for every provider. The
IdP session lives on: there is no SAML Single Logout and no OIDC RP-initiated
logout, so the next **Sign in with …** may go straight through without a password
prompt. On shared operator stations, sign out at the IdP too (or set a short IdP
session lifetime).

## When the identity provider is down

* **Local accounts always work.** The password form above the SSO buttons signs in
  `admin` and every local account without contacting any IdP. That is the
  break-glass path; keep one local admin.
* **LDAP** answers "The directory service is unavailable" (502) per attempt while
  the directory is unreachable; the other providers are not affected.
* **SAML** with `idp_metadata_url`: if the IdP is unreachable when Voltrus starts,
  SAML stays configured, **Sign in with …** answers "Single sign-on is unavailable
  right now" (503), and the metadata is fetched again every 60 s and at the next
  click once a minute has passed. Sign-in works again as soon as the IdP is back,
  without a restart. Once loaded, the metadata is refreshed every
  `metadata_refresh_hours` (12) and the previous keys are kept if a refresh fails.
* **OIDC**: discovery and keys are fetched when needed; while the provider is
  unreachable **Sign in with …** shows "Single sign-on is unavailable right now"
  (or the browser cannot load the IdP page), and the next attempt tries again.

## Configuration reference

All keys below live under `sso:` except the three top-level `oidc_*` keys.

### Top level and `sso`

| Key | Default | Meaning |
|-----|---------|---------|
| `oidc_issuer` (top level) | | OIDC issuer URL; browser sign-in and bearer tokens share it. https, or http on loopback only |
| `oidc_enabled` (top level) | `false` | Accept `Authorization: Bearer` ID/access tokens from the issuer on API routes |
| `oidc_audience`, `oidc_jwks_url` (top level) | | Bearer validation: expected `aud`, JWKS URL |
| `oidc_role_claim`, `oidc_admin_role` (top level) | `realm_access.roles`, `voltrus-admin` | Bearer tokens: a token whose `realm_access.roles` or `groups` contains `oidc_admin_role` is `admin`, any other valid token is `user` |
| `sso.public_url` | | Address browsers use, no trailing slash |

### `sso.oidc`

| Key | Default | Meaning |
|-----|---------|---------|
| `client_id` | | Setting it turns OIDC browser sign-in on |
| `label` | `OpenID Connect` | Button text: "Sign in with &lt;label&gt;" |
| `client_secret` / `client_secret_env` | | Secret, or the variable holding it; empty = public client with PKCE |
| `client_auth` | `post` | `post` (secret in the body) or `basic` (Authorization header) |
| `redirect_uri` | `<public_url>/api/v1/auth/oidc/callback` | Register exactly this at the IdP |
| `scopes` | `[openid, profile, email]` | Add `groups` if your IdP needs a scope for it |
| `username_claim` | `sub` | Claim used for the user name (`preferred_username`, `email`) |
| `groups_claim` | `groups` | Claim with groups or roles; dotted paths work (`realm_access.roles`) |
| `ca_cert` / `ca_cert_file` | | Extra CA (PEM) for a provider on a private CA |
| `role_mapping` | | See [Roles](#roles-and-group-mapping) |

### `sso.saml`

| Key | Default | Meaning |
|-----|---------|---------|
| `enabled` | `false` | Turn SAML on |
| `label` | `SAML 2.0` | Button text |
| `idp_metadata_url` | | https URL of the IdP metadata (refetched, survives an IdP outage at boot) |
| `idp_metadata_file` / `idp_metadata_xml` | | Metadata from a file / inline instead |
| `idp_entity_id`, `idp_sso_url`, `idp_certificate` (`idp_certificates`) | | Explicit IdP trust instead of metadata |
| `metadata_refresh_hours` | `12` | Refresh period of `idp_metadata_url` |
| `ca_cert` / `ca_cert_file` | | Extra CA for `idp_metadata_url` |
| `sp_entity_id`, `acs_url` | from `public_url` | Override the SP values |
| `username_attribute` | NameID | Attribute used as the user name |
| `groups_attribute`, `email_attribute`, `display_name_attribute` | | Attribute names |
| `name_id_format` | unspecified | NameID format requested |
| `allow_idp_initiated` | `false` | Accept responses without `InResponseTo` (IdP-initiated) |
| `allow_sha1` | `false` | Accept RSA-SHA1 signatures |
| `clock_skew_secs` | `60` | Tolerance on `NotBefore`/`NotOnOrAfter` |
| `sign_authn_requests` + `sp_private_key`/`_file`/`_env`, `sp_certificate`/`_file` | `false` | Sign the AuthnRequest (RSA-SHA256) and publish the certificate in SP metadata |

What every SAML response must pass: a valid XML signature on the Response or the
Assertion from a pinned IdP certificate (exclusive C14N, RSA-SHA256/384/512);
claims read only from the signed element; no DTD, duplicate IDs, extra or encrypted
assertions; Issuer = IdP; Destination and Recipient = our ACS; our entity ID in the
AudienceRestriction; time conditions; `InResponseTo` naming a login this server
started in the same browser (10 minutes, once); each assertion ID accepted once.
Not supported: encrypted assertions, ECDSA signatures, SAML logout, artifact binding.

### `sso.ldap`

| Key | Default | Meaning |
|-----|---------|---------|
| `enabled` | `false` | Turn LDAP on |
| `label` | `LDAP` | Heading and button of the login form |
| `url` | | `ldaps://host:636` or `ldap://host:389` |
| `starttls` | `false` | StartTLS on an `ldap://` URL |
| `allow_plain_ldap` | `false` | Allow unencrypted `ldap://` (passwords in clear text) |
| `ca_cert_file` / `ca_cert` | system roots | Trust only this CA |
| `timeout_secs` | `10` | Network timeout |
| `bind_dn`, `bind_password` / `bind_password_env` | | Service account (search mode) |
| `bind_dn_format` | | Direct mode, e.g. `{username}@corp.example` |
| `base_dn`, `user_filter` | `(uid={username})` | Where and how users are searched |
| `username_attribute`, `email_attribute`, `display_name_attribute` | | Attribute names |
| `member_of_attribute` | `memberOf` | Group attribute on the user entry |
| `nested_groups` | `false` | AD transitive group search |
| `group_filter`, `group_base_dn` | | Group search for non-AD directories |

### IdPs behind a private CA

`idp_metadata_url`, OIDC discovery, the token endpoint and the JWKS are fetched over
HTTPS against the built-in public roots. For a Keycloak, ADFS or other IdP whose
certificate comes from a private CA, add that CA: `sso.saml.ca_cert_file` (or inline
`ca_cert`) for SAML metadata, `sso.oidc.ca_cert_file` / `ca_cert` for OIDC (browser
sign-in and bearer keys). Host names are still verified. `SSL_CERT_FILE` and the
system store are not consulted. `idp_metadata_url: http://...` is refused, as is a
non-loopback http `oidc_issuer`.

## Troubleshooting

| Symptom | Cause and fix |
|---------|---------------|
| No SSO buttons on the login page | Not licensed (`/api/license/status` must show `"sso": true`), provider not configured, or disabled by a config error: look for `... sign-in disabled:` in the log after a restart. `GET /api/v1/auth/providers` shows `enabled` per provider. Edited `config.yaml` but did not restart. |
| "Single sign-on requires an Enterprise licence." | The licence is Integrator, expired past grace, or missing. See [licensing.md](licensing.md). |
| OIDC: IdP says "redirect_uri mismatch" / "Invalid parameter: redirect_uri" | The redirect URI registered at the IdP must equal `<public_url>/api/v1/auth/oidc/callback` exactly (scheme, host, port, no trailing slash). |
| OIDC: "OIDC discovery issuer does not match oidc_issuer", or `OIDC login refused: ...` in the log | `oidc_issuer` must equal the issuer the provider publishes and puts in `iss`, character for character (Entra: the `/v2.0` issuer; Okta custom server: `/oauth2/<id>`; no trailing slash unless the provider uses one). |
| OIDC: "sign-in was not started by this browser" / "sign-in took too long" | The state cookie was lost (different host name between start and callback, or blocked cookies) or the IdP page sat open more than 10 minutes. Start again from the Voltrus button. |
| Signed in but got `viewer`, or "Your account has no access to Voltrus" with `default_role: ""` | The groups value did not match. Check what the IdP sends (Okta groups claim filter, Entra object IDs vs names, Keycloak full path `/a/b` vs `b`) and copy it into `role_mapping`. |
| Keycloak SAML: "Invalid requester" | Client **Keys › Client signature required** is on: turn it off, or sign requests. |
| SAML: "response was not requested by this browser" | The binding cookie was dropped. Serve Voltrus over HTTPS (the cookie is `SameSite=None; Secure`); on plain http use the same host name for every step. |
| SAML: "unsolicited responses are not accepted" | The login started at the IdP (app tile / My Apps). Start it from the Voltrus button, or set `allow_idp_initiated: true`. |
| SAML: "encrypted assertions are not supported" | Turn assertion encryption off at the IdP. |
| SAML: "Single sign-on is unavailable right now" | The IdP metadata has not loaded (IdP unreachable since startup). It retries every minute; check `SAML IdP metadata ...` lines in the log and the network path to `idp_metadata_url`. |
| SAML "signature" errors after the IdP rolled its certificate | Voltrus refetches `idp_metadata_url` once on an unknown key (at most every 5 minutes). With `idp_metadata_file` or explicit certificates, update them and restart. |
| LDAP: "Invalid credentials" for a correct password | Wrong `user_filter`/`base_dn` (search mode finds nobody), the account is disabled or locked in AD, or the password expired. |
| LDAP: TLS / certificate errors | The DC certificate has no subjectAltName for the host in `url`, or `ca_cert_file` is not the issuing CA. Test: `openssl s_client -connect dc1.corp.example:636 -CAfile /etc/voltrus/corp-ca.pem`. |
| Log: "LDAP sign-in disabled: ldap.url is cleartext ldap://..." | Use `ldaps://` or `starttls: true`; only set `allow_plain_ldap: true` on an isolated test network. |
| "This account is disabled." | Re-enable it with `PUT /api/v1/users/<id>` `{"disabled": false}` (see [Roles](#roles-and-group-mapping)). |
| Logged out but the next SSO sign-in skips the password | Expected: the IdP session is still active ([Signing out](#signing-out)). |
| The native console shows "Signs in through the browser" | Expected: SSO is web only; sign in on the web client, or give the console user a local account. |

## Testing against real servers

The automated tests cover signature verification against documents signed by
openssl/libxml2, the LDAP client against an in-process LDAP server with TLS and
StartTLS, and the OIDC and SAML clients against stubs. `scripts/sso-live/` runs the
real thing:

```sh
scripts/sso-live/run.sh up     # Keycloak, Samba AD DC, Caddy, four Voltrus instances
scripts/sso-live/run.sh test   # HTTP-level suite (python3, stdlib only), ~30 s
scripts/sso-live/run.sh down   # remove containers, volumes, images and ./.run
```

`HTTPS_PORT=8445` (for `up` and `test`) moves the TLS instance off 8443 when that
port is taken. `KEYCLOAK_TAG=26.8.2` pins the Keycloak image. Note that `down`
deletes the images it used (`--rmi all`), including a Keycloak image you already had.

Needs docker with compose, python3, openssl, curl and a Rust toolchain; the first
`up` builds `voltrus-ha-test-node`, the real Enterprise add-on with a stub Enterprise
licence (feature `ha-test-node`, never in a release build). `BIN=` uses a prebuilt one.
`STATIC_DIR=` (default `core/static` when built) serves the web UI for manual browser
runs.

What `up` creates:

* **Keycloak** (dev mode, `127.0.0.1:8081`), realm `voltrus`: users `alice` (group
  `scada-admins`), `bob` (child group `scada-operators/night-shift`), `carol` (no
  group), `dave` (disabled); a confidential OIDC client `voltrus` with a `groups`
  claim; one SAML client per Voltrus instance, created by posting
  `/api/v1/auth/saml/metadata` to Keycloak's client importer.
* **Samba AD DC** (`ldaps://dc1.localhost:636`, realm `corp.example`): users alice,
  bob, carol, dave (disabled) and `svc-voltrus`; groups `SCADA Admins`,
  `SCADA Operators` and, nested inside it, `SCADA Night Shift` (bob's only group). Its
  autogenerated certificate has no subjectAltName, so the script replaces it with one
  from a throwaway CA (`.run/ldap-ca.pem`). Do the same on a real DC.
* **Voltrus instances**: `http` (`http://localhost:3100`), `https`
  (`https://scada.localhost:8443` through Caddy, so `cookie_secure: auto` turns on and
  the SAML binding cookie is `SameSite=None; Secure`), `strict` (`default_role: ""`)
  and `tlsidp` (Keycloak reached as `https://kc.localhost:8444` through Caddy, with
  `idp_metadata_url` and OIDC discovery trusting Caddy's CA through `ca_cert_file`).

The suite signs in through SAML, OIDC and LDAP as every user, checks the role, the
refusals (disabled user, wrong or empty password, unmapped user with
`default_role: ""`, a Voltrus-disabled account), logout and signing in again, a role
edited by hand being re-applied, and SAML replay and IdP-initiated responses being
refused.

OpenLDAP by hand (a test directory with one user):

```sh
docker run -d --name ldap -p 1389:389 -p 1636:636 \
  -e LDAP_ADMIN_PASSWORD=admin -e LDAP_USERS=alice -e LDAP_PASSWORDS=alice-pw \
  -e LDAP_ROOT=dc=corp,dc=example bitnami/openldap
# sso.ldap: url: ldap://localhost:1389, allow_plain_ldap: true, bind_dn: cn=admin,dc=corp,dc=example,
# bind_password: admin, base_dn: ou=users,dc=corp,dc=example, user_filter: (uid={username})
```

Regenerating the SAML signature fixtures (`enterprise/src/sso/saml/fixtures/generate.sh`)
needs `openssl` and `xmllint`.
