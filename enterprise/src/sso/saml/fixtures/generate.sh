#!/usr/bin/env bash
# Regenerates the SAML test fixtures.
#
# The signatures are produced with openssl and libxml2's `xmllint --exc-c14n`,
# not with the Rust code under test, so the tests check our canonicalization
# and RSA verification against an independent implementation. A throwaway IdP
# key is generated per run and discarded: only the public certificates and the
# signed documents are kept. Rerunning changes every signature; the tests do
# not depend on the old values.
#
#   ./generate.sh        (needs openssl and xmllint)
set -euo pipefail
cd "$(dirname "$0")"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

NS_DS=http://www.w3.org/2000/09/xmldsig#
NS_SAML=urn:oasis:names:tc:SAML:2.0:assertion
NS_SAMLP=urn:oasis:names:tc:SAML:2.0:protocol

for name in idp other; do
  openssl req -x509 -newkey rsa:2048 -nodes -keyout "$tmp/$name.key" -out "$tmp/$name.crt" \
    -subj "/CN=$name.example.com" -days 36500 -sha256 2>/dev/null
  cp "$tmp/$name.crt" "$name.pem"
done
CERT=$(grep -v -- ----- "$tmp/idp.crt" | tr -d '\n')

# Defaults, overridden per variant through the environment.
: "${ISSUER:=https://idp.example.com/metadata}"
: "${AUDIENCE:=https://scada.example.com/api/v1/auth/saml/metadata}"
: "${ACS:=https://scada.example.com/api/v1/auth/saml/acs}"
: "${DESTINATION:=$ACS}"
: "${RECIPIENT:=$ACS}"
: "${IRT:=_req1}"
: "${NAME_ID:=alice@example.com}"

c14n() { xmllint --exc-c14n "$1" | tr -d '\n' > "$2"; }
b64() { base64 -w0 "$1"; }

# sign_element <xml-with-@SIG@> <id> <sig-method-uri> <digest-method-uri> <openssl-digest>
# Prints the element with a finished ds:Signature in place of @SIG@.
sign_element() {
  local xml=$1 id=$2 sigm=$3 digm=$4 dgst=$5
  printf '%s' "${xml//@SIG@/}" > "$tmp/unsigned.xml"
  c14n "$tmp/unsigned.xml" "$tmp/unsigned.c14n"
  local digest; digest=$(openssl dgst "-$dgst" -binary "$tmp/unsigned.c14n" | base64 -w0)
  local signed_info="<ds:SignedInfo xmlns:ds=\"$NS_DS\"><ds:CanonicalizationMethod Algorithm=\"http://www.w3.org/2001/10/xml-exc-c14n#\"></ds:CanonicalizationMethod><ds:SignatureMethod Algorithm=\"$sigm\"></ds:SignatureMethod><ds:Reference URI=\"#$id\"><ds:Transforms><ds:Transform Algorithm=\"$NS_DS""enveloped-signature\"></ds:Transform><ds:Transform Algorithm=\"http://www.w3.org/2001/10/xml-exc-c14n#\"></ds:Transform></ds:Transforms><ds:DigestMethod Algorithm=\"$digm\"></ds:DigestMethod><ds:DigestValue>$digest</ds:DigestValue></ds:Reference></ds:SignedInfo>"
  printf '%s' "$signed_info" > "$tmp/si.xml"
  c14n "$tmp/si.xml" "$tmp/si.c14n"
  openssl dgst "-$dgst" -sign "$tmp/idp.key" -out "$tmp/si.sig" "$tmp/si.c14n"
  local sigv; sigv=$(b64 "$tmp/si.sig")
  # The signature carries SignedInfo as written; the verifier canonicalizes it in place.
  local sig="<ds:Signature xmlns:ds=\"$NS_DS\"><ds:SignedInfo>${signed_info#*>}<ds:SignatureValue>$sigv</ds:SignatureValue><ds:KeyInfo><ds:X509Data><ds:X509Certificate>$CERT</ds:X509Certificate></ds:X509Data></ds:KeyInfo></ds:Signature>"
  printf '%s' "${xml//@SIG@/$sig}"
}

assertion() { # @SIG@ placeholder where the Signature belongs
  local irt=""; [ -n "$IRT" ] && irt=" InResponseTo=\"$IRT\""
  printf '%s' "<saml:Assertion xmlns:saml=\"$NS_SAML\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ID=\"_assertion1\" IssueInstant=\"2030-01-01T00:00:00Z\" Version=\"2.0\"><saml:Issuer>$ISSUER</saml:Issuer>@SIG@<saml:Subject><saml:NameID Format=\"urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress\">$NAME_ID</saml:NameID><saml:SubjectConfirmation Method=\"urn:oasis:names:tc:SAML:2.0:cm:bearer\"><saml:SubjectConfirmationData$irt NotOnOrAfter=\"2030-01-01T00:05:00Z\" Recipient=\"$RECIPIENT\"></saml:SubjectConfirmationData></saml:SubjectConfirmation></saml:Subject><saml:Conditions NotBefore=\"2029-12-31T23:55:00Z\" NotOnOrAfter=\"2030-01-01T00:05:00Z\"><saml:AudienceRestriction><saml:Audience>$AUDIENCE</saml:Audience></saml:AudienceRestriction></saml:Conditions><saml:AuthnStatement AuthnInstant=\"2030-01-01T00:00:00Z\" SessionIndex=\"_session1\"><saml:AuthnContext><saml:AuthnContextClassRef>urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport</saml:AuthnContextClassRef></saml:AuthnContext></saml:AuthnStatement><saml:AttributeStatement><saml:Attribute Name=\"groups\"><saml:AttributeValue xsi:type=\"xs:string\">scada-admins</saml:AttributeValue><saml:AttributeValue xsi:type=\"xs:string\">operators</saml:AttributeValue></saml:Attribute><saml:Attribute Name=\"http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name\" FriendlyName=\"displayName\"><saml:AttributeValue xsi:type=\"xs:string\">Alice Example</saml:AttributeValue></saml:Attribute></saml:AttributeStatement></saml:Assertion>"
}

response() { # $1 = assertion xml, @RSIG@ placeholder for the response signature
  local irt=""; [ -n "$IRT" ] && irt=" InResponseTo=\"$IRT\""
  printf '%s' "<samlp:Response xmlns:samlp=\"$NS_SAMLP\" xmlns:saml=\"$NS_SAML\" ID=\"_response1\" Version=\"2.0\" IssueInstant=\"2030-01-01T00:00:00Z\" Destination=\"$DESTINATION\"$irt><saml:Issuer>$ISSUER</saml:Issuer>@SIG@<samlp:Status><samlp:StatusCode Value=\"urn:oasis:names:tc:SAML:2.0:status:Success\"></samlp:StatusCode></samlp:Status>$1</samlp:Response>"
}

SHA256=(http://www.w3.org/2001/04/xmldsig-more#rsa-sha256 http://www.w3.org/2001/04/xmlenc#sha256 sha256)
SHA1=(http://www.w3.org/2000/09/xmldsig#rsa-sha1 http://www.w3.org/2000/09/xmldsig#sha1 sha1)

# Build `$1.xml` with signing mode `$2` (assertion|response|both|none).
build() {
  local name=$1 mode=$2; shift 2
  local alg=("${SHA256[@]}"); [ "${ALG:-}" = sha1 ] && alg=("${SHA1[@]}")
  local a; a=$(assertion)
  # Assertion fragment is built with its Signature placeholder; sign when asked.
  if [ "$mode" = assertion ] || [ "$mode" = both ]; then
    a=$(sign_element "$a" _assertion1 "${alg[@]}")
  else
    a=${a//@SIG@/}
  fi
    local r; r=$(response "$a")
  if [ "$mode" = response ] || [ "$mode" = both ]; then
    r=$(sign_element "$r" _response1 "${alg[@]}")
  else
    r=${r//@SIG@/}
  fi
  printf '%s' "$r" > "$name.xml"
}

# --- valid documents -------------------------------------------------------
build signed_assertion assertion
build signed_response response
build signed_both both
build unsigned none

# --- signed, but wrong in one respect --------------------------------------
AUDIENCE=https://other-sp.example.com/metadata build audience_wrong assertion
RECIPIENT=https://evil.example.com/acs build recipient_wrong assertion
ISSUER=https://evil-idp.example.com/metadata build issuer_wrong assertion
DESTINATION=https://evil.example.com/acs build destination_wrong response
IRT=_other-request build irt_other assertion
IRT= build idp_initiated assertion
ALG=sha1 build sha1_assertion assertion

echo "fixtures written: $(ls *.xml | tr '\n' ' ')"
