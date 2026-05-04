import React, { useEffect, useState } from 'react'
import { useQuery, useMutation } from '@tanstack/react-query'

interface EdrProduct {
  name: string
  vendor: string
  active: boolean
  notes: string | null
  category: string
}

interface EdrExclusion {
  path: string
  exclusion_type: string
  reason: string
  priority: string
}

interface IoOptimizationStatus {
  batch_writes_enabled: boolean
  batch_size: number
  wal_mode_active: boolean
  checkpoint_optimized: boolean
  write_amplification: number
  sequential_write_pct: number
}

interface EdrReport {
  detected_products: EdrProduct[]
  recommended_exclusions: EdrExclusion[]
  io_optimization: IoOptimizationStatus
  compatible: boolean
  warnings: string[]
  recommendations: string[]
}

const fetchEdrReport = async (): Promise<EdrReport> => {
  const res = await fetch('/api/v1/edr/report', { credentials: 'include' })
  if (!res.ok) throw new Error('Failed to fetch EDR report')
  return res.json()
}

const copyToClipboard = (text: string) => {
  navigator.clipboard.writeText(text).catch(() => {})
}

export const EdrCompatPanel: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const [tab, setTab] = useState<'report' | 'exclusions' | 'guide'>('report')
  const [ps1Script, setPs1Script] = useState<string | null>(null)
  const [gpoGuide, setGpoGuide] = useState<string | null>(null)

  const { data: report, isLoading, error } = useQuery({
    queryKey: ['edr-report'],
    queryFn: fetchEdrReport,
  })

  useEffect(() => {
    if (tab === 'exclusions' && !ps1Script) {
      fetch('/api/v1/edr/exclusions/ps1', { credentials: 'include' })
        .then(r => r.ok ? r.text() : '')
        .then(setPs1Script)
        .catch(() => {})
    }
    if (tab === 'guide' && !gpoGuide) {
      fetch('/api/v1/edr/gpo-guide', { credentials: 'include' })
        .then(r => r.ok ? r.text() : '')
        .then(setGpoGuide)
        .catch(() => {})
    }
  }, [tab])

  const priorityColor = (p: string) => {
    switch (p) {
      case 'critical': return '#ef4444'
      case 'recommended': return '#f59e0b'
      default: return '#6b7280'
    }
  }

  const categoryLabel = (c: string) => {
    switch (c) {
      case 'antivirus': return '🛡️ Antivirus'
      case 'edr': return '🔍 EDR'
      case 'xdr': return '🔮 XDR'
      case 'ot_security': return '🏭 OT Security'
      default: return c
    }
  }

  return (
    <div style={{
      position: 'fixed', inset: 0, zIndex: 1000,
      background: 'rgba(0,0,0,0.5)', display: 'flex',
      alignItems: 'center', justifyContent: 'center',
    }}>
      <div style={{
        width: '90%', maxWidth: 900, maxHeight: '85vh',
        background: 'var(--card)', borderRadius: 12,
        border: '1px solid var(--border)', display: 'flex',
        flexDirection: 'column', overflow: 'hidden',
        boxShadow: '0 20px 60px rgba(0,0,0,0.3)',
      }}>
        {/* Header */}
        <div style={{
          display: 'flex', alignItems: 'center', justifyContent: 'space-between',
          padding: '16px 20px', borderBottom: '1px solid var(--border)',
        }}>
          <div>
            <h2 style={{ margin: 0, fontSize: 18, fontWeight: 600 }}>
              🖥️ EDR Compatibility — Windows Defender & OT Deployment
            </h2>
            <span style={{ fontSize: 12, color: 'var(--muted)' }}>
              v0.36.0 — Ensure Voltrus runs cleanly with EDR/antivirus on OT networks
            </span>
          </div>
          <button onClick={onClose} style={{
            background: 'none', border: 'none', color: 'var(--fg)',
            fontSize: 20, cursor: 'pointer', padding: 4,
          }}>✕</button>
        </div>

        {/* Tabs */}
        <div style={{
          display: 'flex', gap: 0, borderBottom: '1px solid var(--border)',
        }}>
          {(['report', 'exclusions', 'guide'] as const).map(tabKey => (
            <button
              key={tabKey}
              onClick={() => setTab(tabKey)}
              style={{
                flex: 1, padding: '10px 16px', border: 'none', cursor: 'pointer',
                background: tab === tabKey ? 'var(--card)' : 'var(--bg)',
                color: tab === tabKey ? 'var(--fg)' : 'var(--muted)',
                fontWeight: tab === tabKey ? 600 : 400,
                borderBottom: tab === tabKey ? '2px solid var(--accent)' : '2px solid transparent',
                fontSize: 13,
              }}
            >
              {tabKey === 'report' ? '📊 Report' : tabKey === 'exclusions' ? '🚫 Exclusions' : '📖 Deployment Guide'}
            </button>
          ))}
        </div>

        {/* Content */}
        <div style={{ flex: 1, overflow: 'auto', padding: 20 }}>
          {isLoading && (
            <div style={{ textAlign: 'center', padding: 40, color: 'var(--muted)' }}>
              Scanning system for EDR/antivirus products...
            </div>
          )}

          {error && (
            <div style={{ padding: 16, background: '#fef2f2', borderRadius: 8, color: '#dc2626' }}>
              Error loading EDR report: {error.message}
            </div>
          )}

          {report && tab === 'report' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
              {/* Status Banner */}
              <div style={{
                padding: '12px 16px', borderRadius: 8,
                background: report.compatible ? '#f0fdf4' : '#fef2f2',
                border: `1px solid ${report.compatible ? '#86efac' : '#fca5a5'}`,
                display: 'flex', alignItems: 'center', gap: 8,
              }}>
                <span style={{ fontSize: 20 }}>{report.compatible ? '✅' : '⚠️'}</span>
                <div>
                  <div style={{ fontWeight: 600, color: report.compatible ? '#166534' : '#991b1b' }}>
                    {report.compatible ? 'System is EDR-compatible' : 'EDR compatibility issues detected'}
                  </div>
                  <div style={{ fontSize: 12, color: 'var(--muted)' }}>
                    {report.detected_products.length} product(s) detected • {report.warnings.length} warning(s)
                  </div>
                </div>
              </div>

              {/* Detected Products */}
              <div>
                <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 8 }}>
                  Detected EDR/Security Products
                </h3>
                {report.detected_products.length === 0 ? (
                  <div style={{ padding: 12, background: 'var(--bg)', borderRadius: 6, color: 'var(--muted)', fontSize: 13 }}>
                    No EDR/antivirus products detected on this system.
                  </div>
                ) : (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                    {report.detected_products.map((p, i) => (
                      <div key={i} style={{
                        padding: '10px 14px', background: 'var(--bg)', borderRadius: 6,
                        display: 'flex', alignItems: 'flex-start', gap: 10,
                      }}>
                        <span style={{ fontSize: 13, minWidth: 120 }}>{categoryLabel(p.category)}</span>
                        <div style={{ flex: 1 }}>
                          <div style={{ fontWeight: 500, fontSize: 13 }}>{p.name} <span style={{ color: 'var(--muted)', fontWeight: 400 }}>({p.vendor})</span></div>
                          {p.notes && <div style={{ fontSize: 12, color: 'var(--muted)', marginTop: 2 }}>{p.notes}</div>}
                        </div>
                        <span style={{
                          padding: '2px 8px', borderRadius: 4, fontSize: 11,
                          background: p.active ? '#dcfce7' : '#f3f4f6',
                          color: p.active ? '#166534' : '#6b7280',
                        }}>
                          {p.active ? 'Active' : 'Inactive'}
                        </span>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* I/O Optimization Status */}
              <div>
                <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 8 }}>
                  I/O Optimization Status
                </h3>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: 8 }}>
                  {[
                    { label: 'Batch Writes', value: report.io_optimization.batch_writes_enabled ? '✅ Enabled' : '❌ Disabled' },
                    { label: 'Batch Size', value: report.io_optimization.batch_size.toString() },
                    { label: 'WAL Mode', value: report.io_optimization.wal_mode_active ? '✅ Active' : '❌ Inactive' },
                    { label: 'Checkpoint Optimized', value: report.io_optimization.checkpoint_optimized ? '✅ Yes' : '❌ No' },
                    { label: 'Write Amplification', value: report.io_optimization.write_amplification.toFixed(1) + 'x' },
                    { label: 'Sequential Writes', value: (report.io_optimization.sequential_write_pct * 100).toFixed(0) + '%' },
                  ].map((item, i) => (
                    <div key={i} style={{
                      padding: '8px 12px', background: 'var(--bg)', borderRadius: 6,
                      display: 'flex', justifyContent: 'space-between', alignItems: 'center',
                    }}>
                      <span style={{ fontSize: 12, color: 'var(--muted)' }}>{item.label}</span>
                      <span style={{ fontSize: 13, fontWeight: 500 }}>{item.value}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Warnings */}
              {report.warnings.length > 0 && (
                <div>
                  <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 8 }}>⚠️ Warnings</h3>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
                    {report.warnings.map((w, i) => (
                      <div key={i} style={{
                        padding: '8px 12px', background: '#fffbeb', borderRadius: 6,
                        borderLeft: '3px solid #f59e0b', fontSize: 13,
                      }}>
                        {w}
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Recommendations */}
              {report.recommendations.length > 0 && (
                <div>
                  <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 8 }}>💡 Recommendations</h3>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
                    {report.recommendations.map((r, i) => (
                      <div key={i} style={{
                        padding: '8px 12px', background: 'var(--bg)', borderRadius: 6,
                        borderLeft: '3px solid var(--accent)', fontSize: 13,
                      }}>
                        {r}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {report && tab === 'exclusions' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
              <div style={{ fontSize: 13, color: 'var(--muted)' }}>
                Add these exclusion paths to your EDR/antivirus policy. Critical exclusions are required for acceptable performance.
              </div>

              {/* Exclusion Table */}
              <div style={{ overflow: 'auto' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
                  <thead>
                    <tr style={{ borderBottom: '2px solid var(--border)' }}>
                      <th style={{ textAlign: 'left', padding: 8 }}>Priority</th>
                      <th style={{ textAlign: 'left', padding: 8 }}>Type</th>
                      <th style={{ textAlign: 'left', padding: 8 }}>Path</th>
                      <th style={{ textAlign: 'left', padding: 8 }}>Reason</th>
                    </tr>
                  </thead>
                  <tbody>
                    {report.recommended_exclusions.map((exc, i) => (
                      <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
                        <td style={{ padding: 8 }}>
                          <span style={{
                            padding: '2px 8px', borderRadius: 4, fontSize: 11, fontWeight: 600,
                            background: exc.priority === 'critical' ? '#fef2f2' : exc.priority === 'recommended' ? '#fffbeb' : '#f3f4f6',
                            color: priorityColor(exc.priority),
                          }}>
                            {exc.priority.toUpperCase()}
                          </span>
                        </td>
                        <td style={{ padding: 8, fontFamily: 'monospace', fontSize: 12 }}>{exc.exclusion_type}</td>
                        <td style={{ padding: 8, fontFamily: 'monospace', fontSize: 12 }}>{exc.path}</td>
                        <td style={{ padding: 8, color: 'var(--muted)' }}>{exc.reason}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              {/* PowerShell Script */}
              {ps1Script && (
                <div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
                    <h3 style={{ fontSize: 14, fontWeight: 600, margin: 0 }}>
                      Windows Defender PowerShell Script
                    </h3>
                    <button
                      onClick={() => copyToClipboard(ps1Script)}
                      style={{
                        padding: '4px 12px', borderRadius: 4, border: '1px solid var(--border)',
                        background: 'var(--card)', color: 'var(--fg)', fontSize: 12, cursor: 'pointer',
                      }}
                    >
                      📋 Copy Script
                    </button>
                  </div>
                  <pre style={{
                    background: '#1e1e1e', color: '#d4d4d4', padding: 16, borderRadius: 8,
                    fontSize: 12, overflow: 'auto', maxHeight: 300, lineHeight: 1.5,
                  }}>
                    {ps1Script}
                  </pre>
                </div>
              )}
            </div>
          )}

          {tab === 'guide' && (
            <div>
              <div style={{ fontSize: 13, color: 'var(--muted)', marginBottom: 12 }}>
                OT Network Deployment Guide for IT teams deploying Voltrus to SCADA environments.
              </div>
              {gpoGuide ? (
                <pre style={{
                  background: 'var(--bg)', padding: 16, borderRadius: 8,
                  fontSize: 12, overflow: 'auto', maxHeight: 500, lineHeight: 1.7,
                  whiteSpace: 'pre-wrap', fontFamily: 'inherit',
                }}>
                  {gpoGuide}
                </pre>
              ) : (
                <div style={{ color: 'var(--muted)' }}>Loading deployment guide...</div>
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div style={{
          padding: '12px 20px', borderTop: '1px solid var(--border)',
          display: 'flex', justifyContent: 'space-between', alignItems: 'center',
        }}>
          <span style={{ fontSize: 12, color: 'var(--muted)' }}>
            Source: r/SCADA — "xagt.exe (FireEye) deployed via AD to SCADA servers wasted CPU on monitoring communications"
          </span>
          <button onClick={onClose} style={{
            padding: '6px 16px', borderRadius: 6, border: '1px solid var(--border)',
            background: 'var(--card)', color: 'var(--fg)', cursor: 'pointer', fontSize: 13,
          }}>
            Close
          </button>
        </div>
      </div>
    </div>
  )
}
