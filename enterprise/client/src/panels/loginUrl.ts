// Pure helper for the SSO redirect buttons; kept free of React so node tests can import it.

/** URL that starts a browser sign-in (OIDC or SAML), asking the server to return to `returnTo`. */
export function providerLoginUrl(provider: string, returnTo: string): string {
  const base = `/api/v1/auth/${encodeURIComponent(provider)}/login`
  // Only same-site paths are sent; the server applies the same rule again.
  const safe = returnTo.startsWith('/') && !returnTo.startsWith('//') && !returnTo.includes('\\')
  return safe && returnTo !== '/' ? `${base}?return_to=${encodeURIComponent(returnTo)}` : base
}
