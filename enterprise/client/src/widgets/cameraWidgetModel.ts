// The live camera widget for dashboards and SCADA screens, without React so it
// can be tested under node. CameraScreenWidget.tsx draws what `cameraView`
// decides.
import type { ScreenWidgetDef } from '@voltrus/core/edition/widgets'

export const CAMERA_WIDGET_TYPE = 'enterprise.camera'

/** A camera as `GET /api/v1/cameras/available` lists it (no RTSP address). */
export interface AvailableCamera {
  id: number
  name: string
  enabled: boolean
}

export interface CameraWidgetConfig {
  camera_id: number | null
  /** Show the camera's name over the picture. */
  show_name: boolean
}

export const DEFAULT_CAMERA_CONFIG: CameraWidgetConfig = { camera_id: null, show_name: true }

/** The saved config, tolerant of anything a screen file may hold. */
export function cameraConfigOf(config: Record<string, unknown> | undefined): CameraWidgetConfig {
  const raw = config?.camera_id
  const id = typeof raw === 'number' ? raw : typeof raw === 'string' && /^\d+$/.test(raw) ? Number(raw) : null
  return {
    camera_id: id !== null && Number.isSafeInteger(id) && id > 0 ? id : null,
    show_name: config?.show_name !== false,
  }
}

/** The MJPEG stream the video module serves; the session cookie authorises it. */
export function liveUrl(cameraId: number): string {
  return `/api/v1/cameras/${encodeURIComponent(String(cameraId))}/live`
}

export type CameraView =
  | { kind: 'message'; text: string; tone: 'muted' | 'warn' }
  | { kind: 'live'; src: string; name: string }
  /** Editing the screen: no stream is opened (each view holds one of the server's live slots). */
  | { kind: 'placeholder'; name: string }

/**
 * What the widget shows. `status` is the HTTP status of the camera list when it
 * failed (403 for a role or licence without video), `streamFailed` is set when
 * the picture did not load.
 */
export function cameraView(args: {
  config: CameraWidgetConfig
  cameras: AvailableCamera[] | undefined
  loading: boolean
  status?: number
  editMode: boolean
  streamFailed: boolean
}): CameraView {
  const { config, cameras, loading, status, editMode, streamFailed } = args
  if (status === 401 || status === 403) {
    return { kind: 'message', text: 'Your role cannot view cameras', tone: 'warn' }
  }
  if (status) return { kind: 'message', text: 'Cameras could not be loaded', tone: 'warn' }
  if (config.camera_id === null) {
    return { kind: 'message', text: editMode ? 'Choose a camera in the widget settings' : 'No camera chosen', tone: 'muted' }
  }
  if (loading || !cameras) return { kind: 'message', text: 'Loading camera…', tone: 'muted' }
  const camera = cameras.find(c => c.id === config.camera_id)
  if (!camera) return { kind: 'message', text: 'This camera no longer exists', tone: 'warn' }
  if (!camera.enabled) return { kind: 'message', text: `${camera.name} is turned off`, tone: 'muted' }
  if (editMode) return { kind: 'placeholder', name: camera.name }
  if (streamFailed) {
    return { kind: 'message', text: `${camera.name}: no picture (camera offline, or too many live views open)`, tone: 'warn' }
  }
  return { kind: 'live', src: liveUrl(camera.id), name: camera.name }
}

/** The widget definition, with the React parts passed in (they live in the .tsx file). */
export function cameraWidgetDef(parts: Pick<ScreenWidgetDef, 'render' | 'configEditor' | 'icon'>): ScreenWidgetDef {
  return {
    type: CAMERA_WIDGET_TYPE,
    label: 'Camera',
    surfaces: ['dashboard', 'scada'],
    defaultSize: { w: 8, h: 6 },
    defaultPixelSize: { width: 480, height: 300 },
    defaultConfig: { ...DEFAULT_CAMERA_CONFIG },
    ...parts,
  }
}
