import { useState } from 'react'
import { useCameraSnapshots, useCaptureSnapshot } from '../queries/video'

interface CameraWidgetProps {
  cameraId: number
  cameraName: string
  onExpand?: (snapshotPath: string) => void
}

export function CameraWidget({ cameraId, cameraName, onExpand }: CameraWidgetProps) {
  const { data: snapshots = [], isLoading } = useCameraSnapshots(cameraId)
  const captureMut = useCaptureSnapshot(cameraId)
  const [imgError, setImgError] = useState(false)

  const latest = snapshots.length > 0 ? snapshots[0] : null
  const snapshotUrl = latest ? `/api/v1/cameras/${cameraId}/snapshots/${latest.id}/file` : null

  const handleCapture = () => {
    captureMut.mutate(undefined, {
      onSuccess: () => setImgError(false),
    })
  }

  const handleImageClick = () => {
    if (latest && onExpand) {
      onExpand(latest.file_path)
    }
  }

  const formatTime = (ts: number) => {
    return new Date(ts * 1000).toLocaleString('en-US', {
      month: 'short', day: 'numeric',
      hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false,
    })
  }

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  return (
    <div style={{
      background: 'var(--card)',
      border: '1px solid var(--border)',
      borderRadius: 10,
      overflow: 'hidden',
      display: 'flex',
      flexDirection: 'column',
    }}>
      {/* Camera name header */}
      <div style={{
        padding: '8px 12px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        borderBottom: '1px solid var(--border)',
      }}>
        <span style={{ fontSize: 13, fontWeight: 600, color: 'var(--fg)' }}>
          {cameraName}
        </span>
        <button
          onClick={handleCapture}
          disabled={captureMut.isPending}
          style={{
            padding: '3px 8px',
            borderRadius: 4,
            border: '1px solid var(--border)',
            background: 'transparent',
            color: 'var(--fg)',
            cursor: captureMut.isPending ? 'not-allowed' : 'pointer',
            fontSize: 11,
            opacity: captureMut.isPending ? 0.5 : 1,
          }}
          aria-label={`Capture snapshot from ${cameraName}`}
        >
          {captureMut.isPending ? '...' : 'Capture'}
        </button>
      </div>

      {/* Snapshot display */}
      <div style={{
        position: 'relative',
        minHeight: 180,
        background: '#0a0a1a',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
      }}>
        {isLoading ? (
          <span style={{ color: 'var(--muted)', fontSize: 12 }}>Loading...</span>
        ) : latest && snapshotUrl && !imgError ? (
          <img
            src={snapshotUrl}
            alt={`Snapshot from ${cameraName}`}
            onClick={handleImageClick}
            onError={() => setImgError(true)}
            style={{
              width: '100%',
              height: 'auto',
              maxHeight: 240,
              objectFit: 'cover',
              cursor: onExpand ? 'pointer' : 'default',
              display: 'block',
            }}
          />
        ) : (
          <span style={{ color: 'var(--muted)', fontSize: 12 }}>
            {latest ? 'Image unavailable' : 'No snapshots'}
          </span>
        )}

        {/* Timestamp overlay */}
        {latest && (
          <div style={{
            position: 'absolute',
            bottom: 0,
            left: 0,
            right: 0,
            padding: '4px 8px',
            background: 'linear-gradient(transparent, rgba(0,0,0,0.8))',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'flex-end',
          }}>
            <span style={{ fontSize: 10, color: 'rgba(255,255,255,0.7)', fontFamily: "var(--font-mono)" }}>
              {formatTime(latest.timestamp)}
            </span>
            {latest.alarm_source && (
              <span style={{
                fontSize: 10,
                color: '#e74c3c',
                background: 'rgba(231,76,60,0.15)',
                padding: '1px 6px',
                borderRadius: 3,
                fontWeight: 600,
              }}>
                ALARM
              </span>
            )}
          </div>
        )}
      </div>

      {/* Footer info */}
      {latest && (
        <div style={{
          padding: '4px 12px 6px',
          display: 'flex',
          justifyContent: 'space-between',
          fontSize: 10,
          color: 'var(--muted)',
        }}>
          <span>{formatSize(latest.file_size)}</span>
          <span>{snapshots.length} snapshot{snapshots.length !== 1 ? 's' : ''}</span>
        </div>
      )}
    </div>
  )
}
