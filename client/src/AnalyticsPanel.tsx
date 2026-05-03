import { useState, useMemo, useCallback } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import {
  useAnalyticsAggregate,
  useAnalyticsCompare,
  useSpcChart,
  useHistogram,
  useRunChart,
  useCustomMetrics,
  useCustomMetricValues,
} from './queries/analytics';
import { fetchOk } from './queries/fetch';
import { useModbusDevices } from './queries/modbus';
import { useLicense } from './LicenseContext';
import { btnStyle } from './styles';
import { SPCChart } from './components/SPCChart';
import { HistogramChart } from './components/HistogramChart';
import { RunChart } from './components/RunChart';
import { CalcBuilder } from './components/CalcBuilder';

const RANGES: { label: string; seconds: number }[] = [
  { label: '1h', seconds: 3600 },
  { label: '6h', seconds: 21600 },
  { label: '24h', seconds: 86400 },
  { label: '7d', seconds: 604800 },
  { label: '30d', seconds: 2592000 },
];

const INTERVALS: Record<string, number> = {
  '1h': 60,
  '6h': 300,
  '24h': 3600,
  '7d': 21600,
  '30d': 86400,
};

type TabId = 'overview' | 'spc' | 'histogram' | 'run-chart' | 'compare' | 'custom-metrics';

const TABS: { id: TabId; label: string }[] = [
  { id: 'overview', label: 'Overview' },
  { id: 'spc', label: 'SPC' },
  { id: 'histogram', label: 'Histogram' },
  { id: 'run-chart', label: 'Run Chart' },
  { id: 'compare', label: 'Compare' },
  { id: 'custom-metrics', label: 'Custom Metrics' },
];

function fmt(v: number, decimals = 1) {
  if (Number.isNaN(v) || !Number.isFinite(v)) return '--';
  return v.toFixed(decimals);
}

function fmtTs(ts: number) {
  return new Date(ts * 1000).toLocaleString();
}

function exportCsv(data: [number, { min: number; max: number; avg: number; count: number }][], metric: string) {
  const header = 'timestamp,min,max,avg,count,metric\n';
  const rows = data.map(([ts, m]) =>
    `${ts},${m.min},${m.max},${m.avg},${m.count},${metric}`
  ).join('\n');
  const blob = new Blob([header + rows], { type: 'text/csv' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `analytics_${metric}_${Date.now()}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}

export function AnalyticsPanel() {
  const license = useLicense();
  const isAnalyticsDisabled = license?.feature_flags?.analytics === false;

  const { data: devices } = useModbusDevices();
  const [deviceId, setDeviceId] = useState('');
  const [metric, setMetric] = useState('');
  const [rangeIdx, setRangeIdx] = useState(2); // 24h
  const [tab, setTab] = useState<TabId>('overview');

  // SPC params
  const [spcChartType, setSpcChartType] = useState<'xbar_r' | 'cusum' | 'ewma'>('xbar_r');
  const [subgroupSize, setSubgroupSize] = useState(5);
  const [spcTarget, setSpcTarget] = useState<number | ''>('');
  const [spcLambda, setSpcLambda] = useState<number | ''>('');
  const [spcK, setSpcK] = useState<number | ''>('');
  const [spcH, setSpcH] = useState<number | ''>('');

  // Histogram params
  const [histBins, setHistBins] = useState(20);
  const [histLsl, setHistLsl] = useState<number | ''>('');
  const [histUsl, setHistUsl] = useState<number | ''>('');

  const now = Math.floor(Date.now() / 1000);
  const range = RANGES[rangeIdx];
  const from = now - range.seconds;
  const to = now;
  const interval = INTERVALS[range.label] || 3600;

  // Comparison: split the range in half
  const mid = Math.floor((from + to) / 2);

  // Shared queries
  const { data: aggData, isLoading: aggLoading } = useAnalyticsAggregate(
    deviceId || undefined, metric || undefined, from, to, interval,
  );
  const { data: compData, isLoading: compLoading } = useAnalyticsCompare(
    deviceId || undefined, metric || undefined, from, mid, mid, to,
  );

  // SPC query
  const { data: spcData, isLoading: spcLoading } = useSpcChart(
    deviceId || undefined, metric || undefined, from, to, spcChartType,
    subgroupSize || undefined,
    spcTarget !== '' ? spcTarget : undefined,
    spcLambda !== '' ? spcLambda : undefined,
    spcK !== '' ? spcK : undefined,
    spcH !== '' ? spcH : undefined,
  );

  // Histogram query
  const { data: histData, isLoading: histLoading } = useHistogram(
    deviceId || undefined, metric || undefined, from, to,
    histBins || undefined,
    histLsl !== '' ? histLsl : undefined,
    histUsl !== '' ? histUsl : undefined,
  );

  // Run chart query
  const { data: runData, isLoading: runLoading } = useRunChart(
    deviceId || undefined, metric || undefined, from, to,
  );

  // Custom metrics queries
  const { data: customMetrics } = useCustomMetrics();
  const { data: customValues } = useCustomMetricValues();
  const queryClient = useQueryClient();

  const handleCreateMetric = useCallback(async (input: { name: string; description: string; formula: string; unit?: string }) => {
    await fetchOk('/api/v1/analytics/custom-metrics', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(input),
    });
    queryClient.invalidateQueries({ queryKey: ['analytics', 'custom-metrics'] });
  }, [queryClient]);

  const handleUpdateMetric = useCallback(async (id: number, input: { name?: string; description?: string; formula?: string; unit?: string }) => {
    await fetchOk(`/api/v1/analytics/custom-metrics/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(input),
    });
    queryClient.invalidateQueries({ queryKey: ['analytics', 'custom-metrics'] });
  }, [queryClient]);

  const handleDeleteMetric = useCallback(async (id: number) => {
    await fetchOk(`/api/v1/analytics/custom-metrics/${id}`, { method: 'DELETE' });
    queryClient.invalidateQueries({ queryKey: ['analytics', 'custom-metrics'] });
  }, [queryClient]);

  // Compute overall stats from aggregated data
  const overallStats = useMemo(() => {
    if (!aggData || aggData.length === 0) return null;
    let min = Infinity, max = -Infinity, sum = 0, count = 0;
    for (const [, m] of aggData) {
      if (m.min < min) min = m.min;
      if (m.max > max) max = m.max;
      sum += m.avg * m.count;
      count += m.count;
    }
    return {
      min,
      max,
      avg: count > 0 ? sum / count : 0,
      count,
      dataPoints: aggData.length,
    };
  }, [aggData]);

  const sparklinePoints = useMemo(() => {
    if (!aggData || aggData.length === 0) return '';
    const values = aggData.map(([, m]) => m.avg);
    if (values.length < 2) return '';
    const mn = Math.min(...values);
    const mx = Math.max(...values);
    const range = mx - mn || 1;
    const w = 200;
    const h = 40;
    const pts = values.map((v, i) => {
      const x = (i / (values.length - 1)) * w;
      const y = h - ((v - mn) / range) * h;
      return `${x},${y}`;
    }).join(' ');
    return `<polyline points="${pts}" fill="none" stroke="var(--cyan)" stroke-width="1.5" />`;
  }, [aggData]);

  if (isAnalyticsDisabled) {
    return (
      <div style={{ padding: 40, textAlign: 'center', color: 'var(--muted)' }}>
        <h2 style={{ marginBottom: 12 }}>Analytics</h2>
        <p>This feature requires a Starter tier license or above.</p>
        <p style={{ marginTop: 8 }}>Upgrade at <strong>voltrus.id</strong></p>
      </div>
    );
  }

  return (
    <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 12 }}>
      {/* Shared controls */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, flexWrap: 'wrap' }}>
        <h2 style={{ margin: 0 }}>Analytics</h2>

        <select
          value={deviceId}
          onChange={e => { setDeviceId(e.target.value); setMetric(''); }}
          style={selectStyle}
        >
          <option value="">Select device...</option>
          {devices?.map(d => (
            <option key={d.id} value={d.id}>{d.name || d.id}</option>
          ))}
        </select>

        <input
          placeholder="Metric name (e.g. voltage)"
          value={metric}
          onChange={e => setMetric(e.target.value)}
          style={inputStyle}
        />

        <div style={{ display: 'flex', gap: 4 }}>
          {RANGES.map((r, i) => (
            <button
              key={r.label}
              onClick={() => setRangeIdx(i)}
              style={{
                ...btnStyle,
                padding: '4px 12px',
                fontSize: 12,
                background: rangeIdx === i ? 'rgba(34,197,94,.15)' : 'transparent',
                color: rangeIdx === i ? 'var(--cyan)' : 'var(--muted)',
                border: rangeIdx === i ? '1px solid var(--cyan)' : '1px solid var(--border)',
              }}
            >
              {r.label}
            </button>
          ))}
        </div>
      </div>

      {/* Tab bar */}
      <div style={{ display: 'flex', gap: 0, borderBottom: '1px solid var(--border)' }}>
        {TABS.map(t => (
          <button
            key={t.id}
            onClick={() => setTab(t.id)}
            style={{
              padding: '8px 16px',
              fontSize: 12,
              fontWeight: 600,
              cursor: 'pointer',
              border: 'none',
              borderBottom: tab === t.id ? '2px solid var(--cyan)' : '2px solid transparent',
              color: tab === t.id ? 'var(--cyan)' : 'var(--muted)',
              background: 'transparent',
              textTransform: 'uppercase' as const,
              letterSpacing: '.04em',
            }}
          >
            {t.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      {!deviceId || !metric ? (
        <p style={{ color: 'var(--muted)', textAlign: 'center', marginTop: 40 }}>
          Select a device and enter a metric name to view analytics.
        </p>
      ) : tab === 'overview' ? (
        <OverviewTab
          aggData={aggData}
          aggLoading={aggLoading}
          overallStats={overallStats}
          sparklinePoints={sparklinePoints}
          from={from}
          to={to}
          metric={metric}
        />
      ) : tab === 'spc' ? (
        <SpcTab
          data={spcData ?? null}
          loading={spcLoading}
          chartType={spcChartType}
          setChartType={setSpcChartType}
          subgroupSize={subgroupSize}
          setSubgroupSize={setSubgroupSize}
          target={spcTarget}
          setTarget={setSpcTarget}
          lambda={spcLambda}
          setLambda={setSpcLambda}
          k={spcK}
          setK={setSpcK}
          h={spcH}
          setH={setSpcH}
        />
      ) : tab === 'histogram' ? (
        <HistogramTab
          data={histData ?? null}
          loading={histLoading}
          bins={histBins}
          setBins={setHistBins}
          lsl={histLsl}
          setLsl={setHistLsl}
          usl={histUsl}
          setUsl={setHistUsl}
        />
      ) : tab === 'run-chart' ? (
        <RunChartTab data={runData ?? null} loading={runLoading} />
      ) : tab === 'compare' ? (
        <CompareTab compData={compData ?? null} compLoading={compLoading} />
      ) : tab === 'custom-metrics' ? (
        <CustomMetricsTab
          metrics={customMetrics ?? []}
          values={customValues ?? null}
          onCreate={handleCreateMetric}
          onUpdate={handleUpdateMetric}
          onDelete={handleDeleteMetric}
        />
      ) : null}
    </div>
  );
}

// ── Overview Tab ──

function OverviewTab({ aggData, aggLoading, overallStats, sparklinePoints, from, to, metric }: {
  aggData: [number, { min: number; max: number; avg: number; count: number }][] | undefined;
  aggLoading: boolean;
  overallStats: { min: number; max: number; avg: number; count: number; dataPoints: number } | null;
  sparklinePoints: string;
  from: number;
  to: number;
  metric: string;
}) {
  return (
    <>
      {aggLoading && <p style={{ color: 'var(--muted)' }}>Loading...</p>}
      {overallStats && (
        <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(140px, 1fr))',
          gap: 12,
        }}>
          <StatCard label="Min" value={fmt(overallStats.min)} />
          <StatCard label="Max" value={fmt(overallStats.max)} />
          <StatCard label="Average" value={fmt(overallStats.avg)} />
          <StatCard label="Samples" value={String(overallStats.count)} />
          <StatCard label="Buckets" value={String(overallStats.dataPoints)} />
        </div>
      )}
      {sparklinePoints && (
        <div style={{
          background: 'var(--card)',
          border: '1px solid var(--border)',
          borderRadius: 8,
          padding: 16,
        }}>
          <svg width="100%" height="40" viewBox={`0 0 200 40`} preserveAspectRatio="none">
            {sparklinePoints}
          </svg>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11, color: 'var(--muted)', marginTop: 4 }}>
            <span>{fmtTs(from)}</span>
            <span>{fmtTs(to)}</span>
          </div>
        </div>
      )}
      {aggData && aggData.length > 0 && (
        <button
          onClick={() => exportCsv(aggData, metric)}
          style={{ ...btnStyle, padding: '6px 16px', fontSize: 12 }}
        >
          Export CSV
        </button>
      )}
    </>
  );
}

// ── SPC Tab ──

function SpcTab({ data, loading, chartType, setChartType, subgroupSize, setSubgroupSize, target, setTarget, lambda, setLambda, k, setK, h, setH }: {
  data: {
    chart_type: string;
    data_points: Array<{ index: number; value: number; timestamp: number }>;
    control_limits: { ucl: number; cl: number; lcl: number };
    violations: number[];
    in_control_percent: number;
  } | null;
  loading: boolean;
  chartType: 'xbar_r' | 'cusum' | 'ewma';
  setChartType: (v: 'xbar_r' | 'cusum' | 'ewma') => void;
  subgroupSize: number;
  setSubgroupSize: (v: number) => void;
  target: number | '';
  setTarget: (v: number | '') => void;
  lambda: number | '';
  setLambda: (v: number | '') => void;
  k: number | '';
  setK: (v: number | '') => void;
  h: number | '';
  setH: (v: number | '') => void;
}) {
  return (
    <>
      {/* SPC controls */}
      <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap', alignItems: 'center' }}>
        <div style={{ display: 'flex', gap: 4 }}>
          {(['xbar_r', 'cusum', 'ewma'] as const).map(ct => (
            <button
              key={ct}
              onClick={() => setChartType(ct)}
              style={{
                ...btnStyle,
                padding: '4px 12px',
                fontSize: 12,
                background: chartType === ct ? 'rgba(34,197,94,.15)' : 'transparent',
                color: chartType === ct ? 'var(--cyan)' : 'var(--muted)',
                border: chartType === ct ? '1px solid var(--cyan)' : '1px solid var(--border)',
              }}
            >
              {ct === 'xbar_r' ? 'X-bar & R' : ct === 'cusum' ? 'CUSUM' : 'EWMA'}
            </button>
          ))}
        </div>
        {chartType === 'xbar_r' && (
          <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--muted)' }}>
            Subgroup size
            <input
              type="number"
              min={2}
              max={25}
              value={subgroupSize}
              onChange={e => setSubgroupSize(Number(e.target.value) || 5)}
              style={{ ...inputStyle, width: 60 }}
            />
          </label>
        )}
        {chartType === 'cusum' && (
          <>
            <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--muted)' }}>
              Target
              <input
                type="number"
                step="any"
                value={target}
                onChange={e => setTarget(e.target.value === '' ? '' : Number(e.target.value))}
                placeholder="auto"
                style={{ ...inputStyle, width: 80 }}
              />
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--muted)' }}>
              k
              <input
                type="number"
                step="any"
                value={k}
                onChange={e => setK(e.target.value === '' ? '' : Number(e.target.value))}
                placeholder="0.5"
                style={{ ...inputStyle, width: 60 }}
              />
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--muted)' }}>
              h
              <input
                type="number"
                step="any"
                value={h}
                onChange={e => setH(e.target.value === '' ? '' : Number(e.target.value))}
                placeholder="4"
                style={{ ...inputStyle, width: 60 }}
              />
            </label>
          </>
        )}
        {chartType === 'ewma' && (
          <>
            <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--muted)' }}>
              Target
              <input
                type="number"
                step="any"
                value={target}
                onChange={e => setTarget(e.target.value === '' ? '' : Number(e.target.value))}
                placeholder="auto"
                style={{ ...inputStyle, width: 80 }}
              />
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--muted)' }}>
              Lambda
              <input
                type="number"
                step="0.01"
                min={0}
                max={1}
                value={lambda}
                onChange={e => setLambda(e.target.value === '' ? '' : Number(e.target.value))}
                placeholder="0.2"
                style={{ ...inputStyle, width: 70 }}
              />
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--muted)' }}>
              k
              <input
                type="number"
                step="any"
                value={k}
                onChange={e => setK(e.target.value === '' ? '' : Number(e.target.value))}
                placeholder="3"
                style={{ ...inputStyle, width: 60 }}
              />
            </label>
          </>
        )}
      </div>
      {/* SPC chart */}
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 8, height: 320 }}>
        <SPCChart data={data} loading={loading} />
      </div>
    </>
  );
}

// ── Histogram Tab ──

function HistogramTab({ data, loading, bins, setBins, lsl, setLsl, usl, setUsl }: {
  data: {
    bins: Array<{ lower: number; upper: number; count: number; frequency: number }>;
    normal_curve: Array<{ x: number; y: number }>;
    stats: { mean: number; stddev: number; count: number };
    capability: { cp: number; cpk: number; pp: number; ppk: number } | null;
  } | null;
  loading: boolean;
  bins: number;
  setBins: (v: number) => void;
  lsl: number | '';
  setLsl: (v: number | '') => void;
  usl: number | '';
  setUsl: (v: number | '') => void;
}) {
  return (
    <>
      {/* Histogram controls */}
      <div style={{ display: 'flex', gap: 16, flexWrap: 'wrap', alignItems: 'center' }}>
        <label style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 12, color: 'var(--muted)' }}>
          Bins: {bins}
          <input
            type="range"
            min={5}
            max={100}
            value={bins}
            onChange={e => setBins(Number(e.target.value))}
            style={{ width: 120 }}
          />
        </label>
        <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--muted)' }}>
          LSL
          <input
            type="number"
            step="any"
            value={lsl}
            onChange={e => setLsl(e.target.value === '' ? '' : Number(e.target.value))}
            placeholder="optional"
            style={{ ...inputStyle, width: 80 }}
          />
        </label>
        <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--muted)' }}>
          USL
          <input
            type="number"
            step="any"
            value={usl}
            onChange={e => setUsl(e.target.value === '' ? '' : Number(e.target.value))}
            placeholder="optional"
            style={{ ...inputStyle, width: 80 }}
          />
        </label>
      </div>
      {/* Histogram chart */}
      <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 8, height: 360 }}>
        <HistogramChart
          data={data}
          loading={loading}
          lsl={lsl !== '' ? lsl : undefined}
          usl={usl !== '' ? usl : undefined}
        />
      </div>
    </>
  );
}

// ── Run Chart Tab ──

function RunChartTab({ data, loading }: {
  data: {
    points: Array<{ timestamp: number; value: number; above_median: boolean }>;
    median: number;
    signals: Array<{ rule: string; start_idx: number; end_idx: number; description: string }>;
    runs_count: number;
  } | null;
  loading: boolean;
}) {
  return (
    <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 8, height: 320 }}>
      <RunChart data={data} loading={loading} />
    </div>
  );
}

// ── Compare Tab ──

function CompareTab({ compData, compLoading }: {
  compData: {
    metric: string;
    current: { min: number; max: number; avg: number; count: number; first_ts: number; last_ts: number };
    previous: { min: number; max: number; avg: number; count: number; first_ts: number; last_ts: number };
    change_percent: number;
  } | null;
  compLoading: boolean;
}) {
  return (
    <>
      {compLoading && <p style={{ color: 'var(--muted)' }}>Loading...</p>}
      {compData && (
        <div style={{
          background: 'var(--card)',
          border: '1px solid var(--border)',
          borderRadius: 8,
          padding: 16,
        }}>
          <h3 style={{ margin: '0 0 16px', fontSize: 14 }}>Period Comparison</h3>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
            <div>
              <div style={{ fontSize: 11, color: 'var(--muted)', marginBottom: 8 }}>
                Current ({fmtTs(compData.current.first_ts)} - {fmtTs(compData.current.last_ts)})
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
                <div><span style={{ color: 'var(--muted)', fontSize: 11 }}>Avg</span><br />{fmt(compData.current.avg)}</div>
                <div><span style={{ color: 'var(--muted)', fontSize: 11 }}>Min</span><br />{fmt(compData.current.min)}</div>
                <div><span style={{ color: 'var(--muted)', fontSize: 11 }}>Max</span><br />{fmt(compData.current.max)}</div>
                <div><span style={{ color: 'var(--muted)', fontSize: 11 }}>Count</span><br />{compData.current.count}</div>
              </div>
            </div>
            <div>
              <div style={{ fontSize: 11, color: 'var(--muted)', marginBottom: 8 }}>
                Previous ({fmtTs(compData.previous.first_ts)} - {fmtTs(compData.previous.last_ts)})
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
                <div><span style={{ color: 'var(--muted)', fontSize: 11 }}>Avg</span><br />{fmt(compData.previous.avg)}</div>
                <div><span style={{ color: 'var(--muted)', fontSize: 11 }}>Min</span><br />{fmt(compData.previous.min)}</div>
                <div><span style={{ color: 'var(--muted)', fontSize: 11 }}>Max</span><br />{fmt(compData.previous.max)}</div>
                <div><span style={{ color: 'var(--muted)', fontSize: 11 }}>Count</span><br />{compData.previous.count}</div>
              </div>
            </div>
          </div>
          <div style={{
            marginTop: 16, padding: '8px 12px', borderRadius: 6,
            background: compData.change_percent >= 0 ? 'rgba(34,197,94,.1)' : 'rgba(239,68,68,.1)',
            textAlign: 'center', fontSize: 14, fontWeight: 600,
            color: compData.change_percent >= 0 ? 'var(--ok)' : 'var(--crit)',
          }}>
            {compData.change_percent >= 0 ? '+' : ''}{fmt(compData.change_percent)}% change
          </div>
        </div>
      )}
    </>
  );
}

// ── Custom Metrics Tab ──

function CustomMetricsTab({ metrics, values, onCreate, onUpdate, onDelete }: {
  metrics: Array<{
    id: number;
    name: string;
    description: string;
    formula: string;
    unit: string | null;
    validation_error: string | null;
  }>;
  values: Array<{ id: number; name: string; value: number; unit: string | null }> | null;
  onCreate: (input: { name: string; description: string; formula: string; unit?: string }) => void;
  onUpdate: (id: number, input: { name?: string; description?: string; formula?: string; unit?: string }) => void;
  onDelete: (id: number) => void;
}) {
  return (
    <div style={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 8 }}>
      <CalcBuilder
        metrics={metrics}
        values={values}
        onCreate={onCreate}
        onUpdate={onUpdate}
        onDelete={onDelete}
      />
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div style={{
      background: 'var(--card)',
      border: '1px solid var(--border)',
      borderRadius: 8,
      padding: '12px 16px',
    }}>
      <div style={{ fontSize: 11, color: 'var(--muted)', marginBottom: 4 }}>{label}</div>
      <div style={{ fontSize: 18, fontWeight: 600 }}>{value}</div>
    </div>
  );
}

const selectStyle: React.CSSProperties = {
  background: 'var(--card)',
  border: '1px solid var(--border)',
  borderRadius: 6,
  padding: '6px 12px',
  color: 'var(--fg)',
  fontSize: 13,
};

const inputStyle: React.CSSProperties = {
  background: 'var(--card)',
  border: '1px solid var(--border)',
  borderRadius: 6,
  padding: '6px 12px',
  color: 'var(--fg)',
  fontSize: 13,
  width: 200,
};
