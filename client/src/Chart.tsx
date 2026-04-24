import { useRef, useEffect } from 'react';

export function Chart({ data, color: rawColor, warn, crit }: { data: number[]; color: string; warn: number; crit: number }) {
  const color = rawColor.replace(/"/g, '');
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const cv = ref.current; if (!cv || data.length < 2) return;
    const ctx = cv.getContext('2d')!, dpr = devicePixelRatio || 1, rect = cv.getBoundingClientRect();
    cv.width = rect.width * dpr; cv.height = rect.height * dpr; ctx.scale(dpr, dpr);
    const W = rect.width, H = rect.height, pL = 40, pR = 10, pT = 10, pB = 24, cW = W - pL - pR, cH = H - pT - pB;
    ctx.clearRect(0, 0, W, H);
    ctx.strokeStyle = '#1a1a1e'; ctx.lineWidth = 1;
    for (let i = 0; i <= 4; i++) { const y = pT + (cH / 4) * i; ctx.beginPath(); ctx.moveTo(pL, y); ctx.lineTo(W - pR, y); ctx.stroke(); }
    let yMin = Math.min(...data) - 2, yMax = Math.max(...data) + 2; if (yMax - yMin < 4) { yMin--; yMax++; }
    const toX = (i: number) => pL + (i / (data.length - 1)) * cW;
    const toY = (v: number) => pT + cH - ((v - yMin) / (yMax - yMin)) * cH;
    ctx.fillStyle = '#52525b'; ctx.font = '10px JetBrains Mono,monospace'; ctx.textAlign = 'right';
    for (let i = 0; i <= 4; i++) { const v = yMin + ((yMax - yMin) / 4) * (4 - i); ctx.fillText(v.toFixed(1), pL - 6, pT + (cH / 4) * i + 3); }
    ctx.setLineDash([4, 4]);
    if (warn >= yMin && warn <= yMax) { ctx.strokeStyle = 'rgba(234,179,8,0.3)'; ctx.beginPath(); ctx.moveTo(pL, toY(warn)); ctx.lineTo(W - pR, toY(warn)); ctx.stroke(); }
    if (crit >= yMin && crit <= yMax) { ctx.strokeStyle = 'rgba(239,68,68,0.3)'; ctx.beginPath(); ctx.moveTo(pL, toY(crit)); ctx.lineTo(W - pR, toY(crit)); ctx.stroke(); }
    ctx.setLineDash([]);
    ctx.beginPath();
    for (let i = 0; i < data.length; i++) { const x = toX(i), y = toY(data[i]); i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y); }
    ctx.strokeStyle = color; ctx.lineWidth = 2; ctx.lineJoin = 'round'; ctx.lineCap = 'round'; ctx.stroke();
    const lx = toX(data.length - 1), fx = toX(0);
    ctx.lineTo(lx, pT + cH); ctx.lineTo(fx, pT + cH); ctx.closePath();
    const g = ctx.createLinearGradient(0, pT, 0, pT + cH); g.addColorStop(0, color + '30'); g.addColorStop(1, color + '00');
    ctx.fillStyle = g; ctx.fill();
    const lv = data[data.length - 1], cy = toY(lv);
    ctx.beginPath(); ctx.arc(lx, cy, 4, 0, Math.PI * 2); ctx.fillStyle = color; ctx.fill();
    ctx.beginPath(); ctx.arc(lx, cy, 7, 0, Math.PI * 2); ctx.strokeStyle = color + '40'; ctx.lineWidth = 2; ctx.stroke();
    ctx.fillStyle = color; ctx.font = 'bold 11px JetBrains Mono,monospace'; ctx.textAlign = 'right'; ctx.fillText(lv.toFixed(1), W - pR - 2, cy - 8);
  });
  return <canvas ref={ref} style={{ display: 'block', width: '100%', height: '100%' }} />;
}
