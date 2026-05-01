# Chart Zoom Transition Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an iOS-style FLIP zoom transition when opening/closing sensor detail from the card grid.

**Architecture:** Pure CSS-transform FLIP technique orchestrated by a custom hook (`useChartTransition`) and a fixed overlay component (`ChartTransitionOverlay`). No new dependencies.

**Tech Stack:** React 19, TypeScript, Vite, CSS transforms

---

## File Map

| File | Action | Responsibility |
|---|---|---|
| `client/src/hooks/useChartTransition.ts` | Create | FLIP state machine with 7 phases |
| `client/src/components/ChartTransitionOverlay.tsx` | Create | Fixed overlay rendering animated clone + backdrop blur |
| `client/src/SensorCard.tsx` | Modify | Add `data-sensor-id` attribute; remove unused `setSelectedSensor` prop |
| `client/src/SensorGrid.tsx` | Modify | Replace `setSelectedSensor` with `onCardClick(id, el)`; pass element to handler |
| `client/src/CanvasView.tsx` | Modify | Stop passing dead `setSelectedSensor` prop to `SensorCard` |
| `client/src/components/MainContent.tsx` | Modify | Add `onCardClick` prop; pass to `SensorGrid` only |
| `client/src/App.tsx` | Modify | Wire hook + overlay; restructure conditional rendering so grid stays mounted |

---

### Task 1: Create `useChartTransition` hook

**Files:**
- Create: `client/src/hooks/useChartTransition.ts`

- [ ] **Step 1: Write the hook**

```typescript
import { useState, useCallback, useRef, useEffect } from 'react'

export type TransitionPhase =
  | 'idle'
  | 'capturing'
  | 'animating-forward'
  | 'settled'
  | 'capturing-back'
  | 'animating-back'

export interface TransitionState {
  phase: TransitionPhase
  sensorId: number | null
  fromRect: DOMRect | null
  toRect: DOMRect | null
}

function computeDetailRect(): DOMRect {
  const vw = window.innerWidth
  const vh = window.innerHeight
  const width = Math.min(1200, vw - 80)
  const height = Math.min(vh - 120, vh - 40)
  const left = (vw - width) / 2
  const top = Math.max(20, (vh - height) / 2 - 20)
  return {
    left, top, width, height,
    right: left + width,
    bottom: top + height,
    x: left,
    y: top,
    toJSON() { return { left, top, width, height } },
  } as DOMRect
}

export function useChartTransition() {
  const [state, setState] = useState<TransitionState>({
    phase: 'idle',
    sensorId: null,
    fromRect: null,
    toRect: null,
  })

  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const rafRef = useRef<number | null>(null)

  const clearTimers = useCallback(() => {
    if (timerRef.current) { clearTimeout(timerRef.current); timerRef.current = null }
    if (rafRef.current) { cancelAnimationFrame(rafRef.current); rafRef.current = null }
  }, [])

  const start = useCallback((sensorId: number, cardEl: HTMLElement) => {
    clearTimers()

    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches
    if (reduced) {
      setState({ phase: 'settled', sensorId, fromRect: null, toRect: null })
      return
    }

    const cardRect = cardEl.getBoundingClientRect()
    const detailRect = computeDetailRect()

    setState({
      phase: 'capturing',
      sensorId,
      fromRect: cardRect,
      toRect: detailRect,
    })

    rafRef.current = requestAnimationFrame(() => {
      rafRef.current = requestAnimationFrame(() => {
        setState(prev => ({ ...prev, phase: 'animating-forward' }))
        timerRef.current = setTimeout(() => {
          setState(prev => ({ ...prev, phase: 'settled' }))
        }, 400)
      })
    })
  }, [clearTimers])

  const reverse = useCallback((sensorId: number) => {
    clearTimers()

    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches
    if (reduced) {
      setState({ phase: 'idle', sensorId: null, fromRect: null, toRect: null })
      return
    }

    const cardEl = document.querySelector(`[data-sensor-id="${sensorId}"]`) as HTMLElement | null
    const cardRect = cardEl?.getBoundingClientRect() ?? null
    const detailRect = computeDetailRect()

    setState({
      phase: 'capturing-back',
      sensorId,
      fromRect: detailRect,
      toRect: cardRect,
    })

    rafRef.current = requestAnimationFrame(() => {
      rafRef.current = requestAnimationFrame(() => {
        setState(prev => ({ ...prev, phase: 'animating-back' }))
        timerRef.current = setTimeout(() => {
          setState({ phase: 'idle', sensorId: null, fromRect: null, toRect: null })
        }, 350)
      })
    })
  }, [clearTimers])

  useEffect(() => {
    return () => clearTimers()
  }, [clearTimers])

  return { state, start, reverse }
}
```

- [ ] **Step 2: Commit**

```bash
git add client/src/hooks/useChartTransition.ts
git commit -m "feat(transition): add useChartTransition FLIP hook"
```

---

### Task 2: Create `ChartTransitionOverlay` component

**Files:**
- Create: `client/src/components/ChartTransitionOverlay.tsx`

- [ ] **Step 1: Write the component**

```tsx
import { useMemo } from 'react'
import { TransitionState } from '../hooks/useChartTransition'

interface Props {
  state: TransitionState
}

export function ChartTransitionOverlay({ state }: Props) {
  const { phase, fromRect, toRect } = state

  const isActive = phase === 'animating-forward' || phase === 'animating-back' || phase === 'capturing' || phase === 'capturing-back'
  const showClone = phase !== 'idle' && phase !== 'settled' && fromRect != null

  const cloneStyle = useMemo((): React.CSSProperties => {
    if (!fromRect || !showClone) return { display: 'none' }

    const base: React.CSSProperties = {
      position: 'fixed',
      left: fromRect.left,
      top: fromRect.top,
      width: fromRect.width,
      height: fromRect.height,
      borderRadius: 10,
      background: 'var(--card)',
      border: '1px solid var(--border)',
      boxShadow: '0 24px 48px rgba(0,0,0,0.4)',
      zIndex: 101,
      willChange: 'transform',
      transformOrigin: 'top left',
    }

    if (phase === 'capturing' || phase === 'capturing-back') {
      return { ...base, transform: 'none', transition: 'none' }
    }

    if (phase === 'animating-forward' && toRect) {
      const sx = toRect.width / fromRect.width
      const sy = toRect.height / fromRect.height
      const tx = toRect.left - fromRect.left
      const ty = toRect.top - fromRect.top
      return {
        ...base,
        transform: `translate(${tx}px, ${ty}px) scale(${sx}, ${sy})`,
        transition: 'transform 400ms cubic-bezier(0.32, 0.72, 0, 1)',
      }
    }

    if (phase === 'animating-back') {
      if (toRect) {
        const sx = toRect.width / fromRect.width
        const sy = toRect.height / fromRect.height
        const tx = toRect.left - fromRect.left
        const ty = toRect.top - fromRect.top
        return {
          ...base,
          transform: `translate(${tx}px, ${ty}px) scale(${sx}, ${sy})`,
          transition: 'transform 350ms cubic-bezier(0.32, 0.72, 0, 1)',
        }
      } else {
        // Fallback: shrink to center + fade
        const cx = window.innerWidth / 2 - fromRect.left - fromRect.width / 2
        const cy = window.innerHeight / 2 - fromRect.top - fromRect.height / 2
        return {
          ...base,
          transform: `translate(${cx}px, ${cy}px) scale(0.1)`,
          opacity: 0,
          transition: 'transform 350ms cubic-bezier(0.32, 0.72, 0, 1), opacity 350ms ease',
        }
      }
    }

    return { display: 'none' }
  }, [phase, fromRect, toRect, showClone])

  const backdropStyle: React.CSSProperties = {
    position: 'fixed',
    inset: 0,
    zIndex: 100,
    background: isActive ? 'rgba(0,0,0,0.5)' : 'rgba(0,0,0,0)',
    backdropFilter: isActive ? 'blur(8px)' : 'blur(0px)',
    WebkitBackdropFilter: isActive ? 'blur(8px)' : 'blur(0px)',
    transition: 'background 400ms ease, backdrop-filter 400ms ease',
    pointerEvents: isActive ? 'auto' : 'none',
  }

  if (phase === 'idle') return null

  return (
    <div style={backdropStyle}>
      <div style={cloneStyle} />
    </div>
  )
}
```

- [ ] **Step 2: Commit**

```bash
git add client/src/components/ChartTransitionOverlay.tsx
git commit -m "feat(transition): add ChartTransitionOverlay component"
```

---

### Task 3: Update `SensorCard` — add `data-sensor-id` and remove dead prop

**Files:**
- Modify: `client/src/SensorCard.tsx`

- [ ] **Step 1: Remove `setSelectedSensor` from props and `onClick` usage**

In `client/src/SensorCard.tsx`, find the `Props` interface and remove `setSelectedSensor`. Find the destructured props and remove it. The component never reads it.

```tsx
// BEFORE
interface Props {
  sensor: SensorConfig
  editMode: boolean
  activeScreen: string
  editingId: number | null
  shortcut: string | undefined
  loadSensorsForScreen: () => void
  setEditingId: (id: number | null) => void
  setSelectedSensor: (id: number | null) => void
  showDragHandle?: boolean
  selected?: boolean
  mode: 'live' | 'history' | 'system'
  onClick?: (e: React.MouseEvent) => void
}

// AFTER
interface Props {
  sensor: SensorConfig
  editMode: boolean
  activeScreen: string
  editingId: number | null
  shortcut: string | undefined
  loadSensorsForScreen: () => void
  setEditingId: (id: number | null) => void
  showDragHandle?: boolean
  selected?: boolean
  mode: 'live' | 'history' | 'system'
  onClick?: (e: React.MouseEvent) => void
}
```

In the component signature:

```tsx
// BEFORE
export const SensorCard = React.memo(function SensorCard({
  sensor: s, editMode, activeScreen, editingId,
  shortcut, loadSensorsForScreen, setEditingId, setSelectedSensor,
  showDragHandle, selected, mode, onClick,
}: Props) {

// AFTER
export const SensorCard = React.memo(function SensorCard({
  sensor: s, editMode, activeScreen, editingId,
  shortcut, loadSensorsForScreen, setEditingId,
  showDragHandle, selected, mode, onClick,
}: Props) {
```

- [ ] **Step 2: Add `data-sensor-id` to root div**

Find the root `<div>` inside `SensorCard` and add the attribute:

```tsx
<div
  data-sensor-id={s.id}
  style={{ ... }}
  onClick={onClick}
>
```

- [ ] **Step 3: Remove `setSelectedSensor` from custom comparison**

In the `React.memo` comparison function at the bottom of `SensorCard.tsx`, remove the `setSelectedSensor` check (there isn't one currently because it's not compared, but if there were, remove it). The current comparison only checks: `id`, `editMode`, `activeScreen`, `editingId`, `selected`, `shortcut`, `showDragHandle`, `mode`. No change needed here.

- [ ] **Step 4: Commit**

```bash
git add client/src/SensorCard.tsx
git commit -m "refactor(SensorCard): remove dead setSelectedSensor prop, add data-sensor-id"
```

---

### Task 4: Update `SensorGrid` — `onCardClick` with element

**Files:**
- Modify: `client/src/SensorGrid.tsx`

- [ ] **Step 1: Update props interface**

```tsx
// BEFORE
interface Props {
  screenLayouts: ScreenLayouts
  screenSensors: SensorConfig[]
  editMode: boolean
  activeScreen: string
  gap: number
  editingId: number | null
  setEditingId: (id: number | null) => void
  setSelectedSensor: (id: number | null) => void
  loadSensorsForScreen: () => void
  onLayoutChange: (currentLayout: any[], allLayouts: { [bp: string]: any[] }) => void
  onSave: (s: SensorConfig, shortcut?: string | null) => void
  onDelete: (id: number) => void
  forcedBreakpoint: string | null
  shortcuts: Record<number, string>
  mode: 'live' | 'history' | 'system'
}

// AFTER
interface Props {
  screenLayouts: ScreenLayouts
  screenSensors: SensorConfig[]
  editMode: boolean
  activeScreen: string
  gap: number
  editingId: number | null
  setEditingId: (id: number | null) => void
  onCardClick?: (id: number, el: HTMLElement) => void
  loadSensorsForScreen: () => void
  onLayoutChange: (currentLayout: any[], allLayouts: { [bp: string]: any[] }) => void
  onSave: (s: SensorConfig, shortcut?: string | null) => void
  onDelete: (id: number) => void
  forcedBreakpoint: string | null
  shortcuts: Record<number, string>
  mode: 'live' | 'history' | 'system'
}
```

- [ ] **Step 2: Update destructured props and SensorCard call**

```tsx
// BEFORE
export function SensorGrid({ screenLayouts, screenSensors, editMode, activeScreen, gap, editingId, setEditingId, setSelectedSensor, loadSensorsForScreen, onLayoutChange, onSave, onDelete, forcedBreakpoint, shortcuts, mode }: Props) {

// AFTER
export function SensorGrid({ screenLayouts, screenSensors, editMode, activeScreen, gap, editingId, setEditingId, onCardClick, loadSensorsForScreen, onLayoutChange, onSave, onDelete, forcedBreakpoint, shortcuts, mode }: Props) {
```

In the `SensorCard` JSX call, remove `setSelectedSensor={setSelectedSensor}`:

```tsx
// BEFORE
<SensorCard
  sensor={s}
  editMode={editMode}
  activeScreen={activeScreen}
  editingId={editingId}
  shortcut={shortcuts[s.id]}
  loadSensorsForScreen={loadSensorsForScreen}
  setEditingId={setEditingId}
  setSelectedSensor={setSelectedSensor}
  showDragHandle={true}
  mode={mode}
  onClick={e => { ... }}
/>

// AFTER
<SensorCard
  sensor={s}
  editMode={editMode}
  activeScreen={activeScreen}
  editingId={editingId}
  shortcut={shortcuts[s.id]}
  loadSensorsForScreen={loadSensorsForScreen}
  setEditingId={setEditingId}
  showDragHandle={true}
  mode={mode}
  onClick={e => { ... }}
/>
```

- [ ] **Step 3: Update onClick handler to call `onCardClick`**

```tsx
// BEFORE
onClick={e => {
  if (didDrag.current) { didDrag.current = false; return; }
  if (editMode) { e.stopPropagation(); setEditingId(editingId === s.id ? null : s.id); }
  else { setSelectedSensor(s.id); }
}}

// AFTER
onClick={e => {
  if (didDrag.current) { didDrag.current = false; return; }
  if (editMode) { e.stopPropagation(); setEditingId(editingId === s.id ? null : s.id); }
  else { onCardClick?.(s.id, e.currentTarget); }
}}
```

- [ ] **Step 4: Commit**

```bash
git add client/src/SensorGrid.tsx
git commit -m "feat(SensorGrid): replace setSelectedSensor with onCardClick including element"
```

---

### Task 5: Update `CanvasView` — stop passing dead prop

**Files:**
- Modify: `client/src/CanvasView.tsx`

- [ ] **Step 1: Remove `setSelectedSensor` from SensorCard call(s)**

Search for all `<SensorCard` usages in `CanvasView.tsx`. Remove the `setSelectedSensor={setSelectedSensor}` prop from each.

There are two occurrences inside `CanvasView.tsx` (one for normal cards, one for selected group). Remove `setSelectedSensor={setSelectedSensor}` from both.

- [ ] **Step 2: Commit**

```bash
git add client/src/CanvasView.tsx
git commit -m "refactor(CanvasView): stop passing dead setSelectedSensor prop"
```

---

### Task 6: Update `MainContent` — pass `onCardClick` through

**Files:**
- Modify: `client/src/components/MainContent.tsx`

- [ ] **Step 1: Add `onCardClick` prop to interface and destructuring**

```tsx
// BEFORE
interface Props {
  viewMode: 'cards' | 'canvas' | 'twin' | 'scada'
  mode: 'live' | 'history' | 'system'
  screenSensors: SensorConfig[]
  effectiveLayouts: ScreenLayouts
  canvasLayouts: CanvasLayoutItem[]
  editMode: boolean
  activeScreen: string
  gap: number
  editingId: number | null
  setEditingId: (id: number | null) => void
  setSelectedSensor: (id: number | null) => void
  onLayoutChange: (current: any[], all: { [bp: string]: any[] }) => void
  onCanvasLayoutChange: (items: CanvasLayoutItem[]) => void
  onSave: (s: SensorConfig, shortcut?: string | null) => void
  onDelete: (id: number) => void
  forcedBreakpoint: string | null
  shortcuts: Record<number, string>
  scadaEquipment: any[]
  scadaConnections: any[]
  twinLayouts: any[]
  twinCamera: any
  twinConnections: any[]
  sensors: SensorConfig[]
  saveScadaEquipmentMut: any
  saveScadaConnectionsMut: any
  saveTwinLayoutMut: any
  saveTwinConnectionsMut: any
  updateSensorMut: any
  fetchOk: any
}

// AFTER
interface Props {
  viewMode: 'cards' | 'canvas' | 'twin' | 'scada'
  mode: 'live' | 'history' | 'system'
  screenSensors: SensorConfig[]
  effectiveLayouts: ScreenLayouts
  canvasLayouts: CanvasLayoutItem[]
  editMode: boolean
  activeScreen: string
  gap: number
  editingId: number | null
  setEditingId: (id: number | null) => void
  setSelectedSensor: (id: number | null) => void
  onCardClick?: (id: number, el: HTMLElement) => void
  onLayoutChange: (current: any[], all: { [bp: string]: any[] }) => void
  onCanvasLayoutChange: (items: CanvasLayoutItem[]) => void
  onSave: (s: SensorConfig, shortcut?: string | null) => void
  onDelete: (id: number) => void
  forcedBreakpoint: string | null
  shortcuts: Record<number, string>
  scadaEquipment: any[]
  scadaConnections: any[]
  twinLayouts: any[]
  twinCamera: any
  twinConnections: any[]
  sensors: SensorConfig[]
  saveScadaEquipmentMut: any
  saveScadaConnectionsMut: any
  saveTwinLayoutMut: any
  saveTwinConnectionsMut: any
  updateSensorMut: any
  fetchOk: any
}
```

In the function signature:

```tsx
// BEFORE
export function MainContent({
  viewMode, mode, screenSensors, effectiveLayouts, canvasLayouts,
  editMode, activeScreen, gap, editingId, setEditingId, setSelectedSensor,
  onLayoutChange, onCanvasLayoutChange, onSave, onDelete,
  forcedBreakpoint, shortcuts, scadaEquipment, scadaConnections,
  twinLayouts, twinCamera, twinConnections, sensors,
  saveScadaEquipmentMut, saveScadaConnectionsMut, saveTwinLayoutMut,
  saveTwinConnectionsMut, updateSensorMut, fetchOk,
}: Props) {

// AFTER
export function MainContent({
  viewMode, mode, screenSensors, effectiveLayouts, canvasLayouts,
  editMode, activeScreen, gap, editingId, setEditingId, setSelectedSensor, onCardClick,
  onLayoutChange, onCanvasLayoutChange, onSave, onDelete,
  forcedBreakpoint, shortcuts, scadaEquipment, scadaConnections,
  twinLayouts, twinCamera, twinConnections, sensors,
  saveScadaEquipmentMut, saveScadaConnectionsMut, saveTwinLayoutMut,
  saveTwinConnectionsMut, updateSensorMut, fetchOk,
}: Props) {
```

- [ ] **Step 2: Pass `onCardClick` to `SensorGrid`**

```tsx
// BEFORE
<SensorGrid
  screenLayouts={effectiveLayouts}
  screenSensors={screenSensors}
  editMode={editMode}
  activeScreen={activeScreen}
  gap={gap}
  editingId={editingId}
  setEditingId={setEditingId}
  setSelectedSensor={setSelectedSensor}
  loadSensorsForScreen={loadSensorsForScreen}
  onLayoutChange={onLayoutChange}
  onSave={onSave}
  onDelete={onDelete}
  forcedBreakpoint={forcedBreakpoint}
  shortcuts={shortcuts}
  mode={mode}
/>

// AFTER
<SensorGrid
  screenLayouts={effectiveLayouts}
  screenSensors={screenSensors}
  editMode={editMode}
  activeScreen={activeScreen}
  gap={gap}
  editingId={editingId}
  setEditingId={setEditingId}
  onCardClick={onCardClick}
  loadSensorsForScreen={loadSensorsForScreen}
  onLayoutChange={onLayoutChange}
  onSave={onSave}
  onDelete={onDelete}
  forcedBreakpoint={forcedBreakpoint}
  shortcuts={shortcuts}
  mode={mode}
/>
```

- [ ] **Step 3: Commit**

```bash
git add client/src/components/MainContent.tsx
git commit -m "feat(MainContent): add onCardClick prop and pass to SensorGrid"
```

---

### Task 7: Wire everything in `App.tsx`

**Files:**
- Modify: `client/src/App.tsx`

- [ ] **Step 1: Add imports**

At the top of `client/src/App.tsx`, add:

```tsx
import { useChartTransition } from './hooks/useChartTransition'
import { ChartTransitionOverlay } from './components/ChartTransitionOverlay'
```

- [ ] **Step 2: Instantiate the hook inside `AppInner`**

After the `queryClient` declaration, add:

```tsx
const { state: transition, start: startTransition, reverse: reverseTransition } = useChartTransition()
```

- [ ] **Step 3: Create card click handler**

In the handlers section of `AppInner`, add:

```tsx
const handleCardClick = useCallback((id: number, el: HTMLElement) => {
  startTransition(id, el)
  setSelectedSensor(id)
}, [startTransition])

const handleDetailBack = useCallback(() => {
  if (selectedSensor != null) {
    reverseTransition(selectedSensor)
  }
}, [selectedSensor, reverseTransition])
```

- [ ] **Step 4: Auto-clear selectedSensor when transition ends**

Add a `useEffect` near the other effects:

```tsx
useEffect(() => {
  if (transition.phase === 'idle' && selectedSensor != null) {
    setSelectedSensor(null)
  }
}, [transition.phase, selectedSensor])
```

- [ ] **Step 5: Replace `onBack` in SensorDetail**

Find the `SensorDetail` call and change `onBack`:

```tsx
// BEFORE
<SensorDetail sensor={s} onBack={() => setSelectedSensor(null)} ... />

// AFTER
<SensorDetail sensor={s} onBack={handleDetailBack} ... />
```

- [ ] **Step 6: Restructure the main content area**

Replace the main content rendering block (around lines 349–419) with:

```tsx
<div style={{ flex: 1, overflow: transition.phase !== 'idle' ? 'hidden' : 'auto', padding: '0 20px 20px', display: 'flex', flexDirection: 'column', position: 'relative' }}>
  {/* Tabs — hidden only when transition is fully settled */}
  {transition.phase !== 'settled' && (
    <>
      <ScreenTabs
        mode={mode} activeScreen={activeScreen} screens={screens}
        effectiveNavShortcuts={effectiveNavShortcuts} editMode={editMode}
        onTabClick={(id) => { setActiveScreen(id); if (mode === 'system') setMode('live') }}
        onSystemClick={() => { setMode('system'); setActiveScreen('all') }}
        onRenameScreen={handleRenameScreen}
        onDeleteScreen={handleDeleteScreen}
        onAddScreen={handleAddScreen}
        isAdmin={isAdmin}
        onToggleEditMode={handleToggleEditMode}
        editLabel={editMode ? 'Done' : 'Edit'}
        onAddSource={() => setShowAddDialog(true)}
        canAddSource={activeScreen !== 'all'}
        onAutoArrange={handleAutoArrange}
      />

      {editMode && (
        <LayoutBreakpointSelector layoutBreakpoint={layoutBreakpoint} onChange={setLayoutBreakpoint} />
      )}
    </>
  )}

  {/* Grid content — always mounted so card DOM exists for reverse lookup */}
  <div style={{
    visibility: transition.phase === 'settled' ? 'hidden' : 'visible',
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    overflow: transition.phase === 'idle' ? 'auto' : 'hidden',
  }}>
    {mode === 'system' && <SystemPanel systemInfo={effectiveSystemInfo} />}
    <MainContent
      viewMode={viewMode} mode={mode}
      screenSensors={screenSensors}
      effectiveLayouts={effectiveLayouts}
      canvasLayouts={canvasLayouts}
      editMode={editMode} activeScreen={activeScreen}
      gap={dashSettings.gap}
      editingId={editingId}
      setEditingId={setEditingId}
      setSelectedSensor={setSelectedSensor}
      onCardClick={handleCardClick}
      onLayoutChange={handleLayoutChange}
      onCanvasLayoutChange={handleCanvasLayoutChange}
      onSave={handleSave}
      onDelete={handleDelete}
      forcedBreakpoint={layoutBreakpoint}
      shortcuts={shortcuts}
      scadaEquipment={scadaEquipment}
      scadaConnections={scadaConnections}
      twinLayouts={twinLayouts}
      twinCamera={twinCamera}
      twinConnections={twinConnections}
      sensors={sensors}
      saveScadaEquipmentMut={saveScadaEquipmentMut}
      saveScadaConnectionsMut={saveScadaConnectionsMut}
      saveTwinLayoutMut={saveTwinLayoutMut}
      saveTwinConnectionsMut={saveTwinConnectionsMut}
      updateSensorMut={updateSensorMut}
      fetchOk={fetchOk}
    />
  </div>

  {/* Detail view */}
  {selectedSensor != null && (
    <div style={{
      position: 'absolute',
      inset: 0,
      zIndex: 2,
      display: transition.phase === 'animating-forward' || transition.phase === 'capturing' ? 'none' : 'flex',
      flexDirection: 'column',
      padding: '0 20px 20px',
      overflow: 'hidden',
    }}>
      {(() => {
        const s = sensors.find(x => x.id === selectedSensor)
        return s ? <SensorDetail sensor={s} onBack={handleDetailBack} historyRange={historyRange} onRangeChange={setHistoryRange} historyDetail={historyDetail} onDetailChange={setHistoryDetail} historyAutoAgg={dashSettings.historyAutoAgg} shortcuts={shortcuts} mode={mode === 'history' ? 'history' : 'live'} onModeChange={setMode} allSensors={sensors} /> : null
      })()}
    </div>
  )}

  <ChartTransitionOverlay state={transition} />

  {showDashSettings && <DashboardSettings settings={dashSettings} onChange={setDashSettings} onClose={() => setShowDashSettings(false)} alarmMuted={alarmMuted} onToggleMute={() => { toggleMute(); setAlarmMuted(!alarmMuted) }} />}
  {showDataManager && <DataManager sensors={sensors} onSave={handleSave} onDelete={handleDelete} onAdd={handleAddSensorToManager} onDuplicate={handleDuplicateSensor} onClose={() => setShowDataManager(false)} shortcuts={shortcuts} isAdmin={isAdmin} />}
  {showAddDialog && activeScreen !== 'all' && (
    <AddSensorDialog screenId={activeScreen} sensors={sensors} screenSensorIds={screenSensors.map(s => s.id)} onClose={() => setShowAddDialog(false)} onDone={() => { setShowAddDialog(false); queryClient.invalidateQueries({ queryKey: ['screenDataSources', activeScreen] }); queryClient.invalidateQueries({ queryKey: ['sensors'] }) }} />
  )}
  {screenPrompt && <PromptDialog {...screenPrompt} onClose={() => setScreenPrompt(null)} />}
  {screenConfirm && <ConfirmDialog {...screenConfirm} onClose={() => setScreenConfirm(null)} />}
  {showBrandingDialog && <BrandingDialog branding={branding} onSave={handleSaveBranding} onClose={() => setShowBrandingDialog(false)} />}
  {showAlarmPanel && <AlarmPanel onClose={() => setShowAlarmPanel(false)} />}
</div>
```

**Important:** The dialogs (`showDashSettings`, `showDataManager`, etc.) that were previously inside the same conditional block as the grid are now outside the `selectedSensor != null` conditional and render unconditionally at the bottom of the flex container. Make sure they remain inside the main `<div>` but outside the grid/detail blocks.

- [ ] **Step 7: Remove unused `useCallback` import if needed**

`useCallback` is already imported in `App.tsx` (line 1). No change needed.

- [ ] **Step 8: Build and verify**

```bash
cd client && npm run build
```

Expected: TypeScript compiles without errors. If errors, fix type mismatches (e.g., `SensorGrid` prop changes).

- [ ] **Step 9: Commit**

```bash
git add client/src/App.tsx
git commit -m "feat(App): wire FLIP chart zoom transition"
```

---

## Verification

1. Open the app in browser dev mode (`npm run dev`)
2. Click any sensor card in the grid → card should zoom smoothly to detail view with backdrop blur
3. Click Back → detail should shrink smoothly back to the card's current grid position
4. Scroll the grid, open a card, then click Back → should shrink to the card's new scrolled position
5. Press Escape to go back → should trigger the same reverse animation
6. Enable `prefers-reduced-motion` in OS → transition should be instant
7. Switch screens while detail is open, then click Back → should gracefully fallback to center shrink

## Self-Review Checklist

- [ ] **Spec coverage:** FLIP hook (Task 1), overlay (Task 2), card attribute (Task 3), grid click handler (Task 4), CanvasView cleanup (Task 5), MainContent prop (Task 6), App wiring (Task 7) — all covered.
- [ ] **Placeholder scan:** No TBDs, TODOs, or vague steps. Every step has exact code.
- [ ] **Type consistency:** `TransitionState` interface used consistently. `onCardClick` signature `(id: number, el: HTMLElement) => void` used in `SensorGrid`, `MainContent`, and `App.tsx`.
