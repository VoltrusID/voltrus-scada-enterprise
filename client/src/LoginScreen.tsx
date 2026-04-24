import { useState } from 'react';
import { inputStyle, labelStyle, labelTextStyle, btnStyle } from './styles';
import type { BrandingSettings } from './BrandingDialog';

export function LoginScreen({ onLogin, error, branding }: { onLogin: (u: string, p: string) => void; error: string; branding: BrandingSettings }) {
  const [user, setUser] = useState('');
  const [pass, setPass] = useState('');
  const submit = (e: React.FormEvent) => { e.preventDefault(); onLogin(user, pass); };
  return (
    <div style={{ height: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg)' }}>
      <form onSubmit={submit} style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 12, padding: 32, width: 340 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 24 }}>
          <img src={branding.logo || '/favicon-32.png'} alt={branding.title} style={{ width: 36, height: 36, borderRadius: 8, objectFit: 'contain' }} />
          <span style={{ fontSize: 18, fontWeight: 700 }}>{branding.title}</span>
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          <label style={labelStyle}>
            <span style={labelTextStyle}>Username</span>
            <input value={user} onChange={e => setUser(e.target.value)} style={inputStyle} autoComplete="username" autoFocus />
          </label>
          <label style={labelStyle}>
            <span style={labelTextStyle}>Password</span>
            <input type="password" value={pass} onChange={e => setPass(e.target.value)} style={inputStyle} autoComplete="current-password" />
          </label>
          {error && <div style={{ color: 'var(--crit)', fontSize: 12 }}>{error}</div>}
          <button type="submit" style={{ ...btnStyle, justifyContent: 'center', width: '100%', marginTop: 4, background: 'var(--cyan)', color: '#000', borderColor: 'var(--cyan)', fontWeight: 600 }}>Sign In</button>
        </div>
      </form>
    </div>
  );
}
