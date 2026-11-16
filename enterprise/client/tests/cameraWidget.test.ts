import assert from 'node:assert/strict'
import { test } from 'node:test'
import {
  CAMERA_WIDGET_TYPE, cameraConfigOf, cameraView, cameraWidgetDef, DEFAULT_CAMERA_CONFIG, liveUrl,
} from '../src/widgets/cameraWidgetModel.ts'
// The core registry the module's `screenWidgets` feed (React-free).
import {
  getScreenWidget, initialConfigOf, registerScreenWidgets, resolveScreenWidget, screenWidgetError, screenWidgets, unregisterScreenWidget,
} from '../../../core/client/src/edition/widgets.ts'

const Render = () => null
const Editor = () => null

test('the camera widget registers on dashboards and SCADA screens', () => {
  const def = cameraWidgetDef({ render: Render as never, configEditor: Editor as never })
  assert.equal(screenWidgetError(def), null)
  const errors: string[] = []
  registerScreenWidgets([def], m => errors.push(m))
  try {
    assert.deepEqual(errors, [])
    assert.equal(getScreenWidget(CAMERA_WIDGET_TYPE, 'dashboard')?.label, 'Camera')
    assert.equal(getScreenWidget(CAMERA_WIDGET_TYPE, 'scada')?.render, Render)
    assert.ok(screenWidgets('dashboard').some(d => d.type === CAMERA_WIDGET_TYPE))
    const resolved = resolveScreenWidget(CAMERA_WIDGET_TYPE, 'scada')
    assert.equal(resolved.available, true)
    // Each placed widget gets its own copy of the default config.
    const a = initialConfigOf(def)
    a.camera_id = 3
    assert.deepEqual(initialConfigOf(def), { ...DEFAULT_CAMERA_CONFIG })
  } finally {
    unregisterScreenWidget(CAMERA_WIDGET_TYPE)
  }
  assert.equal(resolveScreenWidget(CAMERA_WIDGET_TYPE, 'dashboard').available, false)
})

test('a saved config is read defensively', () => {
  assert.deepEqual(cameraConfigOf({}), { camera_id: null, show_name: true })
  assert.deepEqual(cameraConfigOf({ camera_id: 4, show_name: false }), { camera_id: 4, show_name: false })
  assert.equal(cameraConfigOf({ camera_id: '7' }).camera_id, 7)
  assert.equal(cameraConfigOf({ camera_id: -1 }).camera_id, null)
  assert.equal(cameraConfigOf({ camera_id: '1; drop' }).camera_id, null)
  assert.equal(liveUrl(5), '/api/v1/cameras/5/live')
})

const cams = [{ id: 1, name: 'Filler', enabled: true }, { id: 2, name: 'Dock', enabled: false }]
const base = { cameras: cams, loading: false, editMode: false, streamFailed: false }

test('it renders the live MJPEG view of the chosen camera', () => {
  assert.deepEqual(cameraView({ ...base, config: { camera_id: 1, show_name: true } }), {
    kind: 'live', src: '/api/v1/cameras/1/live', name: 'Filler',
  })
})

test('it opens no stream while the screen is edited', () => {
  assert.deepEqual(cameraView({ ...base, editMode: true, config: { camera_id: 1, show_name: true } }), { kind: 'placeholder', name: 'Filler' })
})

test('it explains every state instead of showing a broken image', () => {
  const text = (v: ReturnType<typeof cameraView>) => (v.kind === 'message' ? v.text : v.kind)
  const cfg = { camera_id: 1, show_name: true }
  assert.equal(text(cameraView({ ...base, config: cfg, status: 403 })), 'Your role cannot view cameras')
  assert.equal(text(cameraView({ ...base, config: cfg, status: 500 })), 'Cameras could not be loaded')
  assert.equal(text(cameraView({ ...base, config: { camera_id: null, show_name: true } })), 'No camera chosen')
  assert.equal(text(cameraView({ ...base, editMode: true, config: { camera_id: null, show_name: true } })), 'Choose a camera in the widget settings')
  assert.equal(text(cameraView({ ...base, cameras: undefined, loading: true, config: cfg })), 'Loading camera…')
  assert.equal(text(cameraView({ ...base, config: { camera_id: 9, show_name: true } })), 'This camera no longer exists')
  assert.equal(text(cameraView({ ...base, config: { camera_id: 2, show_name: true } })), 'Dock is turned off')
  assert.match(text(cameraView({ ...base, streamFailed: true, config: cfg })), /^Filler: no picture/)
})
