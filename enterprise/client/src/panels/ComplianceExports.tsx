import { useEffect, useState } from 'react'
import { btnStyle, inputStyle, labelStyle, labelTextStyle } from '@voltrus/core/styles'
import { useSaveSignerName, useSignerNames } from '../queries/compliance'
import { auditExportUrl, rangeProblem, signaturesExportUrl, type ExportFilter } from './exportUrl'

const linkBtn: React.CSSProperties = { ...btnStyle, textDecoration: 'none', display: 'inline-flex', alignItems: 'center' }

function FilterFields({ filter, onChange, fields }: {
  filter: ExportFilter
  onChange: (f: ExportFilter) => void
  fields: (keyof ExportFilter)[]
}) {
  const labels: Record<keyof ExportFilter, string> = {
    from: 'From', to: 'To', username: 'User', action: 'Action', search: 'Contains',
  }
  return (
    <div style={{ display: 'flex', flexWrap: 'wrap', gap: 10, alignItems: 'flex-end' }}>
      {fields.map(key => (
        <label key={key} style={{ ...labelStyle, minWidth: key === 'from' || key === 'to' ? 150 : 130 }}>
          <span style={labelTextStyle}>{labels[key]}</span>
          <input type={key === 'from' || key === 'to' ? 'date' : 'text'} value={filter[key] ?? ''}
            onChange={e => onChange({ ...filter, [key]: e.target.value })} style={inputStyle}
            placeholder={key === 'username' ? 'any' : key === 'action' ? 'e.g. login' : ''} />
        </label>
      ))}
    </div>
  )
}

function ExportButtons({ csv, pdf, problem }: { csv: string; pdf: string; problem: string | null }) {
  if (problem) return <span role="alert" style={{ color: 'var(--crit)', fontSize: 12 }}>{problem}</span>
  return (
    <div style={{ display: 'flex', gap: 8 }}>
      <a href={csv} download style={linkBtn}>Export CSV</a>
      <a href={pdf} download style={{ ...linkBtn, borderColor: 'var(--cyan)', color: 'var(--cyan)' }}>Export PDF</a>
    </div>
  )
}

/** The export bar above the signature records (admins). */
export function SignatureExportBar() {
  const [filter, setFilter] = useState<ExportFilter>({})
  return (
    <div style={{ display: 'flex', flexWrap: 'wrap', gap: 12, alignItems: 'flex-end', justifyContent: 'space-between' }}>
      <FilterFields filter={filter} onChange={setFilter} fields={['from', 'to', 'username']} />
      <ExportButtons csv={signaturesExportUrl('csv', filter)} pdf={signaturesExportUrl('pdf', filter)} problem={rangeProblem(filter)} />
    </div>
  )
}

/** Audit trail export: the core CSV and the Part 11 PDF with the same filter. */
export function AuditTrailTab() {
  const [filter, setFilter] = useState<ExportFilter>({})
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16, maxWidth: 820 }}>
      <p style={{ margin: 0, fontSize: 13, color: 'var(--muted)' }}>
        Every entry of the audit trail in the range, oldest first, with its checksum (its link in the hash chain) and the
        result of verifying the whole chain. Times are printed in this browser's time zone. The full log is under Data
        Manager › Admin › Audit Log.
      </p>
      <FilterFields filter={filter} onChange={setFilter} fields={['from', 'to', 'username', 'action', 'search']} />
      <ExportButtons csv={auditExportUrl('csv', filter)} pdf={auditExportUrl('pdf', filter)} problem={rangeProblem(filter)} />
    </div>
  )
}

function SignerNameRow({ username, role, fullName }: { username: string; role: string; fullName: string }) {
  const [value, setValue] = useState(fullName)
  const save = useSaveSignerName()
  useEffect(() => setValue(fullName), [fullName])
  const changed = value.trim() !== fullName
  return (
    <tr>
      <td style={{ padding: '6px 12px', fontFamily: 'var(--font-mono)', fontSize: 12 }}>{username}</td>
      <td style={{ padding: '6px 12px', fontSize: 12, color: 'var(--muted)' }}>{role}</td>
      <td style={{ padding: '6px 12px' }}>
        <input aria-label={`Full name of ${username}`} value={value} onChange={e => setValue(e.target.value)} placeholder="Full legal name"
          style={{ ...inputStyle, width: '100%' }} />
      </td>
      <td style={{ padding: '6px 12px', whiteSpace: 'nowrap' }}>
        <button type="button" disabled={!changed || save.isPending} style={{ ...btnStyle, opacity: changed ? 1 : 0.5 }}
          onClick={() => save.mutate({ username, full_name: value })}>Save</button>
        {save.isError && <span style={{ color: 'var(--crit)', fontSize: 11, marginLeft: 6 }}>{String((save.error as Error).message)}</span>}
      </td>
    </tr>
  )
}

/** Full names printed with each person's signatures (§11.50). */
export function SignerNamesTab() {
  const { data, isLoading } = useSignerNames()
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12, maxWidth: 820 }}>
      <p style={{ margin: 0, fontSize: 13, color: 'var(--muted)' }}>
        The printed name of each signer. It is recorded with every signature at the moment of signing, so changing it
        later does not change signatures already given.
      </p>
      {isLoading ? <div style={{ color: 'var(--muted)' }}>Loading...</div> : (
        <table style={{ borderCollapse: 'collapse', width: '100%' }}>
          <thead>
            <tr>{['User', 'Role', 'Full name', ''].map(h => (
              <th key={h} style={{ textAlign: 'left', padding: '6px 12px', fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', borderBottom: '1px solid var(--border)' }}>{h}</th>
            ))}</tr>
          </thead>
          <tbody>
            {(data ?? []).map(u => <SignerNameRow key={u.username} username={u.username} role={u.role} fullName={u.full_name} />)}
          </tbody>
        </table>
      )}
    </div>
  )
}
