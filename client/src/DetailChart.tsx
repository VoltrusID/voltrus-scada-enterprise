import { useRef, useEffect, useState, useMemo } from 'react';
import { DataPoint } from './types';
import { fmtTs, safeMinMax, toHexColor } from './styles';

export interface OverlaySeries {
  data: DataPoint[];
  color: string;
  name: string;
  unit: string;
}

export function DetailChart({ data: rawData, color: rawColor, warn, crit, unit, overlays = [], pointWidth = 0 }: { data: DataPoint[]; color: string; warn: number; crit: number; unit: string; overlays?: OverlaySeries[]; pointWidth?: number }) {
  const data = rawData.filter(d => d.value != null && !Number.isNaN(d.value));
  const color = toHexColor(rawColor);
  const validOverlays = useMemo(() => overlays.map(o => ({ ...o, data: o.data.filter((d: DataPoint) => d.value != null && !Number.isNaN(d.value)), color: toHexColor(o.color) })).filter(o => o.data.length >= 2), [overlays]);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const wrapperRef = useRef<HTMLDivElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const [hover, setHover] = useState<{ idx: number; clientX: number; clientY: number; py: number } | null>(null);
  const areaRef = useRef({ pL: 60, pR: 20, pT: 20, pB: 44, cW: 0, cH: 0, yMin: 0, yMax: 0, dLen: 0 });

  useEffect(() => {
    const cv = canvasRef.current;
    const wrapper = wrapperRef.current;
    if (!cv || !wrapper || data.length < 2) return;
    const ctx = cv.getContext('2d')!, dpr = devicePixelRatio || 1;

    // Measure the wrapper's current size (it always has width:100%, height:100%)
    const wrapRect = wrapper.getBoundingClientRect();
    const wrapW = wrapRect.width, wrapH = wrapRect.height;
    if (wrapW <= 0 || wrapH <= 0) return;

    // Virtual chart width
    const W = pointWidth > 0 ? Math.max(pointWidth * data.length + 80, wrapW) : wrapW;
    const H = wrapH;

    // Size the wrapper to virtual width (this creates scroll extent)
    wrapper.style.width = W + 'px';

    // Canvas fills the wrapper, buffer matches virtual size
    cv.width = W * dpr;
    cv.height = H * dpr;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    const pL = 60, pR = 20, pT = 20, pB = 44, cW = W - pL - pR, cH = H - pT - pB;
    if (cW <= 0 || cH <= 0) return;
    const values = data.map(d => d.value);
    const [rawMin, rawMax] = safeMinMax(values);
    let yMin = rawMin - 2, yMax = rawMax + 2;
    if (yMax - yMin < 4) { yMin--; yMax++; }
    areaRef.current = { pL, pR, pT, pB, cW, cH, yMin, yMax, dLen: data.length };

    const toX = (i: number, len: number) => pL + (i / (len - 1)) * cW;
    const toY = (v: number) => pT + cH - ((v - yMin) / (yMax - yMin)) * cH;

    ctx.clearRect(0, 0, W, H);

    // Grid
    ctx.strokeStyle = '#1a1a1e'; ctx.lineWidth = 1;
    for (let i = 0; i <= 5; i++) { const y = pT + (cH / 5) * i; ctx.beginPath(); ctx.moveTo(pL, y); ctx.lineTo(W - pR, y); ctx.stroke(); }

    // Y labels
    ctx.fillStyle = '#52525b'; ctx.font = '11px JetBrains Mono,monospace'; ctx.textAlign = 'right';
    for (let i = 0; i <= 5; i++) { const v = yMin + ((yMax - yMin) / 5) * (5 - i); ctx.fillText(v.toFixed(1), pL - 8, pT + (cH / 5) * i + 4); }

    // X labels — adaptive
    ctx.textAlign = 'center';
    const labelGap = Math.max(60, cW / 12);
    const pxStep = cW / (data.length - 1);
    const indexStep = Math.max(1, Math.ceil(labelGap / pxStep));
    for (let i = 0; i < data.length; i += indexStep) ctx.fillText(fmtTs(data[i].ts), toX(i, data.length), H - pB + 16);
    ctx.fillText(fmtTs(data[data.length - 1].ts), toX(data.length - 1, data.length), H - pB + 16);

    // Threshold lines
    ctx.setLineDash([6, 4]);
    if (warn >= yMin && warn <= yMax) { ctx.strokeStyle = 'rgba(234,179,8,0.4)'; ctx.lineWidth = 1; ctx.beginPath(); ctx.moveTo(pL, toY(warn)); ctx.lineTo(W - pR, toY(warn)); ctx.stroke(); ctx.fillStyle = 'rgba(234,179,8,0.7)'; ctx.font = '10px JetBrains Mono,monospace'; ctx.textAlign = 'left'; ctx.fillText('WARN', W - pR + 4, toY(warn) + 3); }
    if (crit >= yMin && crit <= yMax) { ctx.strokeStyle = 'rgba(239,68,68,0.4)'; ctx.lineWidth = 1; ctx.beginPath(); ctx.moveTo(pL, toY(crit)); ctx.lineTo(W - pR, toY(crit)); ctx.stroke(); ctx.fillStyle = 'rgba(239,68,68,0.7)'; ctx.textAlign = 'left'; ctx.fillText('CRIT', W - pR + 4, toY(crit) + 3); }
    ctx.setLineDash([]);

    // Overlays (normalized to chart area)
    for (const ov of validOverlays) {
      const ovVals = ov.data.map(d => d.value);
      const [ovMin, ovMax] = safeMinMax(ovVals);
      const ovRange = Math.max(ovMax - ovMin, 0.001);
      const ovToY = (v: number) => pT + cH - ((v - ovMin) / ovRange) * cH;
      ctx.beginPath();
      for (let i = 0; i < ov.data.length; i++) { const x = toX(i, ov.data.length), y = ovToY(ov.data[i].value); i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y); }
      ctx.strokeStyle = ov.color; ctx.lineWidth = 1.5; ctx.lineJoin = 'round'; ctx.lineCap = 'round'; ctx.stroke();
      const lx = toX(ov.data.length - 1, ov.data.length), ly = ovToY(ov.data[ov.data.length - 1].value);
      ctx.beginPath(); ctx.arc(lx, ly, 3, 0, Math.PI * 2); ctx.fillStyle = ov.color; ctx.fill();
    }

    // Primary line
    ctx.beginPath();
    for (let i = 0; i < data.length; i++) { const x = toX(i, data.length), y = toY(data[i].value); i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y); }
    ctx.strokeStyle = color; ctx.lineWidth = 2; ctx.lineJoin = 'round'; ctx.lineCap = 'round'; ctx.stroke();
    const lx = toX(data.length - 1, data.length);
    ctx.lineTo(lx, pT + cH); ctx.lineTo(toX(0, data.length), pT + cH); ctx.closePath();
    const g = ctx.createLinearGradient(0, pT, 0, pT + cH); g.addColorStop(0, color + '25'); g.addColorStop(1, color + '00'); ctx.fillStyle = g; ctx.fill();
    const lv = data[data.length - 1].value, cy = toY(lv);
    ctx.beginPath(); ctx.arc(lx, cy, 5, 0, Math.PI * 2); ctx.fillStyle = color; ctx.fill();
    ctx.beginPath(); ctx.arc(lx, cy, 9, 0, Math.PI * 2); ctx.strokeStyle = color + '40'; ctx.lineWidth = 2; ctx.stroke();
    ctx.fillStyle = color; ctx.font = 'bold 13px JetBrains Mono,monospace'; ctx.textAlign = 'right';
    ctx.fillText(lv.toFixed(1) + (unit ? ' ' + unit : ''), W - pR - 2, cy - 12);
  });

  // Auto-scroll to end
  useEffect(() => {
    if (pointWidth > 0 && scrollRef.current) {
      requestAnimationFrame(() => {
        if (!scrollRef.current) return;
        scrollRef.current.scrollLeft = scrollRef.current.scrollWidth - scrollRef.current.clientWidth;
      });
    }
  }, [pointWidth, data.length]);

  const onMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const cv = canvasRef.current; if (!cv || data.length < 2) { setHover(null); return; }
    const rect = cv.getBoundingClientRect(), mx = e.clientX - rect.left;
    const { pL, cW, dLen, pT, cH, yMin, yMax } = areaRef.current;
    const idx = Math.round(((mx - pL) / cW) * (dLen - 1));
    if (idx < 0 || idx >= dLen) { setHover(null); return; }
    const toY = (v: number) => pT + cH - ((v - yMin) / (yMax - yMin)) * cH;
    setHover({ idx, clientX: e.clientX, clientY: e.clientY, py: toY(data[idx].value) });
  };

  return (
    <div ref={scrollRef} style={{ position: 'relative', width: '100%', height: '100%', overflowX: 'auto', overflowY: 'hidden' }}>
      {/* Sizing wrapper: always fills height, width set by JS for scroll extent */}
      <div ref={wrapperRef} style={{ position: 'relative', height: '100%' }}>
        <canvas ref={canvasRef} style={{ display: 'block', width: '100%', height: '100%', cursor: 'crosshair' }} onMouseMove={onMove} onMouseLeave={() => setHover(null)} />
        {/* Legend */}
        {validOverlays.length > 0 && (
          <div style={{ position: 'absolute', top: 8, right: 24, display: 'flex', gap: 10, pointerEvents: 'none', zIndex: 1 }}>
            {validOverlays.map((ov, i) => (
              <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 4, fontSize: 11, fontFamily: "'JetBrains Mono',monospace", color: ov.color, opacity: 0.85, background: 'rgba(17,17,19,0.8)', padding: '2px 6px', borderRadius: 4 }}>
                <div style={{ width: 12, height: 2, background: ov.color, borderRadius: 1 }} />
                {ov.name} ({ov.unit})
              </div>
            ))}
          </div>
        )}
      </div>
      {/* Tooltip — fixed position relative to viewport, unaffected by scroll */}
      {hover && data[hover.idx] && (() => {
        const scrollEl = scrollRef.current;
        const scrollRect = scrollEl ? scrollEl.getBoundingClientRect() : null;
        if (!scrollRect) return null;
        const allSeries: { name: string; val: string; color: string }[] = [];
        allSeries.push({ name: 'Primary', val: data[hover.idx].value.toFixed(2) + (unit ? ' ' + unit : ''), color });
        for (let oi = 0; oi < validOverlays.length; oi++) {
          const ov = validOverlays[oi];
          const ovIdx = Math.min(Math.round((hover.idx / (data.length - 1)) * (ov.data.length - 1)), ov.data.length - 1);
          if (ov.data[ovIdx]) allSeries.push({ name: ov.name, val: ov.data[ovIdx].value.toFixed(2) + (ov.unit ? ' ' + ov.unit : ''), color: ov.color });
        }
        const tipW = 190, tipH = 28 + allSeries.length * 18;
        const vpX = hover.clientX - scrollRect.left;
        const vpY = hover.clientY - scrollRect.top;
        const tipLeft = vpX + 16 + tipW > scrollRect.width ? vpX - tipW - 12 : vpX + 16;
        const tipTop = Math.max(4, Math.min(vpY - tipH / 2, scrollRect.height - tipH - 4));
        return (<>
          <div style={{ position: 'fixed', left: hover.clientX, top: scrollRect.top, width: 1, height: scrollRect.height, background: 'rgba(255,255,255,0.12)', pointerEvents: 'none', zIndex: 9999 }} />
          <div style={{ position: 'fixed', left: hover.clientX - 5, top: hover.clientY - 5, width: 10, height: 10, borderRadius: '50%', background: color, border: '2px solid #fff', pointerEvents: 'none', zIndex: 9999 }} />
          <div style={{ position: 'fixed', left: scrollRect.left + tipLeft, top: scrollRect.top + tipTop, background: 'rgba(17,17,19,0.95)', border: '1px solid var(--border)', borderRadius: 8, padding: '8px 12px', pointerEvents: 'none', zIndex: 9999, fontFamily: "'JetBrains Mono',monospace", fontSize: 12, whiteSpace: 'nowrap' }}>
            {allSeries.map((s, i) => (
              <div key={i} style={{ color: s.color, fontWeight: i === 0 ? 700 : 500, fontSize: i === 0 ? 14 : 11, marginTop: i > 0 ? 2 : 0 }}>
                {allSeries.length > 1 && <span style={{ fontSize: 10, opacity: 0.7 }}>{s.name}: </span>}{s.val}
              </div>
            ))}
            <div style={{ color: 'var(--muted)', fontSize: 11, marginTop: 2 }}>{new Date(data[hover.idx].ts).toLocaleString('en-US', { hour12: false, month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit', second: '2-digit' })}</div>
          </div>
        </>);
      })()}
    </div>
  );
}
