import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { btnStyle, inputStyle, labelStyle, labelTextStyle } from '@voltrus/core/styles'
import { useRole } from '@voltrus/core/RoleContext'
import {
  useDeleteReconcile, useEnergyBill, useEnergyDaily, useEnergySite, useEnergySites, useReconcile, useRecompute,
} from '../queries/energy'
import {
  dailyScale, formatNumber, formatPct, formatRupiah, monthOf, numOrNull, showUnconfirmedNote,
  type Bill, type DayRow, type MonthReport, type PlnFigures,
} from '../energy/model'
import { cardStyle } from '../energy/styles'
import { EnergySetup, SiteForm } from './EnergySetup'

const thStyle: React.CSSProperties = {
  textAlign: 'left', padding: '6px 10px', fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase',
  letterSpacing: '.05em', fontWeight: 600, borderBottom: '1px solid var(--border)',
}
const tdStyle: React.CSSProperties = { padding: '6px 10px', fontSize: 13, borderBottom: '1px solid var(--border)' }
const numStyle: React.CSSProperties = { ...tdStyle, textAlign: 'right', fontVariantNumeric: 'tabular-nums' }
const sectionTitle: React.CSSProperties = { fontSize: 14, fontWeight: 600, marginBottom: 10 }

function useLang() {
  const { t, i18n } = useTranslation('enterprise')
  return { t, lang: i18n.resolvedLanguage ?? i18n.language ?? 'en' }
}

/** "Tagihan PLN": the shadow PLN bill of a site. */
export function EnergyPanel() {
  const { t } = useLang()
  const role = useRole()
  const sites = useEnergySites()
  const [picked, setPicked] = useState<number | null>(null)
  const [month, setMonth] = useState(monthOf(new Date()))
  const [view, setView] = useState<'bill' | 'setup' | 'new'>('bill')
  const list = sites.data ?? []
  const siteId = picked ?? list[0]?.site.id ?? null

  if (sites.isLoading) return <div style={{ padding: 24, color: 'var(--muted)' }}>{t('energy.loading')}</div>
  if (sites.error) return <div style={{ padding: 24, color: 'var(--crit)' }}>{String(sites.error)}</div>

  const tab = (id: typeof view, label: string) => (
    <button type="button" onClick={() => setView(id)} aria-pressed={view === id}
      style={{ ...btnStyle, borderColor: view === id ? 'var(--green)' : 'var(--border)', color: view === id ? 'var(--green)' : undefined }}>
      {label}
    </button>
  )

  return (
    <div style={{ padding: 24, display: 'flex', flexDirection: 'column', gap: 16, maxWidth: 1200 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, flexWrap: 'wrap' }}>
        <h1 style={{ fontSize: 20, fontWeight: 700, marginRight: 'auto' }}>{t('energy.title')}</h1>
        {list.length > 0 && (
          <>
            <select aria-label={t('energy.site')} value={siteId ?? ''} style={{ ...inputStyle, width: 200 }}
              onChange={e => { setPicked(Number(e.target.value)); if (view === 'new') setView('bill') }}>
              {list.map(s => <option key={s.site.id} value={s.site.id}>{s.site.name}</option>)}
            </select>
            <input type="month" aria-label={t('energy.month')} value={month} style={{ ...inputStyle, width: 160 }}
              onChange={e => e.target.value && setMonth(e.target.value)} />
            {tab('bill', t('energy.tabBill'))}
            {role.canConfigure && tab('setup', t('energy.tabSetup'))}
          </>
        )}
        {role.canConfigure && list.length > 0 && tab('new', t('energy.newSite'))}
      </div>

      {list.length === 0 || view === 'new' ? (
        role.canConfigure ? (
          <div style={cardStyle}>
            <div style={sectionTitle}>{t('energy.newSite')}</div>
            {list.length === 0 && <p style={{ color: 'var(--muted)', fontSize: 13, marginBottom: 12 }}>{t('energy.noSites')}</p>}
            <SiteForm onSaved={id => { setPicked(id); setView('setup') }} />
          </div>
        ) : (
          <div style={{ ...cardStyle, color: 'var(--muted)' }}>{t('energy.noSitesReadOnly')}</div>
        )
      ) : siteId != null && view === 'setup' ? (
        <EnergySetup siteId={siteId} month={month} />
      ) : siteId != null ? (
        <BillView siteId={siteId} month={month} />
      ) : null}
    </div>
  )
}

function BillView({ siteId, month }: { siteId: number; month: string }) {
  const { t, lang } = useLang()
  const role = useRole()
  const bill = useEnergyBill(siteId, month)
  const daily = useEnergyDaily(siteId, month)
  const site = useEnergySite(siteId)
  const recompute = useRecompute()
  const report = bill.data

  if (bill.isLoading) return <div style={{ color: 'var(--muted)' }}>{t('energy.loading')}</div>
  if (bill.error || !report) return <div style={{ color: 'var(--crit)' }}>{String(bill.error ?? '')}</div>
  const noMeters = (site.data?.meters ?? []).filter(m => m.role === 'pln_incomer' && m.enabled).length === 0

  return (
    <>
      {showUnconfirmedNote(report, site.data) && (
        <div role="note" style={{ ...cardStyle, borderColor: 'var(--warn)', fontSize: 13 }}>
          <strong>{t('energy.unconfirmedTitle')}</strong> {t('energy.unconfirmedBody')}
        </div>
      )}
      <div style={{ fontSize: 12, color: 'var(--muted)' }}>{t('energy.excluded')}</div>
      {noMeters && <div style={{ ...cardStyle, color: 'var(--muted)', fontSize: 13 }}>{t('energy.noIncomer')}</div>}

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(340px, 1fr))', gap: 16 }}>
        <BillCard title={report.complete ? t('energy.monthTotal') : t('energy.monthToDate')} bill={report.month_to_date} />
        {!report.complete && <BillCard title={t('energy.projected')} bill={report.projected} projected />}
      </div>

      <div style={{ fontSize: 12, color: 'var(--muted)', display: 'flex', gap: 16, flexWrap: 'wrap', alignItems: 'center' }}>
        <span>{t('energy.hoursWithData', {
          seen: report.hours_with_data.lwbp + report.hours_with_data.wbp,
          total: report.hours_in_month.lwbp + report.hours_in_month.wbp,
        })}</span>
        <span>{t('energy.qualityCounts', { ...report.quality })}</span>
        {report.tariff && <span>{t('energy.tariffLine', {
          golongan: report.tariff.golongan, from: report.tariff.valid_from,
          k: report.tariff.golongan === 'I-3' ? formatNumber(report.tariff.k_factor ?? 1.4, lang, 2) : '-',
        })}</span>}
        {role.canConfigure && (
          <button type="button" style={{ ...btnStyle, padding: '2px 8px', fontSize: 11 }} disabled={recompute.isPending}
            onClick={() => recompute.mutate({ siteId, month })}>
            {recompute.isPending ? t('energy.recomputing') : t('energy.recompute')}
          </button>
        )}
      </div>

      <DailySection days={daily.data?.days ?? []} />
      <ReconcileSection report={report} siteId={siteId} month={month} />
    </>
  )
}

function BillCard({ title, bill, projected }: { title: string; bill: Bill | null; projected?: boolean }) {
  const { t, lang } = useLang()
  if (!bill) return <div style={cardStyle}><div style={sectionTitle}>{title}</div><div style={{ color: 'var(--muted)', fontSize: 13 }}>{t('energy.noData')}</div></div>
  const n = (v: number | null, d = 0) => formatNumber(v, lang, d)
  const rp = (v: number | null) => formatRupiah(v, lang)
  const rows: [string, string, string][] = [
    [t('energy.lineLwbp'), `${n(bill.kwh_lwbp)} kWh × ${rp(bill.rate_lwbp)}`, rp(bill.cost_lwbp)],
    [t('energy.lineWbp'), `${n(bill.kwh_wbp)} kWh × ${formatRupiah(bill.rate_wbp, lang)}${bill.k_factor != null ? ` (K ${n(bill.k_factor, 2)})` : ''}`, rp(bill.cost_wbp)],
    [t('energy.lineKvarh'), bill.kvarh == null ? t('energy.noKvarh')
      : `${n(bill.kvarh_excess)} kVArh × ${rp(bill.rate_kvarh)}`, rp(bill.cost_kvarh)],
  ]
  return (
    <div style={cardStyle}>
      <div style={{ ...sectionTitle, display: 'flex', justifyContent: 'space-between' }}>
        <span>{title}</span>
        {projected && <span style={{ fontSize: 11, color: 'var(--muted)', fontWeight: 400 }}>{t('energy.estimate')}</span>}
      </div>
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <tbody>
          {rows.map(([label, detail, amount]) => (
            <tr key={label}>
              <td style={tdStyle}>{label}<div style={{ fontSize: 11, color: 'var(--muted)' }}>{detail}</div></td>
              <td style={numStyle}>{amount}</td>
            </tr>
          ))}
          <tr>
            <td style={{ ...tdStyle, fontWeight: 700 }}>{t('energy.total')}</td>
            <td style={{ ...numStyle, fontWeight: 700 }}>{rp(bill.total)}</td>
          </tr>
        </tbody>
      </table>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 8, marginTop: 12, fontSize: 12 }}>
        <Stat label={t('energy.wbpPremium')} value={rp(bill.wbp_premium)} />
        <Stat label="cos φ" value={bill.cos_phi == null ? '-' : n(bill.cos_phi, 3)} />
        <Stat label={t('energy.kvarhTotal')} value={bill.kvarh == null ? '-' : `${n(bill.kvarh)} / ${n(bill.kvarh_allowed)}`}
          hint={t('energy.kvarhAllowedHint', { factor: n(bill.kvarh_factor, 4) })} />
      </div>
    </div>
  )
}

function Stat({ label, value, hint }: { label: string; value: string; hint?: string }) {
  return (
    <div title={hint}>
      <div style={{ color: 'var(--muted)', fontSize: 11 }}>{label}</div>
      <div style={{ fontWeight: 600, fontVariantNumeric: 'tabular-nums' }}>{value}</div>
    </div>
  )
}

// Categorical slots 1 and 2 of the dataviz reference palette (validated pair), per theme.
function seriesColors(): [string, string] {
  const dark = typeof document !== 'undefined' && document.documentElement.getAttribute('data-theme') !== 'light'
  return dark ? ['#3987e5', '#d95926'] : ['#2a78d6', '#eb6834']
}

function DailySection({ days }: { days: DayRow[] }) {
  const { t, lang } = useLang()
  const [showTable, setShowTable] = useState(false)
  const [premiumColor, kvarhColor] = seriesColors()
  const top = dailyScale(days)
  const W = 720, H = 180, pad = { l: 8, r: 8, t: 8, b: 20 }
  const slot = days.length > 0 ? (W - pad.l - pad.r) / days.length : 0
  const bar = Math.max(2, Math.min(10, slot / 2 - 2))
  const y = (v: number) => pad.t + (H - pad.t - pad.b) * (1 - v / top)

  return (
    <div style={cardStyle}>
      <div style={{ ...sectionTitle, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span>{t('energy.dailyTitle')}</span>
        <button type="button" style={{ ...btnStyle, padding: '2px 8px', fontSize: 11 }} onClick={() => setShowTable(s => !s)}>
          {showTable ? t('energy.showChart') : t('energy.showTable')}
        </button>
      </div>
      {days.length === 0 ? (
        <div style={{ color: 'var(--muted)', fontSize: 13 }}>{t('energy.noData')}</div>
      ) : showTable ? (
        <div style={{ overflow: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead><tr>
              <th style={thStyle}>{t('energy.date')}</th>
              <th style={{ ...thStyle, textAlign: 'right' }}>kWh LWBP</th>
              <th style={{ ...thStyle, textAlign: 'right' }}>kWh WBP</th>
              <th style={{ ...thStyle, textAlign: 'right' }}>kVArh</th>
              <th style={{ ...thStyle, textAlign: 'right' }}>cos φ</th>
              <th style={{ ...thStyle, textAlign: 'right' }}>{t('energy.wbpPremium')}</th>
              <th style={{ ...thStyle, textAlign: 'right' }}>{t('energy.kvarhCharge')}</th>
              <th style={{ ...thStyle, textAlign: 'right' }}>{t('energy.total')}</th>
              <th style={thStyle}>{t('energy.quality')}</th>
            </tr></thead>
            <tbody>
              {days.map(d => (
                <tr key={d.date}>
                  <td style={tdStyle}>{d.date}</td>
                  <td style={numStyle}>{formatNumber(d.kwh_lwbp, lang)}</td>
                  <td style={numStyle}>{formatNumber(d.kwh_wbp, lang)}</td>
                  <td style={numStyle}>{formatNumber(d.kvarh, lang)}</td>
                  <td style={numStyle}>{formatNumber(d.cos_phi, lang, 3)}</td>
                  <td style={numStyle}>{formatRupiah(d.wbp_premium, lang)}</td>
                  <td style={numStyle}>{formatRupiah(d.kvarh_charge, lang)}</td>
                  <td style={numStyle}>{formatRupiah(d.cost_total, lang)}</td>
                  <td style={{ ...tdStyle, fontSize: 11, color: 'var(--muted)' }}>
                    {d.quality.ok}/{d.quality.estimated}/{d.quality.gap}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <div style={{ fontSize: 11, color: 'var(--muted)', marginTop: 6 }}>{t('energy.qualityLegend')}</div>
        </div>
      ) : (
        <>
          <div style={{ display: 'flex', gap: 16, fontSize: 12, marginBottom: 6 }}>
            <Legend color={premiumColor} label={t('energy.wbpPremium')} />
            <Legend color={kvarhColor} label={t('energy.kvarhCharge')} />
            <span style={{ marginLeft: 'auto', color: 'var(--muted)' }}>{t('energy.axisTop', { v: formatRupiah(top, lang) })}</span>
          </div>
          <svg viewBox={`0 0 ${W} ${H}`} width="100%" role="img" aria-label={t('energy.dailyTitle')} style={{ display: 'block' }}>
            {[0.5, 1].map(f => (
              <line key={f} x1={pad.l} x2={W - pad.r} y1={y(top * f)} y2={y(top * f)} stroke="var(--border)" strokeWidth={1} />
            ))}
            <line x1={pad.l} x2={W - pad.r} y1={y(0)} y2={y(0)} stroke="var(--muted)" strokeWidth={1} />
            {days.map((d, i) => {
              const cx = pad.l + slot * i + slot / 2
              const day = d.date.slice(8)
              const tip = `${d.date}\n${t('energy.wbpPremium')}: ${formatRupiah(d.wbp_premium, lang)}\n${t('energy.kvarhCharge')}: ${formatRupiah(d.kvarh_charge, lang)}\ncos φ ${formatNumber(d.cos_phi, lang, 3)}`
              return (
                <g key={d.date}>
                  <title>{tip}</title>
                  <rect x={cx - slot / 2} y={pad.t} width={slot} height={H - pad.t - pad.b} fill="transparent" />
                  <rect x={cx - bar - 1} y={y(d.wbp_premium)} width={bar} height={Math.max(0, y(0) - y(d.wbp_premium))} rx={2} fill={premiumColor} />
                  <rect x={cx + 1} y={y(d.kvarh_charge ?? 0)} width={bar} height={Math.max(0, y(0) - y(d.kvarh_charge ?? 0))} rx={2} fill={kvarhColor} />
                  {(i % Math.ceil(days.length / 15) === 0) && (
                    <text x={cx} y={H - 6} textAnchor="middle" fontSize={10} fill="var(--muted)">{day}</text>
                  )}
                </g>
              )
            })}
          </svg>
          <div style={{ fontSize: 11, color: 'var(--muted)', marginTop: 6 }}>{t('energy.dailyKvarhNote')}</div>
        </>
      )}
    </div>
  )
}

function Legend({ color, label }: { color: string; label: string }) {
  return (
    <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6 }}>
      <span style={{ width: 10, height: 10, borderRadius: 2, background: color, display: 'inline-block' }} />{label}
    </span>
  )
}

const RECONCILE_FIELDS: { key: keyof PlnFigures; required?: boolean }[] = [
  { key: 'kwh_lwbp', required: true }, { key: 'kwh_wbp', required: true }, { key: 'kvarh', required: true },
  { key: 'kvarh_excess' }, { key: 'cost_lwbp' }, { key: 'cost_wbp' }, { key: 'cost_kvarh' }, { key: 'total' },
]

function ReconcileSection({ report, siteId, month }: { report: MonthReport; siteId: number; month: string }) {
  const { t, lang } = useLang()
  const role = useRole()
  const save = useReconcile()
  const remove = useDeleteReconcile()
  const [editing, setEditing] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const rec = report.reconciliation

  const submit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const fd = new FormData(e.currentTarget)
    const figures: Record<string, unknown> = { note: String(fd.get('note') ?? '') }
    for (const f of RECONCILE_FIELDS) {
      const v = numOrNull(String(fd.get(f.key) ?? ''))
      if (f.required && v == null) { setError(t('energy.reconcileRequired')); return }
      figures[f.key] = v
    }
    setError(null)
    save.mutate({ siteId, month, figures: figures as unknown as PlnFigures }, {
      onSuccess: () => setEditing(false),
      onError: err => setError(err instanceof Error ? err.message : String(err)),
    })
  }

  return (
    <div style={cardStyle}>
      <div style={{ ...sectionTitle, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span>{t('energy.reconcileTitle', { month })}</span>
        {role.canConfigure && !editing && (
          <span style={{ display: 'flex', gap: 6 }}>
            <button type="button" style={{ ...btnStyle, padding: '2px 8px', fontSize: 11 }} onClick={() => setEditing(true)}>
              {report.pln ? t('energy.reconcileEdit') : t('energy.reconcileEnter')}
            </button>
            {report.pln && (
              <button type="button" style={{ ...btnStyle, padding: '2px 8px', fontSize: 11, color: 'var(--crit)' }}
                onClick={() => confirm(t('energy.reconcileDeleteConfirm')) && remove.mutate({ siteId, month })}>
                {t('energy.delete')}
              </button>
            )}
          </span>
        )}
      </div>
      {editing ? (
        <form onSubmit={submit} style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: 10 }}>
          {RECONCILE_FIELDS.map(f => (
            <label key={f.key} style={labelStyle}>
              <span style={labelTextStyle}>{t(`energy.pln.${f.key}`)}{f.required ? ' *' : ''}</span>
              <input name={f.key} inputMode="decimal" style={inputStyle}
                defaultValue={report.pln?.[f.key] != null ? String(report.pln[f.key]) : ''} />
            </label>
          ))}
          <label style={{ ...labelStyle, gridColumn: '1 / -1' }}>
            <span style={labelTextStyle}>{t('energy.note')}</span>
            <input name="note" style={inputStyle} defaultValue={report.pln?.note ?? ''} />
          </label>
          <div style={{ gridColumn: '1 / -1', fontSize: 11, color: 'var(--muted)' }}>{t('energy.reconcileHelp')}</div>
          {error && <div role="alert" style={{ gridColumn: '1 / -1', color: 'var(--crit)', fontSize: 12 }}>{error}</div>}
          <div style={{ gridColumn: '1 / -1', display: 'flex', gap: 8 }}>
            <button type="submit" style={btnStyle} disabled={save.isPending}>{t('energy.save')}</button>
            <button type="button" style={btnStyle} onClick={() => setEditing(false)}>{t('energy.cancel')}</button>
          </div>
        </form>
      ) : !rec ? (
        <div style={{ color: 'var(--muted)', fontSize: 13 }}>{report.complete ? t('energy.reconcileNone') : t('energy.reconcileAfterMonth')}</div>
      ) : (
        <>
          <div style={{ fontSize: 13, marginBottom: 8, color: rec.within_tolerance ? 'var(--ok)' : 'var(--warn)' }}>
            {rec.within_tolerance ? t('energy.reconcileWithin') : t('energy.reconcileOutside')}
          </div>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead><tr>
              <th style={thStyle}>{t('energy.line')}</th>
              <th style={{ ...thStyle, textAlign: 'right' }}>{t('energy.ours')}</th>
              <th style={{ ...thStyle, textAlign: 'right' }}>PLN</th>
              <th style={{ ...thStyle, textAlign: 'right' }}>{t('energy.difference')}</th>
              <th style={{ ...thStyle, textAlign: 'right' }}>%</th>
            </tr></thead>
            <tbody>
              {rec.lines.map(l => {
                const money = l.line.startsWith('cost') || l.line === 'total'
                const fmt = (v: number | null) => money ? formatRupiah(v, lang) : formatNumber(v, lang)
                return (
                  <tr key={l.line}>
                    <td style={{ ...tdStyle, fontWeight: l.line === 'total' ? 700 : 400 }}>{t(`energy.pln.${l.line}`)}</td>
                    <td style={numStyle}>{fmt(l.computed)}</td>
                    <td style={numStyle}>{fmt(l.pln)}</td>
                    <td style={numStyle}>{fmt(l.difference)}</td>
                    <td style={numStyle}>{formatPct(l.difference_pct, lang)}</td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </>
      )}
    </div>
  )
}
