import { useState, useRef } from 'react';
import { inputStyle, labelStyle, labelTextStyle } from '@voltrus/core/styles';

import { DEFAULT_BRANDING, type BrandingSettings } from '@voltrus/core/edition/branding';
import { brandingPayload } from './brandingPayload';

export function BrandingDialog({ branding, logoOnly = false, onSave, onClose }: {
  branding: BrandingSettings;
  /** Licence allows the logo only (no theme_full): title and subtitle stay the Voltrus defaults. */
  logoOnly?: boolean;
  onSave: (settings: Partial<BrandingSettings>) => void;
  onClose: () => void;
}) {
  const [form, setForm] = useState<BrandingSettings>({ ...branding });
  const [logoTab, setLogoTab] = useState<'url' | 'upload'>(branding.logo && !branding.logo.startsWith('data:') ? 'url' : 'upload');
  const [logoError, setLogoError] = useState('');
  const fileRef = useRef<HTMLInputElement>(null);

  const handleFile = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    if (file.size > 100_000) {
      setLogoError('File too large (max 100KB)');
      return;
    }
    setLogoError('');
    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result === 'string') {
        setForm(f => ({ ...f, logo: reader.result as string }));
      }
    };
    reader.readAsDataURL(file);
  };

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 60 }} onClick={onClose}>
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, width: 420, padding: 20 }} onClick={e => e.stopPropagation()}>
        <div style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>Branding Settings</div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          {logoOnly ? (
            <div style={{ fontSize: 12, color: 'var(--muted)' }}>
              Your licence includes a custom logo. Changing the title and subtitle needs the Integrator licence.
            </div>
          ) : (
            <>
              <label style={labelStyle}>
                <span style={labelTextStyle}>Title</span>
                <input value={form.title} onChange={e => setForm(f => ({ ...f, title: e.target.value }))} placeholder={DEFAULT_BRANDING.title} style={inputStyle} />
              </label>

              <label style={labelStyle}>
                <span style={labelTextStyle}>Subtitle</span>
                <input value={form.subtitle} onChange={e => setForm(f => ({ ...f, subtitle: e.target.value }))} placeholder={DEFAULT_BRANDING.subtitle} style={inputStyle} />
              </label>
            </>
          )}

          <div style={labelStyle}>
            <span style={labelTextStyle}>Logo</span>
            <div style={{ display: 'flex', gap: 0, borderBottom: '1px solid var(--border)', marginBottom: 8 }}>
              {(['url', 'upload'] as const).map(tab => (
                <button key={tab} onClick={() => setLogoTab(tab)} style={{
                  padding: '6px 14px', fontSize: 11, fontWeight: 600, cursor: 'pointer', border: 'none',
                  borderBottom: logoTab === tab ? '2px solid var(--cyan)' : '2px solid transparent',
                  color: logoTab === tab ? 'var(--cyan)' : 'var(--muted)', background: 'transparent',
                  textTransform: 'uppercase' as const, letterSpacing: '.04em',
                }}>{tab === 'url' ? 'URL' : 'Upload'}</button>
              ))}
            </div>

            {logoTab === 'url' ? (
              <div style={{ display: 'flex', gap: 8 }}>
                <input value={form.logo.startsWith('data:') ? '' : form.logo} onChange={e => setForm(f => ({ ...f, logo: e.target.value }))} placeholder="https://example.com/logo.png" style={{ ...inputStyle, flex: 1 }} />
              </div>
            ) : (
              <div>
                <input ref={fileRef} type="file" accept="image/*" onChange={handleFile} style={{ display: 'none' }} />
                <button onClick={() => fileRef.current?.click()} style={{ padding: '6px 14px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg)', color: 'var(--fg)', cursor: 'pointer', fontSize: 13 }}>Choose File</button>
                {logoError && <div style={{ color: 'var(--crit)', fontSize: 11, marginTop: 4 }}>{logoError}</div>}
              </div>
            )}

            {(form.logo || branding.logo) && (
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginTop: 8 }}>
                <img src={form.logo || '/favicon-32.png'} alt="Preview" style={{ width: 32, height: 32, borderRadius: 6, objectFit: 'contain', border: '1px solid var(--border)' }} />
                <span style={{ fontSize: 11, color: 'var(--muted)' }}>Preview</span>
                <button onClick={() => setForm(f => ({ ...f, logo: '' }))} style={{ fontSize: 11, color: 'var(--crit)', background: 'none', border: 'none', cursor: 'pointer' }}>Reset</button>
              </div>
            )}
          </div>
        </div>

        <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 20 }}>
          <button onClick={onClose} style={{ padding: '7px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'transparent', color: 'var(--fg)', cursor: 'pointer', fontSize: 13 }}>Cancel</button>
          <button onClick={() => { onSave(brandingPayload(form, logoOnly)); onClose(); }} style={{ padding: '7px 16px', borderRadius: 6, border: '1px solid var(--cyan)', background: 'transparent', color: 'var(--cyan)', cursor: 'pointer', fontSize: 13, fontWeight: 600 }}>Save</button>
        </div>
      </div>
    </div>
  );
}
