import { useEffect, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { fetchWithRefresh } from '@voltrus/core/queries/fetch'
import type { ScreenWidgetConfigEditorProps, ScreenWidgetRenderProps } from '@voltrus/core/edition/widgets'
import { inputStyle, labelStyle, labelTextStyle } from '@voltrus/core/styles'
import { type AvailableCamera, cameraConfigOf, cameraView } from './cameraWidgetModel'

class HttpError extends Error {
  constructor(readonly status: number) {
    super(`HTTP ${status}`)
  }
}

/** Cameras an operator may watch (id, name, on/off). */
function useAvailableCameras() {
  return useQuery<AvailableCamera[], HttpError>({
    queryKey: ['cameras', 'available'],
    queryFn: async () => {
      const r = await fetchWithRefresh('/api/v1/cameras/available')
      if (!r.ok) throw new HttpError(r.status)
      return r.json()
    },
    staleTime: 30_000,
    retry: (count, err) => !(err instanceof HttpError && err.status < 500) && count < 2,
  })
}

export function CameraScreenWidget({ config, editMode }: ScreenWidgetRenderProps) {
  const cfg = cameraConfigOf(config)
  const { data, isLoading, error } = useAvailableCameras()
  const [streamFailed, setStreamFailed] = useState(false)
  useEffect(() => setStreamFailed(false), [cfg.camera_id])
  const view = cameraView({
    config: cfg,
    cameras: data,
    loading: isLoading,
    status: error instanceof HttpError ? error.status : error ? 500 : undefined,
    editMode,
    streamFailed,
  })

  const frame: React.CSSProperties = {
    position: 'relative', width: '100%', height: '100%', background: '#0a0a12',
    display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden', borderRadius: 6,
  }
  const caption = (name: string) => cfg.show_name && (
    <span style={{
      position: 'absolute', left: 8, top: 8, padding: '2px 8px', borderRadius: 4,
      background: 'rgba(0,0,0,.6)', color: '#fff', fontSize: 12, fontWeight: 600,
    }}>{name}</span>
  )

  if (view.kind === 'message') {
    return (
      <div style={frame} data-testid="camera-widget">
        <span role="status" style={{ color: view.tone === 'warn' ? 'var(--warn)' : 'var(--muted)', fontSize: 12, padding: 12, textAlign: 'center' }}>
          {view.text}
        </span>
      </div>
    )
  }
  if (view.kind === 'placeholder') {
    return (
      <div style={frame} data-testid="camera-widget">
        {caption(view.name)}
        <span style={{ color: 'var(--muted)', fontSize: 12 }}>Live view starts when you leave edit mode</span>
      </div>
    )
  }
  return (
    <div style={frame} data-testid="camera-widget">
      <img
        src={view.src}
        alt={`Live view of ${view.name}`}
        onError={() => setStreamFailed(true)}
        style={{ width: '100%', height: '100%', objectFit: 'contain', display: 'block' }}
      />
      {caption(view.name)}
      <span aria-hidden style={{
        position: 'absolute', right: 8, top: 8, padding: '1px 6px', borderRadius: 3,
        background: 'rgba(220,38,38,.85)', color: '#fff', fontSize: 10, fontWeight: 700, letterSpacing: '.05em',
      }}>LIVE</span>
    </div>
  )
}

export function CameraWidgetSettings({ config, onChange }: ScreenWidgetConfigEditorProps) {
  const cfg = cameraConfigOf(config)
  const { data = [], isLoading, error } = useAvailableCameras()
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <label style={labelStyle}>
        <span style={labelTextStyle}>Camera</span>
        <select
          value={cfg.camera_id ?? ''}
          onChange={e => onChange({ ...config, camera_id: e.target.value ? Number(e.target.value) : null })}
          style={inputStyle}
          disabled={isLoading || !!error}
        >
          <option value="">{isLoading ? 'Loading…' : error ? 'Cameras could not be loaded' : 'Choose a camera'}</option>
          {data.map(c => (
            <option key={c.id} value={c.id}>{c.name}{c.enabled ? '' : ' (off)'}</option>
          ))}
        </select>
      </label>
      {!isLoading && !error && data.length === 0 && (
        <span style={{ fontSize: 12, color: 'var(--muted)' }}>No cameras yet: an administrator adds them under Data › Cameras.</span>
      )}
      <label style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 13 }}>
        <input type="checkbox" checked={cfg.show_name} onChange={e => onChange({ ...config, show_name: e.target.checked })} />
        Show the camera name
      </label>
    </div>
  )
}
