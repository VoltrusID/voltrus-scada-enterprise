import { useEffect, useMemo, useRef, useState } from 'react'
import { btnStyle, inputStyle, labelStyle, labelTextStyle } from '@voltrus/core/styles'
import { useLicense } from '@voltrus/core/LicenseContext'
import { hasFeature } from '@voltrus/core/edition/license'
import { DEFAULT_BRANDING } from '@voltrus/core/edition/branding'
import { useBranding, useDeleteBrandFont, useSaveBranding, useUploadBrandFont, type EnterpriseBranding } from '../queries/branding'
import { MAX_LOGO_FILE_BYTES } from './brandingPayload'
import {
  contrastChecks, DEFAULTS, FONT_STACKS, MAX_FONT_FILE_BYTES, normalizeColour, themeCss, themeProblem,
  type Mode, type Palette, type Theme,
} from './themeModel'

const MODES: Mode[] = ['dark', 'light']
const COLOUR_FIELDS: { key: keyof Palette; label: string }[] = [
  { key: 'background', label: 'Background' },
  { key: 'surface', label: 'Surface (cards, dialogs)' },
  { key: 'text', label: 'Text' },
  { key: 'accent', label: 'Accent (this theme)' },
]

interface Form {
  title: string
  subtitle: string
  logo: string
  accent: string
  theme: Theme
}

function fromBranding(b: EnterpriseBranding | null | undefined): Form {
  return {
    title: b?.title ?? DEFAULT_BRANDING.title,
    subtitle: b?.subtitle ?? DEFAULT_BRANDING.subtitle,
    logo: b?.logo ?? '',
    accent: b?.accent ?? '',
    theme: { dark: { ...b?.theme?.dark }, light: { ...b?.theme?.light }, font: b?.theme?.font ?? '' },
  }
}

function ColourInput({ label, value, fallback, onChange }: { label: string; value: string; fallback: string; onChange: (v: string) => void }) {
  const valid = normalizeColour(value) !== null
  return (
    <div style={labelStyle}>
      <span style={labelTextStyle}>{label}</span>
      <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
        <input type="color" aria-label={`${label} picker`} value={normalizeColour(value) || fallback} onChange={e => onChange(e.target.value)}
          style={{ width: 34, height: 28, padding: 0, border: '1px solid var(--border)', borderRadius: 6, background: 'transparent', cursor: 'pointer' }} />
        <input aria-label={label} value={value} placeholder={fallback} onChange={e => onChange(e.target.value.trim())}
          style={{ ...inputStyle, flex: 1, fontFamily: 'var(--font-mono)', borderColor: valid ? undefined : 'var(--crit)' }} />
        <button type="button" onClick={() => onChange('')} style={{ fontSize: 11, color: 'var(--muted)', background: 'none', border: 'none', cursor: 'pointer' }}>Default</button>
      </div>
    </div>
  )
}

/** A miniature of the app in one mode, painted with the form's theme. */
function Preview({ form, mode, fontUrl }: { form: Form; mode: Mode; fontUrl?: string | null }) {
  return (
    <div className="voltrus-brand-preview" data-theme={mode} style={{
      background: 'var(--bg)', color: 'var(--fg)', fontFamily: 'var(--font-ui)', border: '1px solid var(--border)',
      borderRadius: 10, overflow: 'hidden', fontSize: 13,
    }} aria-label={`${mode} theme preview`}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '8px 12px', borderBottom: '1px solid var(--border)', background: 'var(--card)' }}>
        <img src={form.logo || '/favicon-32.png'} alt="" style={{ width: 20, height: 20, borderRadius: 4, objectFit: 'contain' }} />
        <strong>{form.title || DEFAULT_BRANDING.title}</strong>
        <span style={{ color: 'var(--muted)', fontSize: 11 }}>{form.subtitle}</span>
        <span style={{ marginLeft: 'auto', fontSize: 11, color: 'var(--cyan)', borderBottom: '2px solid var(--cyan)' }}>Dashboard</span>
      </div>
      <div style={{ padding: 12, display: 'flex', flexDirection: 'column', gap: 8 }}>
        <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 8, padding: 10 }}>
          <div style={{ fontWeight: 600 }}>Tank 1 level</div>
          <div style={{ color: 'var(--text-dim)' }}>72.4 % <span style={{ color: 'var(--muted)', fontSize: 11 }}>updated 2 s ago</span></div>
          <a href="#preview" onClick={e => e.preventDefault()} style={{ color: 'var(--cyan)', fontSize: 12 }}>Open trend</a>
        </div>
        <div style={{ display: 'flex', gap: 8 }}>
          <input readOnly value="Search tags" aria-label="Preview input" style={{ ...inputStyle, flex: 1, background: 'var(--input-bg)', color: 'var(--muted)' }} />
          <button type="button" style={{ ...btnStyle, borderColor: 'var(--cyan)', color: 'var(--cyan)' }}>Save</button>
        </div>
        {form.theme.font === 'custom' && !fontUrl && <div style={{ fontSize: 11, color: 'var(--warn)' }}>Upload a font file to use it.</div>}
      </div>
    </div>
  )
}

function ContrastList({ form, mode }: { form: Form; mode: Mode }) {
  return (
    <ul style={{ listStyle: 'none', margin: 0, padding: 0, fontSize: 11, display: 'grid', gap: 2 }}>
      {contrastChecks(form.theme, mode, normalizeColour(form.accent) ?? '').map(c => (
        <li key={c.what} style={{ color: c.ok ? 'var(--muted)' : 'var(--crit)' }}>
          {c.ok ? '✓' : '✗'} {c.what}: {c.ratio.toFixed(2)}:1 (at least {c.min}:1)
        </li>
      ))}
    </ul>
  )
}

/** Data Manager › Admin › Branding: title, logo, colours of both themes and the font, with a live preview. */
export function BrandingPanel() {
  const license = useLicense()
  const branding = useBranding(license) as EnterpriseBranding | null
  const logoOnly = !hasFeature(license, 'theme_full')
  const save = useSaveBranding()
  const uploadFont = useUploadBrandFont()
  const deleteFont = useDeleteBrandFont()
  const [form, setForm] = useState<Form>(() => fromBranding(branding))
  const [loadedFrom, setLoadedFrom] = useState<EnterpriseBranding | null>(branding)
  const [message, setMessage] = useState<{ ok: boolean; text: string } | null>(null)
  const logoRef = useRef<HTMLInputElement>(null)
  const fontRef = useRef<HTMLInputElement>(null)

  // Take the server's values once they arrive (and after a save reloads them).
  useEffect(() => {
    if (branding && branding !== loadedFrom) {
      setForm(fromBranding(branding))
      setLoadedFrom(branding)
    }
  }, [branding, loadedFrom])

  const problem = logoOnly ? null : themeProblem(form.theme, form.accent)
  const previewCss = useMemo(
    () => themeCss({ accent: form.accent, theme: form.theme, font_url: branding?.font_url }, '.voltrus-brand-preview', true),
    [form.accent, form.theme, branding?.font_url],
  )
  const setPalette = (mode: Mode, key: keyof Palette, value: string) =>
    setForm(f => ({ ...f, theme: { ...f.theme, [mode]: { ...f.theme[mode], [key]: value } } }))

  const readFile = (file: File, max: number, then: (dataUrl: string) => void) => {
    if (file.size > max) {
      setMessage({ ok: false, text: `File too large (max ${Math.floor(max / 1024)} KB).` })
      return
    }
    const reader = new FileReader()
    reader.onload = () => { if (typeof reader.result === 'string') then(reader.result) }
    reader.readAsDataURL(file)
  }

  const onSave = () => {
    setMessage(null)
    const body: Partial<EnterpriseBranding> = logoOnly
      ? { logo: form.logo }
      : { title: form.title, subtitle: form.subtitle, logo: form.logo, accent: form.accent, theme: form.theme }
    save.mutate(body, {
      onSuccess: () => setMessage({ ok: true, text: 'Branding saved. Everyone sees it at their next page load.' }),
      onError: e => setMessage({ ok: false, text: e instanceof Error ? e.message : String(e) }),
    })
  }

  return (
    <div style={{ padding: 24, display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))', gap: 24, alignItems: 'start' }}>
      <style>{previewCss}</style>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
        <h2 style={{ fontSize: 16, fontWeight: 700, margin: 0 }}>Branding</h2>
        {logoOnly && (
          <p style={{ margin: 0, fontSize: 12, color: 'var(--muted)' }}>
            Your licence includes a custom logo. The title, colours and font need the Integrator licence.
          </p>
        )}
        {!logoOnly && (
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
          <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
            <input aria-label="Logo URL" value={form.logo.startsWith('data:') ? '' : form.logo} onChange={e => setForm(f => ({ ...f, logo: e.target.value }))}
              placeholder={form.logo.startsWith('data:') ? 'Uploaded image' : 'https://example.com/logo.svg'} style={{ ...inputStyle, flex: 1 }} />
            <input ref={logoRef} type="file" accept="image/png,image/svg+xml,image/jpeg,image/webp" style={{ display: 'none' }}
              onChange={e => { const f = e.target.files?.[0]; if (f) readFile(f, MAX_LOGO_FILE_BYTES, logo => setForm(x => ({ ...x, logo }))) }} />
            <button type="button" style={btnStyle} onClick={() => logoRef.current?.click()}>Upload</button>
            {form.logo && <button type="button" style={{ ...btnStyle, color: 'var(--crit)' }} onClick={() => setForm(f => ({ ...f, logo: '' }))}>Remove</button>}
          </div>
        </div>

        {!logoOnly && (
          <>
            <ColourInput label="Accent colour (both themes)" value={form.accent} fallback={DEFAULTS.dark.accent} onChange={accent => setForm(f => ({ ...f, accent }))} />
            {MODES.map(mode => (
              <fieldset key={mode} style={{ border: '1px solid var(--border)', borderRadius: 8, padding: 12, display: 'grid', gap: 10 }}>
                <legend style={{ fontSize: 12, fontWeight: 600, padding: '0 4px' }}>{mode === 'dark' ? 'Dark theme' : 'Light theme'}</legend>
                {COLOUR_FIELDS.map(({ key, label }) => (
                  <ColourInput key={key} label={label} value={form.theme[mode]?.[key] ?? ''}
                    fallback={key === 'accent' ? (normalizeColour(form.accent) || DEFAULTS[mode].accent) : DEFAULTS[mode][key]}
                    onChange={v => setPalette(mode, key, v)} />
                ))}
                <ContrastList form={form} mode={mode} />
              </fieldset>
            ))}
            <div style={labelStyle}>
              <span style={labelTextStyle}>Font</span>
              <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                <select aria-label="Font" value={form.theme.font || 'system'} onChange={e => setForm(f => ({ ...f, theme: { ...f.theme, font: e.target.value === 'system' ? '' : e.target.value } }))}
                  style={{ ...inputStyle, flex: 1 }}>
                  {Object.entries(FONT_STACKS).map(([id, { label }]) => (
                    <option key={id} value={id} disabled={id === 'custom' && !branding?.font_name}>
                      {id === 'custom' && branding?.font_name ? `Uploaded: ${branding.font_name}` : label}
                    </option>
                  ))}
                </select>
                <input ref={fontRef} type="file" accept=".woff2,.woff,.ttf,.otf,font/woff2,font/woff,font/ttf,font/otf" style={{ display: 'none' }}
                  onChange={e => {
                    const file = e.target.files?.[0]
                    if (!file) return
                    readFile(file, MAX_FONT_FILE_BYTES, data => uploadFont.mutate({ name: file.name.replace(/\.[^.]+$/, ''), data }, {
                      onSuccess: () => { setForm(f => ({ ...f, theme: { ...f.theme, font: 'custom' } })); setMessage({ ok: true, text: 'Font uploaded and selected.' }) },
                      onError: err => setMessage({ ok: false, text: err instanceof Error ? err.message : String(err) }),
                    }))
                  }} />
                <button type="button" style={btnStyle} onClick={() => fontRef.current?.click()}>Upload font</button>
                {branding?.font_name && <button type="button" style={{ ...btnStyle, color: 'var(--crit)' }} onClick={() => deleteFont.mutate()}>Remove</button>}
              </div>
              <span style={{ fontSize: 11, color: 'var(--muted)' }}>System fonts load nothing from the internet. An uploaded WOFF2, WOFF, TTF or OTF file (at most 256 KB) is served by this server.</span>
            </div>
          </>
        )}

        {problem && <div role="alert" style={{ color: 'var(--crit)', fontSize: 12 }}>{problem}</div>}
        {message && <div role="status" style={{ color: message.ok ? 'var(--ok)' : 'var(--crit)', fontSize: 12 }}>{message.text}</div>}
        <div style={{ display: 'flex', gap: 8 }}>
          <button type="button" disabled={!!problem || save.isPending} onClick={onSave}
            style={{ ...btnStyle, borderColor: 'var(--cyan)', color: 'var(--cyan)', fontWeight: 600, opacity: problem ? 0.5 : 1 }}>
            {save.isPending ? 'Saving…' : 'Save'}
          </button>
          <button type="button" style={btnStyle} onClick={() => { setForm(fromBranding(branding)); setMessage(null) }}>Discard changes</button>
        </div>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 12, position: 'sticky', top: 0 }}>
        <span style={labelTextStyle}>Live preview</span>
        {MODES.map(mode => <Preview key={mode} form={form} mode={mode} fontUrl={branding?.font_url} />)}
      </div>
    </div>
  )
}
