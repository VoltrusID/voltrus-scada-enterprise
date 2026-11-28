import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { btnStyle, inputStyle, labelStyle, labelTextStyle, monoInputStyle } from '@voltrus/core/styles'
import { useSensors } from '@voltrus/core/queries/sensors'
import {
  useCreateSite, useDeleteMeter, useDeleteSite, useDeleteTariff, useEnergySite, useSaveMeter, useSetTariff, useUpdateSite,
  type MeterInput, type SiteInput,
} from '../queries/energy'
import {
  DEFAULT_TARIFFS, TIMEZONES, deviceTags, excessFactor, formatNumber, formatRupiah, numOrNull,
  type AlertConfig, type Golongan, type Meter, type Site, type SiteDetail, type WbpWindow,
} from '../energy/model'
import { cardStyle } from '../energy/styles'

const sectionTitle: React.CSSProperties = { fontSize: 14, fontWeight: 600, marginBottom: 10 }
const grid: React.CSSProperties = { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: 10 }
const smallBtn: React.CSSProperties = { ...btnStyle, padding: '2px 8px', fontSize: 11 }

function useLang() {
  const { t, i18n } = useTranslation('enterprise')
  return { t, lang: i18n.resolvedLanguage ?? i18n.language ?? 'en' }
}

function Field({ label, children, wide, hint }: { label: string; children: React.ReactNode; wide?: boolean; hint?: string }) {
  return (
    <label style={{ ...labelStyle, ...(wide ? { gridColumn: '1 / -1' } : {}) }}>
      <span style={labelTextStyle}>{label}</span>
      {children}
      {hint && <span style={{ fontSize: 11, color: 'var(--muted)' }}>{hint}</span>}
    </label>
  )
}

function ErrorLine({ error }: { error: unknown }) {
  if (!error) return null
  return <div role="alert" style={{ gridColumn: '1 / -1', color: 'var(--crit)', fontSize: 12 }}>{error instanceof Error ? error.message : String(error)}</div>
}

const DEFAULT_ALERTS: AlertConfig = { pf_warn: 0.87, pf_window_hours: 24, wbp_kw_target: null, daily_summary: true, lang: 'id' }

/** Tariff fields; `golongan` switches the defaults. */
function TariffFields({ golongan, onGolongan, initial }: {
  golongan: Golongan
  onGolongan: (g: Golongan) => void
  initial?: Partial<typeof DEFAULT_TARIFFS['I-3']>
}) {
  const { t, lang } = useLang()
  const d = { ...DEFAULT_TARIFFS[golongan], ...(initial?.golongan === golongan ? initial : {}) }
  return (
    <>
      <Field label={t('energy.golongan')}>
        <select name="golongan" value={golongan} style={inputStyle} onChange={e => onGolongan(e.target.value as Golongan)}>
          <option value="I-3">I-3 / TM (&gt; 200 kVA)</option>
          <option value="I-4">I-4 / TT (≥ 30.000 kVA)</option>
        </select>
      </Field>
      {golongan === 'I-3' && (
        <Field label="K" hint={t('energy.kHint')}>
          <input key={`k-${golongan}`} name="k_factor" inputMode="decimal" style={inputStyle} defaultValue={String(d.k_factor ?? 1.4)} />
        </Field>
      )}
      <Field label={t('energy.rateLwbp')}>
        <input key={`l-${golongan}`} name="rate_lwbp" inputMode="decimal" style={inputStyle} defaultValue={String(d.rate_lwbp)} />
      </Field>
      {golongan === 'I-4' && (
        <Field label={t('energy.rateWbp')}>
          <input key={`w-${golongan}`} name="rate_wbp" inputMode="decimal" style={inputStyle} defaultValue={String(d.rate_wbp ?? d.rate_lwbp)} />
        </Field>
      )}
      <Field label={t('energy.rateKvarh')}>
        <input key={`v-${golongan}`} name="rate_kvarh" inputMode="decimal" style={inputStyle} defaultValue={String(d.rate_kvarh)} />
      </Field>
      <Field label={t('energy.pfLimit')}>
        <input key={`p-${golongan}`} name="pf_limit" inputMode="decimal" style={inputStyle} defaultValue={String(d.pf_limit)} />
      </Field>
      <Field label={t('energy.kvarhFactor')} hint={t('energy.kvarhFactorHint', { v: formatNumber(excessFactor(d.pf_limit), lang, 4) })}>
        <input key={`f-${golongan}`} name="kvarh_factor" inputMode="decimal" style={inputStyle} placeholder="tan(acos(PF))"
          defaultValue={d.kvarh_factor != null ? String(d.kvarh_factor) : ''} />
      </Field>
    </>
  )
}

function readTariff(fd: FormData, golongan: Golongan) {
  const num = (k: string) => numOrNull(String(fd.get(k) ?? ''))
  return {
    golongan,
    k_factor: golongan === 'I-3' ? num('k_factor') : null,
    rate_lwbp: num('rate_lwbp') ?? NaN,
    rate_wbp: golongan === 'I-4' ? num('rate_wbp') : null,
    rate_kvarh: num('rate_kvarh') ?? NaN,
    pf_limit: num('pf_limit') ?? 0.85,
    kvarh_factor: num('kvarh_factor'),
  }
}

function readWindows(text: string): WbpWindow[] {
  return text.split(',').map(s => s.trim()).filter(Boolean).map(s => {
    const [start, end] = s.split('-').map(x => x.trim())
    return { start: start ?? '', end: end ?? '' }
  })
}

/** New site (with its first tariff) or the settings of an existing one. */
export function SiteForm({ site, onSaved }: { site?: Site; onSaved?: (id: number) => void }) {
  const { t } = useLang()
  const create = useCreateSite()
  const update = useUpdateSite()
  const [golongan, setGolongan] = useState<Golongan>('I-3')
  const alerts = site?.alerts ?? DEFAULT_ALERTS
  const mut = site ? update : create

  const submit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const fd = new FormData(e.currentTarget)
    const input: SiteInput = {
      name: String(fd.get('name') ?? ''),
      timezone: String(fd.get('timezone') ?? 'Asia/Jakarta'),
      wbp_windows: readWindows(String(fd.get('wbp') ?? '')),
      contracted_kva: numOrNull(String(fd.get('kva') ?? '')),
      alerts: {
        pf_warn: numOrNull(String(fd.get('pf_warn') ?? '')) ?? 0.87,
        pf_window_hours: numOrNull(String(fd.get('pf_window_hours') ?? '')) ?? 24,
        wbp_kw_target: numOrNull(String(fd.get('wbp_kw_target') ?? '')),
        daily_summary: fd.get('daily_summary') === 'on',
        lang: (String(fd.get('lang') ?? 'id') === 'en' ? 'en' : 'id'),
      },
    }
    if (site) {
      update.mutate({ id: site.id, ...input })
    } else {
      input.tariff = { ...readTariff(fd, golongan), valid_from: String(fd.get('valid_from') ?? '') }
      create.mutate(input, { onSuccess: d => onSaved?.(d.site.id) })
    }
  }

  return (
    <form onSubmit={submit} style={grid}>
      <Field label={t('energy.siteName')}><input name="name" required style={inputStyle} defaultValue={site?.name ?? ''} /></Field>
      <Field label={t('energy.timezone')}>
        <select name="timezone" style={inputStyle} defaultValue={site?.timezone ?? 'Asia/Jakarta'}>
          {TIMEZONES.map(z => <option key={z} value={z}>{t(`energy.tz.${z}`)}</option>)}
        </select>
      </Field>
      <Field label={t('energy.wbpWindows')} hint={t('energy.wbpHint')}>
        <input name="wbp" style={monoInputStyle} defaultValue={(site?.wbp_windows ?? [{ start: '18:00', end: '22:00' }]).map(w => `${w.start}-${w.end}`).join(', ')} />
      </Field>
      <Field label={t('energy.contractedKva')} hint={t('energy.contractedKvaHint')}>
        <input name="kva" inputMode="decimal" style={inputStyle} defaultValue={site?.contracted_kva != null ? String(site.contracted_kva) : ''} />
      </Field>
      {!site && (
        <>
          <div style={{ gridColumn: '1 / -1', ...sectionTitle, marginBottom: 0, marginTop: 6 }}>{t('energy.tariff')}</div>
          <Field label={t('energy.validFrom')}>
            <input name="valid_from" type="month" required style={inputStyle} defaultValue={new Date().toISOString().slice(0, 7)} />
          </Field>
          <TariffFields golongan={golongan} onGolongan={setGolongan} />
        </>
      )}
      <div style={{ gridColumn: '1 / -1', ...sectionTitle, marginBottom: 0, marginTop: 6 }}>{t('energy.alerts')}</div>
      <Field label={t('energy.pfWarn')} hint={t('energy.pfWarnHint')}>
        <input name="pf_warn" inputMode="decimal" style={inputStyle} defaultValue={String(alerts.pf_warn)} />
      </Field>
      <Field label={t('energy.pfWindow')}>
        <input name="pf_window_hours" inputMode="numeric" style={inputStyle} defaultValue={String(alerts.pf_window_hours)} />
      </Field>
      <Field label={t('energy.wbpKwTarget')} hint={t('energy.wbpKwTargetHint')}>
        <input name="wbp_kw_target" inputMode="decimal" style={inputStyle} defaultValue={alerts.wbp_kw_target != null ? String(alerts.wbp_kw_target) : ''} />
      </Field>
      <Field label={t('energy.summaryLang')}>
        <select name="lang" style={inputStyle} defaultValue={alerts.lang}>
          <option value="id">Bahasa Indonesia</option>
          <option value="en">English</option>
        </select>
      </Field>
      <label style={{ gridColumn: '1 / -1', display: 'flex', gap: 8, alignItems: 'center', fontSize: 13 }}>
        <input type="checkbox" name="daily_summary" defaultChecked={alerts.daily_summary} />{t('energy.dailySummary')}
      </label>
      <ErrorLine error={mut.error} />
      <div style={{ gridColumn: '1 / -1' }}>
        <button type="submit" style={btnStyle} disabled={mut.isPending}>{site ? t('energy.save') : t('energy.createSite')}</button>
        {update.isSuccess && site && <span style={{ marginLeft: 8, fontSize: 12, color: 'var(--ok)' }}>{t('energy.saved')}</span>}
      </div>
    </form>
  )
}

export function EnergySetup({ siteId }: { siteId: number; month: string }) {
  const { t } = useLang()
  const detail = useEnergySite(siteId)
  const del = useDeleteSite()
  if (detail.isLoading) return <div style={{ color: 'var(--muted)' }}>{t('energy.loading')}</div>
  if (!detail.data) return <div style={{ color: 'var(--crit)' }}>{String(detail.error ?? '')}</div>
  const d = detail.data
  return (
    <>
      <div style={cardStyle}>
        <div style={sectionTitle}>{t('energy.siteSettings')}</div>
        <SiteForm key={d.site.id} site={d.site} />
      </div>
      <TariffSection detail={d} />
      <MeterSection detail={d} />
      <div style={cardStyle}>
        <div style={sectionTitle}>{t('energy.notifications')}</div>
        <p style={{ fontSize: 13, color: 'var(--muted)', marginBottom: 8 }}>{t('energy.notificationsHelp')}</p>
        <ul style={{ fontSize: 13, paddingLeft: 18 }}>
          <li><code>{d.notification_keys.pf}</code>: {t('energy.keyPf')}</li>
          <li><code>{d.notification_keys.wbp_kw}</code>: {t('energy.keyWbp')}</li>
          <li><code>{d.notification_keys.daily}</code>: {t('energy.keyDaily')}</li>
        </ul>
      </div>
      <div>
        <button type="button" style={{ ...btnStyle, color: 'var(--crit)', borderColor: 'var(--crit)' }}
          onClick={() => confirm(t('energy.deleteSiteConfirm', { name: d.site.name })) && del.mutate(d.site.id)}>
          {t('energy.deleteSite')}
        </button>
      </div>
    </>
  )
}

function TariffSection({ detail }: { detail: SiteDetail }) {
  const { t, lang } = useLang()
  const set = useSetTariff()
  const del = useDeleteTariff()
  const current = detail.current_tariff
  const [golongan, setGolongan] = useState<Golongan>(current?.golongan ?? 'I-3')
  const [open, setOpen] = useState(false)

  const submit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const fd = new FormData(e.currentTarget)
    set.mutate({
      siteId: detail.site.id, ...readTariff(fd, golongan),
      valid_from: String(fd.get('valid_from') ?? ''), note: String(fd.get('note') ?? ''),
    }, { onSuccess: () => setOpen(false) })
  }

  return (
    <div style={cardStyle}>
      <div style={{ ...sectionTitle, display: 'flex', justifyContent: 'space-between' }}>
        <span>{t('energy.tariffs')}</span>
        {!open && <button type="button" style={smallBtn} onClick={() => setOpen(true)}>{t('energy.newTariff')}</button>}
      </div>
      <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13, marginBottom: open ? 12 : 0 }}>
        <tbody>
          {detail.tariffs.map(tr => (
            <tr key={tr.id} style={{ borderBottom: '1px solid var(--border)' }}>
              <td style={{ padding: '6px 0' }}>{t('energy.fromMonth', { month: tr.valid_from })}{tr.id === current?.id ? ` (${t('energy.current')})` : ''}</td>
              <td>{tr.golongan}{tr.golongan === 'I-3' ? `, K ${formatNumber(tr.k_factor ?? 1.4, lang, 2)}` : ''}</td>
              <td>LWBP {formatRupiah(tr.rate_lwbp, lang)}</td>
              <td>kVArh {formatRupiah(tr.rate_kvarh, lang)}</td>
              <td>{t('energy.factorShort')} {tr.kvarh_factor != null ? formatNumber(tr.kvarh_factor, lang, 4) : `tan(acos ${tr.pf_limit})`}</td>
              <td style={{ textAlign: 'right' }}>
                {detail.tariffs.length > 1 && (
                  <button type="button" style={{ ...smallBtn, color: 'var(--crit)' }}
                    onClick={() => confirm(t('energy.deleteTariffConfirm')) && del.mutate(tr.id)}>{t('energy.delete')}</button>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {open && (
        <form onSubmit={submit} style={grid}>
          <Field label={t('energy.validFrom')} hint={t('energy.validFromHint')}>
            <input name="valid_from" type="month" required style={inputStyle} defaultValue={new Date().toISOString().slice(0, 7)} />
          </Field>
          <TariffFields golongan={golongan} onGolongan={setGolongan} initial={current ?? undefined} />
          <Field label={t('energy.note')} wide><input name="note" style={inputStyle} /></Field>
          <ErrorLine error={set.error} />
          <div style={{ gridColumn: '1 / -1', display: 'flex', gap: 8 }}>
            <button type="submit" style={btnStyle} disabled={set.isPending}>{t('energy.save')}</button>
            <button type="button" style={btnStyle} onClick={() => setOpen(false)}>{t('energy.cancel')}</button>
          </div>
        </form>
      )}
    </div>
  )
}

function MeterSection({ detail }: { detail: SiteDetail }) {
  const { t, lang } = useLang()
  const del = useDeleteMeter()
  const [editing, setEditing] = useState<Meter | 'new' | null>(null)
  return (
    <div style={cardStyle}>
      <div style={{ ...sectionTitle, display: 'flex', justifyContent: 'space-between' }}>
        <span>{t('energy.meters')}</span>
        {editing == null && <button type="button" style={smallBtn} onClick={() => setEditing('new')}>{t('energy.addMeter')}</button>}
      </div>
      {detail.meters.length === 0 && editing == null && (
        <div style={{ color: 'var(--muted)', fontSize: 13 }}>{t('energy.noMeters')}</div>
      )}
      <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
        <tbody>
          {detail.meters.map(m => (
            <tr key={m.id} style={{ borderBottom: '1px solid var(--border)', opacity: m.enabled ? 1 : 0.5 }}>
              <td style={{ padding: '6px 0', fontWeight: 500 }}>{m.name}</td>
              <td>{t(`energy.role.${m.role}`)}</td>
              <td style={{ fontFamily: 'var(--font-mono)', fontSize: 12 }}>
                {m.device_id} / {m.kwh_metric}{m.kvarh_metric ? `, ${m.kvarh_metric}` : m.kvar_metric ? `, ∫${m.kvar_metric}` : ''}
              </td>
              <td>× {formatNumber(m.multiplier, lang, 4)}</td>
              <td style={{ textAlign: 'right', whiteSpace: 'nowrap' }}>
                <button type="button" style={smallBtn} onClick={() => setEditing(m)}>{t('energy.edit')}</button>{' '}
                <button type="button" style={{ ...smallBtn, color: 'var(--crit)' }}
                  onClick={() => confirm(t('energy.deleteMeterConfirm', { name: m.name })) && del.mutate(m)}>{t('energy.delete')}</button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {editing != null && (
        <MeterForm key={editing === 'new' ? 'new' : editing.id} siteId={detail.site.id}
          meter={editing === 'new' ? undefined : editing} onDone={() => setEditing(null)} />
      )}
    </div>
  )
}

function MeterForm({ siteId, meter, onDone }: { siteId: number; meter?: Meter; onDone: () => void }) {
  const { t } = useLang()
  const save = useSaveMeter()
  const sensors = useSensors(true)
  const tags = deviceTags(sensors.data ?? [])
  const [device, setDevice] = useState(meter?.device_id ?? '')
  const deviceList = [...tags.keys()].sort()
  const metrics = tags.get(device) ?? []

  const submit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const fd = new FormData(e.currentTarget)
    const s = (k: string) => String(fd.get(k) ?? '').trim()
    const input: MeterInput = {
      name: s('name'),
      device_id: device,
      kwh_metric: s('kwh_metric'),
      kvarh_metric: s('kvarh_metric') || null,
      kvar_metric: s('kvarh_metric') ? null : (s('kvar_metric') || null),
      role: s('role') === 'sub' ? 'sub' : 'pln_incomer',
      multiplier: numOrNull(s('multiplier')) ?? 1,
      rollover: numOrNull(s('rollover')),
      max_gap_secs: numOrNull(s('max_gap_secs')) ?? 900,
      enabled: fd.get('enabled') === 'on',
    }
    save.mutate({ siteId, id: meter?.id, ...input }, { onSuccess: onDone })
  }

  // A tag picker that still accepts a metric not listed as a data source.
  const tagInput = (name: string, value: string | null | undefined, required = false) => (
    <>
      <input name={name} list={`energy-tags-${device}`} required={required} style={monoInputStyle} defaultValue={value ?? ''} />
    </>
  )

  return (
    <form onSubmit={submit} style={{ ...grid, marginTop: 12 }}>
      <Field label={t('energy.meterName')}><input name="name" required style={inputStyle} defaultValue={meter?.name ?? ''} /></Field>
      <Field label={t('energy.meterRole')}>
        <select name="role" style={inputStyle} defaultValue={meter?.role ?? 'pln_incomer'}>
          <option value="pln_incomer">{t('energy.role.pln_incomer')}</option>
          <option value="sub">{t('energy.role.sub')}</option>
        </select>
      </Field>
      <Field label={t('energy.device')}>
        <input list="energy-devices" required style={monoInputStyle} value={device} onChange={e => setDevice(e.target.value)} />
        <datalist id="energy-devices">{deviceList.map(d => <option key={d} value={d} />)}</datalist>
        <datalist id={`energy-tags-${device}`}>{metrics.map(m => <option key={m.metric} value={m.metric}>{m.name}</option>)}</datalist>
      </Field>
      <Field label={t('energy.kwhTag')} hint={t('energy.kwhTagHint')}>{tagInput('kwh_metric', meter?.kwh_metric, true)}</Field>
      <Field label={t('energy.kvarhTag')} hint={t('energy.kvarhTagHint')}>{tagInput('kvarh_metric', meter?.kvarh_metric)}</Field>
      <Field label={t('energy.kvarTag')} hint={t('energy.kvarTagHint')}>{tagInput('kvar_metric', meter?.kvar_metric)}</Field>
      <Field label={t('energy.multiplier')} hint={t('energy.multiplierHint')}>
        <input name="multiplier" inputMode="decimal" style={inputStyle} defaultValue={String(meter?.multiplier ?? 1)} />
      </Field>
      <Field label={t('energy.rollover')} hint={t('energy.rolloverHint')}>
        <input name="rollover" inputMode="decimal" style={inputStyle} defaultValue={meter?.rollover != null ? String(meter.rollover) : ''} />
      </Field>
      <Field label={t('energy.maxGap')} hint={t('energy.maxGapHint')}>
        <input name="max_gap_secs" inputMode="numeric" style={inputStyle} defaultValue={String(meter?.max_gap_secs ?? 900)} />
      </Field>
      <label style={{ gridColumn: '1 / -1', display: 'flex', gap: 8, alignItems: 'center', fontSize: 13 }}>
        <input type="checkbox" name="enabled" defaultChecked={meter?.enabled ?? true} />{t('energy.enabled')}
      </label>
      <div style={{ gridColumn: '1 / -1', fontSize: 11, color: 'var(--muted)' }}>{t('energy.meterBackfill')}</div>
      <ErrorLine error={save.error} />
      <div style={{ gridColumn: '1 / -1', display: 'flex', gap: 8 }}>
        <button type="submit" style={btnStyle} disabled={save.isPending}>{t('energy.save')}</button>
        <button type="button" style={btnStyle} onClick={onDone}>{t('energy.cancel')}</button>
      </div>
    </form>
  )
}
