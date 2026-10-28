import test from 'node:test'
import assert from 'node:assert/strict'
import { providerLoginUrl } from '../src/panels/loginUrl.ts'

test('redirect URL carries a local return path', () => {
  assert.equal(providerLoginUrl('saml', '/screens/3?x=1'), '/api/v1/auth/saml/login?return_to=%2Fscreens%2F3%3Fx%3D1')
})

test('the root path and foreign targets send no return_to', () => {
  assert.equal(providerLoginUrl('oidc', '/'), '/api/v1/auth/oidc/login')
  for (const bad of ['//evil.example', 'https://evil.example', '/\\evil.example']) {
    assert.equal(providerLoginUrl('oidc', bad), '/api/v1/auth/oidc/login', bad)
  }
})
