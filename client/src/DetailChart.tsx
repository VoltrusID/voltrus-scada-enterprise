import { useRef, useEffect, useState } from 'react';
import { DataPoint } from './types';
import { fmtTs, safeMinMax, toHexColor } from './styles';

export function DetailChart({ data, color: rawColor, warn, crit, unit }: { data: DataPoint[]; color: string; warn: number; crit: number; unit: string }) {
  const color = toHexColor(rawColor);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [hover, setHover] = useState<{ idx: number; px: number; py: number } | null>(null);
  const areaRef = useRef({ pL: 60, pR: 20, pT: 20, pB: 44, cW: 0, cH: 0, yMin: 0, yMax: 0, dLen: 0 });

  useEffect(() => {
    const cv = canvasRef.current; if (!cv || data.length < 2) return;
    const ctx = cv.getContext('2d')!, dpr = devicePixelRatio || 1, rect = cv.getBoundingClientRect();
    cv.width = rect.width * dpr; cv.height = rect.height * dpr; ctx.scale(dpr, dpr);
    const W = rect.width, H = rect.height, pL = 60, pR = 20, pT = 20, pB = 44, cW = W - pL - pR, cH = H - pT - pB;
    const values = data.map(d => d.value);
    const [rawMin, rawMax] = safeMinMax(values);
    let yMin = rawMin - 2, yMax = rawMax + 2; if (yMax - yMin < 4) { yMin--; yMax++; }
    areaRef.current = { pL, pR, pT, pB, cW, cH, yMin, yMax, dLen: data.length };
    const toX = (i: number) => pL + (i / (data.length - 1)) * cW;
    const toY = (v: number) => pT + cH - ((v - yMin) / (yMax - yMin)) * cH;
    ctx.clearRect(0, 0, W, H);
    ctx.strokeStyle = '#1a1a1e'; ctx.lineWidth = 1;
    for (let i = 0; i <= 5; i++) { const y = pT + (cH / 5) * i; ctx.beginPath(); ctx.moveTo(pL, y); ctx.lineTo(W - pR, y); ctx.stroke(); }
    ctx.fillStyle = '#52525b'; ctx.font = '11px JetBrains Mono,monospace'; ctx.textAlign = 'right';
    for (let i = 0; i <= 5; i++) { const v = yMin + ((yMax - yMin) / 5) * (5 - i); ctx.fillText(v.toFixed(1), pL - 8, pT + (cH / 5) * i + 4); }
    ctx.textAlign = 'center';
    const step = Math.max(1, Math.floor(data.length / 7));
    for (let i = 0; i < data.length; i += step) ctx.fillText(fmtTs(data[i].ts), toX(i), H - pB + 16);
    ctx.fillText(fmtTs(data[data.length - 1].ts), toX(data.length - 1), H - pB + 16);
    ctx.setLineDash([6, 4]);
    if (warn >= yMin && warn <= yMax) { ctx.strokeStyle = 'rgba(234,179,8,0.4)'; ctx.lineWidth = 1; ctx.beginPath(); ctx.moveTo(pL, toY(warn)); ctx.lineTo(W - pR, toY(warn)); ctx.stroke(); ctx.fillStyle = 'rgba(234,179,8,0.7)'; ctx.font = '10px JetBrains Mono,monospace'; ctx.textAlign = 'left'; ctx.fillText('WARN', W - pR + 4, toY(warn) + 3); }
    if (crit >= yMin && crit <= yMax) { ctx.strokeStyle = 'rgba(239,68,68,0.4)'; ctx.lineWidth = 1; ctx.beginPath(); ctx.moveTo(pL, toY(crit)); ctx.lineTo(W - pR, toY(crit)); ctx.stroke(); ctx.fillStyle = 'rgba(239,68,68,0.7)'; ctx.textAlign = 'left'; ctx.fillText('CRIT', W - pR + 4, toY(crit) + 3); }
    ctx.setLineDash([]);
    ctx.beginPath();
    for (let i = 0; i < data.length; i++) { const x = toX(i), y = toY(data[i].value); i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y); }
    ctx.strokeStyle = color; ctx.lineWidth = 2; ctx.lineJoin = 'round'; ctx.lineCap = 'round'; ctx.stroke();
    const lx = toX(data.length - 1);
    ctx.lineTo(lx, pT + cH); ctx.lineTo(toX(0), pT + cH); ctx.closePath();
    const g = ctx.createLinearGradient(0, pT, 0, pT + cH); g.addColorStop(0, color + '25'); g.addColorStop(1, color + '00'); ctx.fillStyle = g; ctx.fill();
    const lv = data[data.length - 1].value, cy = toY(lv);
    ctx.beginPath(); ctx.arc(lx, cy, 5, 0, Math.PI * 2); ctx.fillStyle = color; ctx.fill();
    ctx.beginPath(); ctx.arc(lx, cy, 9, 0, Math.PI * 2); ctx.strokeStyle = color + '40'; ctx.lineWidth = 2; ctx.stroke();
    ctx.fillStyle = color; ctx.font = 'bold 13px JetBrains Mono,monospace'; ctx.textAlign = 'right';
    ctx.fillText(lv.toFixed(1) + (unit ? ' ' + unit : ''), W - pR - 2, cy - 12);
  });

  const onMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const cv = canvasRef.current; if (!cv || data.length < 2) { setHover(null); return; }
    const rect = cv.getBoundingClientRect(), mx = e.clientX - rect.left;
    const { pL, cW, dLen, pT, cH, yMin, yMax } = areaRef.current;
    const idx = Math.round(((mx - pL) / cW) * (dLen - 1));
    if (idx < 0 || idx >= dLen) { setHover(null); return; }
    const toX = (i: number) => pL + (i / (dLen - 1)) * cW;
    const toY = (v: number) => pT + cH - ((v - yMin) / (yMax - yMin)) * cH;
    setHover({ idx, px: toX(idx), py: toY(data[idx].value) });
  };

  return (
    <div style={{ position: 'relative', width: '100%', height: '100%' }}>
      <canvas ref={canvasRef} style={{ display: 'block', width: '100%', height: '100%', cursor: 'crosshair' }} onMouseMove={onMove} onMouseLeave={() => setHover(null)} />
      {hover && data[hover.idx] && (() => {
        const a = areaRef.current, tipW = 170, tipH = 52;
        const tipLeft = hover.px + 16 + tipW > (a.pL + a.cW) ? hover.px - tipW - 12 : hover.px + 16;
        const tipTop = Math.max(a.pT, Math.min(hover.py - tipH / 2, a.pT + a.cH - tipH));
        return (<>
          <div style={{ position: 'absolute', left: hover.px, top: a.pT, width: 1, height: a.cH, background: 'rgba(255,255,255,0.12)', pointerEvents: 'none' }} />
          <div style={{ position: 'absolute', left: hover.px - 5, top: hover.py - 5, width: 10, height: 10, borderRadius: '50%', background: color, border: '2px solid #fff', pointerEvents: 'none', zIndex: 2 }} />
          <div style={{ position: 'absolute', left: tipLeft, top: tipTop, background: 'rgba(17,17,19,0.95)', border: '1px solid var(--border)', borderRadius: 8, padding: '8px 12px', pointerEvents: 'none', zIndex: 3, fontFamily: "'JetBrains Mono',monospace", fontSize: 12, whiteSpace: 'nowrap' }}>
            <div style={{ color, fontWeight: 700, fontSize: 14 }}>{data[hover.idx].value.toFixed(2)}{unit ? ' ' + unit : ''}</div>
            <div style={{ color: 'var(--muted)', fontSize: 11, marginTop: 2 }}>{new Date(data[hover.idx].ts).toLocaleString('en-US', { hour12: false, month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit', second: '2-digit' })}</div>
          </div>
        </>);
      })()}
    </div>
  );
}
