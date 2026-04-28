# Lit Migration Plan — Voltrus Lite Client

> **Status:** Development / Experimentation Phase  
> **Goal:** Evaluate Lit (Web Components) as a React replacement while keeping the existing React client intact.  
> **Principle:** Parallel development. No deletion of React code until Lit is proven feature-complete.

---

## 1. Why Lit?

| Concern | React (Current) | Lit (Target) |
|---------|----------------|--------------|
| Direct deps | 10 | 4 |
| Transitive deps | ~149 | ~10 |
| `node_modules` size | 268 MB | ~2 MB |
| Build tool required | Vite + native binaries | None (ESM native) |
| Component standard | Framework-specific | W3C Web Standard |
| Longevity risk | High (React 19 is months old) | Low (standard APIs, tiny library) |

Lit gives us a component model nearly identical to React (props, reactive state, scoped styles) but compiles to native browser APIs. If Lit disappeared tomorrow, our components still run.

---

## 2. Parallel Setup Strategy

We maintain **two clients side-by-side**:

```
client/          ← React (production, unchanged)
client-lit/      ← Lit (experimentation, new)
```

- `client/` continues to be the primary development target.
- `client-lit/` is a sandbox. We port views one at a time and compare.
- The Rust backend serves both from different static routes if desired, or you run them on different ports during dev.

### Dev Workflow

```bash
# Terminal 0 — Rust backend (must be running first)
cd /project-root && cargo run   # http://localhost:3100

# Terminal 1 — React (existing)
cd client && npm run dev        # http://localhost:5173

# Terminal 2 — Lit (new)
cd client-lit
npm run build                   # one-time: compile TS → dist/
npm run dev                     # http://localhost:5174
```

**Important:** The Lit client connects directly to `http://localhost:3100` (the Rust backend). Unlike the React client, it does not use a proxy — `serve` is a static file server only. The Rust backend must be running before you open the Lit client.

---

## 3. Architecture Mapping

### React → Lit Concepts

| React | Lit Equivalent | Notes |
|-------|---------------|-------|
| Component function / class | `LitElement` subclass with `@customElement` | Very similar mental model |
| `useState` | `@state()` decorated property | Reactive by default |
| `useEffect` + `useRef` | `connectedCallback()` / `disconnectedCallback()` | Lifecycle methods on the element |
| `props` | `@property()` decorated property | Typed, observable |
| JSX | `html` tagged template literal | Slightly different syntax, same result |
| `useContext` | Custom events or a shared store class | We use a tiny event-based store |
| TanStack Query | `fetch()` + `EventSource` wrapped in a service class | ~80 lines, zero deps |
| Inline styles / CSS-in-JS | `static styles = css\`…\`` | Scoped to Shadow DOM automatically |

### State Management Strategy

No Redux, no Zustand, no Context API. Just a thin service layer:

```
services/api.ts        → SSE connection + REST fetch wrapper
services/auth.ts       → JWT cookie handling, login/logout
services/store.ts      → Tiny observable store for shared UI state (active screen, edit mode, etc.)
```

All services are plain TypeScript classes that extend `EventTarget`. Components subscribe with `addEventListener` and re-render when data changes.

---

## 4. Migration Order (Views First, Shell Last)

We port from the **leaf components inward**. The app shell is the last thing to migrate.

### Phase 0: Scaffold ✅ (Done)
- [x] `client-lit/` directory created
- [x] `package.json` with pinned exact versions
- [x] `tsconfig.json` with decorator support
- [x] `index.html` with import map (no bundler)
- [x] `services/api.ts` — SSE data manager
- [x] `components/sensor-card.ts` — first shared component
- [x] `views/grid-view.ts` — first full view
- [x] `components/app-shell.ts` — basic shell with nav
- [x] Dev server wired (`npx serve` + `dist/` output, no python)

### Phase 1: Grid View (In Progress)
- [ ] Sensor card with sparkline (Canvas 2D micro-chart)
- [ ] Drag-and-drop grid layout (native HTML5 DnD or a tiny custom implementation)
- [ ] Responsive breakpoints (CSS Grid `minmax()` + media queries)
- [ ] Edit mode toggle
- [ ] Screen switching integration

### Phase 2: Canvas View
- [ ] Free-form absolute positioning
- [ ] Drag / resize sensors (native mouse events)
- [ ] Save/load layout via API

### Phase 3: 2D SCADA View
- [ ] SVG-based diagram canvas
- [ ] Equipment primitives (transformer, breaker, meter, switch, motor, battery)
- [ ] Draw / delete power connections and signal lines
- [ ] Waypoint routing for lines

### Phase 4: 3D Digital Twin
- [ ] Import Three.js directly as ESM module
- [ ] Scene setup, camera, orbit controls
- [ ] Load equipment models / primitives
- [ ] Drag, rotate, connect equipment
- [ ] Save camera positions per screen

> **Note on 3D:** We drop `@react-three/fiber` and `@react-three/drei`. Three.js is used imperatively inside a Lit element. This is actually closer to how most Three.js projects were built before React wrappers existed — and it’s more stable long-term.

### Phase 5: Shared Components
- [ ] Login screen
- [ ] Alarm panel
- [ ] Sensor detail modal + chart
- [ ] History view + time range picker
- [ ] Dashboard settings panel
- [ ] User manager
- [ ] Modbus device manager
- [ ] Screen dialog
- [ ] Shortcuts config

### Phase 6: App Shell & Router
- [ ] View router
- [ ] Auth guard
- [ ] Keyboard shortcuts (global `keydown` listener)
- [ ] Alarm audio
- [ ] Service worker / push notifications groundwork

### Phase 7: Polish & Feature Parity
- [ ] Branding dialog
- [ ] Data retention UI
- [ ] System monitoring panel
- [ ] CSV / PDF export (if applicable to frontend)
- [ ] Accessibility audit
- [ ] Performance audit (bundle size, render perf)

---

## 5. Build & Deployment Strategy

### Development (Now)
- **No bundler.** `tsc --watch` compiles `.ts` → `.js` into `dist/`.
- `npx serve -l 5174 .` serves the project root with correct MIME types.
- `index.html` lives at the project root and loads `/dist/main.js`.
- Import map in `index.html` resolves `lit`, `three`, `marked`, `dompurify` from `node_modules`.

### Production (Later)
Option A — **Zero-bundler (Recommended for longevity)**
- Run `tsc` to compile `.ts` → `dist/`.
- Serve `index.html` + `dist/` + `node_modules/` as static files from the Rust backend.
- The browser resolves modules via import maps. This works today in Chrome, Firefox, Safari, and Edge.

Option B — **Minimal bundler (If we need polyfills or older browser support)**
- Use `esbuild` or `rollup` only at release time.
- The dev workflow stays bundler-free, so build tool rot only affects CI, not daily development.

---

## 6. Dependency Strategy

### What We Keep (Green)
| Package | Role | Risk |
|---------|------|------|
| `lit` | Component base class + reactivity | Low — 3 deps, 5.8M weekly downloads, Google-backed |
| `three` | 3D rendering | Medium — but we use it directly, no R3F wrapper |
| `marked` | Markdown rendering | Low — stable, self-contained |
| `dompurify` | XSS sanitization | Low — security-focused, stable |

### What We Drop (Red → Gone)
- `react`, `react-dom`
- `@react-three/fiber`, `@react-three/drei`
- `react-grid-layout`
- `react-rnd`
- `@tanstack/react-query`
- `vite`, `@vitejs/plugin-react`
- All `@types/*` packages (Lit and Three.js ship their own types)

**Final direct dependency count: 4.**  
**Final transitive dependency count: ~10.**  
**Final `node_modules` size: ~2–3 MB.**

---

## 7. Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Lit becomes unmaintained | Web Components are a browser standard. We can drop Lit and write raw `HTMLElement` subclasses. The API surface is small enough to vendor (~5KB). |
| Three.js breaks APIs | We pin to an exact version (`0.184.0`). Three.js is the *only* fast-moving dep left. We accept this risk because 3D is a core feature. |
| TypeScript breaks decorators | We use `experimentalDecorators: true`. If TS drops support, Lit will migrate to TC39 decorators (already in progress). The migration is mechanical. |
| Import maps not supported on target hardware | Import maps are supported in all evergreen browsers since 2021. For truly exotic embedded browsers, we fall back to Option B (minimal bundler). |
| Rewrite takes too long | Parallel development means React keeps shipping. There is no deadline pressure. We migrate view-by-view and can stop at any time. |

---

## 8. Decision Gates

| Gate | Condition | Action |
|------|-----------|--------|
| **Gate 1** | Grid view in Lit reaches feature parity with React | Continue to Phase 2 |
| **Gate 2** | Any view proves infeasible in Lit (e.g., 3D performance) | Re-evaluate. Keep React for that view only; Lit handles the rest. |
| **Gate 3** | All views ported and stable for 2 weeks | Declare `client-lit/` the new primary. `client/` enters maintenance mode. |
| **Gate 4** | 1 month stable in production with Lit | Archive `client/` to `client-react-archive/`. Delete from active repo. |

---

## 9. Immediate Next Steps

1. **Run both clients side-by-side**  
   `cd client && npm run dev` + `cd client-lit && npm run dev`

2. **Port the SensorCard sparkline**  
   The React client uses a charting library (or Canvas?). Port that to a `<canvas>` micro-chart inside `sensor-card.ts`.

3. **Implement drag-and-drop for the grid**  
   Evaluate whether native HTML5 Drag and Drop API is sufficient, or if we need a tiny custom mouse-drag implementation (likely under 100 lines).

4. **Connect to real backend data**  
   Point `services/api.ts` at the real Rust SSE endpoint and verify live data flows into the Lit grid.

5. **Compare bundle sizes**  
   Once Grid view is complete, compare the network payload of `client/` vs `client-lit/` loading the same data.

---

## 10. Long-Term Vision

If the Lit migration succeeds, the Voltrus Lite frontend becomes:

- **A set of standard Web Components** that could theoretically be dropped into any HTML page without a build step.
- **Vendorable in its entirety** — `node_modules` is 2 MB. You can commit it to git without shame.
- **Immune to framework churn** — no more React major versions, no more Vite breaking changes, no more `npm audit` fire drills.
- **Runnable in 2040** — as long as browsers support ES modules and Web Components (both are W3C standards), the code runs.

---

*Document version: 2025-04-28*  
*Author: OpenCode Agent*  
*Next review: After Gate 1 (Grid view parity)*
