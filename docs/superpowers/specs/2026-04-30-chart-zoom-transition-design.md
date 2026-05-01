# Chart Zoom Transition — Design Spec

## Overview
Add an iOS-style "app opening/closing" transition when navigating from the main dashboard grid to a sensor's detail view, and back. The clicked sensor card zooms smoothly from its grid position to become the full detail panel. On back, it shrinks back into its exact grid slot with awareness of current scroll/layout.

## Goals
- Feel like opening/closing an app on iOS
- Zero extra dependencies
- 60fps performance via CSS transforms
- Graceful fallbacks when card is no longer in viewport

## Non-Goals
- Morphing chart content from mini to full (too complex, low ROI)
- Animating layout reflow of neighboring grid cards
- Supporting browsers without `getBoundingClientRect` or CSS transforms

## Architecture

### New Files
| File | Purpose |
|---|---|
| `src/hooks/useChartTransition.ts` | FLIP state machine + rect measurement |
| `src/components/ChartTransitionOverlay.tsx` | Fixed overlay with animated clone + backdrop |

### Modified Files
| File | Changes |
|---|---|
| `src/App.tsx` | Wire transition hook; conditionally render overlay; set `selectedSensor` at animation start |
| `src/SensorCard.tsx` | Add `data-sensor-id={s.id}` to root div for back-transition lookup |

## State Machine (`useChartTransition`)

```
idle → capturing → animating-forward → settled → animating-back → idle
```

- **`idle`**: No transition. Overlay not rendered.
- **`capturing`**: Read card rect via `getBoundingClientRect()`. Compute detail target rect. Immediately advance.
- **`animating-forward`**: Render clone at card rect, then on next frame apply CSS transition to detail rect. Fade in backdrop. After 400ms → `settled`.
- **`settled`**: Real `<SensorDetail>` is already visible behind the clone. Hide clone.
- **`animating-back`**: Read current card rect via `[data-sensor-id]`. Animate clone from detail rect down to card rect. Fade out backdrop. After 350ms → `idle`.

## FLIP Mechanics

**First:** `cardRect = cardEl.getBoundingClientRect()`

**Last:** `detailRect` computed as:
- `width = min(1200, viewportWidth - 80)`
- `height = viewportHeight - 120` (header + padding)
- centered in viewport

**Invert:** Clone starts at `position: fixed; left: cardRect.left; top: cardRect.top; width: cardRect.width; height: cardRect.height`. Initial `transform: none`.

**Play:** Apply:
```ts
const scaleX = detailRect.width / cardRect.width;
const scaleY = detailRect.height / cardRect.height;
const translateX = detailRect.left - cardRect.left;
const translateY = detailRect.top - cardRect.top;
// CSS: transform: translate(translateX, translateY) scale(scaleX, scaleY)
// transition: transform 400ms cubic-bezier(0.32, 0.72, 0, 1)
```

## Timing & Easing

| Direction | Duration | Easing |
|---|---|---|
| Forward (zoom in) | 400ms | `cubic-bezier(0.32, 0.72, 0, 1)` |
| Back (zoom out) | 350ms | `cubic-bezier(0.32, 0.72, 0, 1)` |
| Backdrop fade | matches transition | `ease` |

## Backdrop Overlay

Full-viewport fixed div behind the animated clone:

```css
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0);
  backdrop-filter: blur(0px);
  transition: background 400ms, backdrop-filter 400ms;
  pointer-events: none;
  z-index: 100;
}
.overlay.active {
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(8px);
  pointer-events: auto;
}
```

## Zoom-Out Awareness

1. Every `SensorCard` root div gets `data-sensor-id={s.id}`
2. On back: `document.querySelector(\`[data-sensor-id="${id}"]\`)?.getBoundingClientRect()`
3. If found → animate to that rect
4. If not found (scrolled away, screen changed, etc.) → fallback: animate to center-of-viewport + fade out

## `App.tsx` Integration Flow

1. User clicks card → `handleCardClick(sensorId, cardEl)`
2. `useChartTransition.start(sensorId, cardEl)` enters `capturing` state
3. `App.tsx` immediately sets `selectedSensor = sensorId` (detail renders behind overlay)
4. Overlay renders clone + backdrop, animates forward
5. After 400ms → `settled`. Clone hidden. Detail fully visible.
6. User clicks Back → `handleDetailBack()`
7. `useChartTransition.reverse(sensorId)` enters `animating-back`
8. Clone reappears at detail size, animates down to current card rect (or center fallback)
9. After 350ms → `idle`. Overlay removed. `selectedSensor = null`.

## Edge Cases

| Scenario | Behavior |
|---|---|
| Card scrolled out of view | Fallback to center-of-screen shrink + fade |
| Screen changed while detail open | Back zooms to center, then resets state |
| Window resized during transition | Cancel transition, snap to end state |
| Rapid click on another card | Locked — ignore until transition completes |
| Card type = `image` or `text` | Transition still works (whole card zooms) |
| Detail view is taller than viewport | Cap `detailRect.height` to `viewportHeight - 40` |

## Component Interface

### `useChartTransition`
```ts
interface TransitionState {
  phase: 'idle' | 'capturing' | 'animating-forward' | 'settled' | 'animating-back';
  sensorId: number | null;
  cardRect: DOMRect | null;
  detailRect: DOMRect | null;
}

function useChartTransition(): {
  state: TransitionState;
  start: (sensorId: number, cardEl: HTMLElement) => void;
  reverse: (sensorId: number) => void;
}
```

### `ChartTransitionOverlay`
```ts
interface Props {
  state: TransitionState;
  sensor: SensorConfig | null;
  children: React.ReactNode; // the real SensorDetail, rendered behind
}
```

## Performance Notes
- CSS `transform` and `opacity` only — composite-layer friendly
- Clone is a lightweight visual placeholder (matching card background, border, border-radius) — not a full React tree mount; motion carries the effect
- Backdrop blur capped at 8px to avoid GPU strain
- `will-change: transform` on clone during animation

## Accessibility
- Respect `prefers-reduced-motion`: if enabled, skip animation, instant swap
- Trap focus in detail view while overlay is `settled`
- Escape key works for Back (already implemented in `SensorDetail`)
