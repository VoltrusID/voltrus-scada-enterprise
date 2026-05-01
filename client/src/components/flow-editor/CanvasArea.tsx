import { useRef, useMemo, useEffect, useState } from 'react'
import { FlowNode } from './FlowNode'
import { NODE_WIDTH, NODE_GAP, NODE_HEIGHT } from './constants'
import type { FlowNodeConfig } from '../../types'

interface Props {
  trigger: any
  nodes: FlowNodeConfig[]
  selectedIndex: number | null // null = trigger selected, >=0 = node index
  onSelectTrigger: () => void
  onSelectNode: (index: number) => void
  onRemoveNode: (index: number) => void
  onDragNodeStart: (index: number) => void
  onDrop: (insertIndex: number) => void
  isDragOver: boolean
}

export function CanvasArea({
  trigger, nodes, selectedIndex,
  onSelectTrigger, onSelectNode, onRemoveNode, onDragNodeStart, onDrop, isDragOver,
}: Props) {
  const canvasRef = useRef<HTMLDivElement>(null)
  const [canvasSize, setCanvasSize] = useState({ width: 800, height: 400 })

  useEffect(() => {
    const el = canvasRef.current
    if (!el) return
    const ro = new ResizeObserver(entries => {
      const cr = entries[0].contentRect
      setCanvasSize({ width: cr.width, height: cr.height })
    })
    ro.observe(el)
    return () => ro.disconnect()
  }, [])

  // Auto-layout: trigger + nodes, centered horizontally
  const positions = useMemo(() => {
    const totalItems = 1 + nodes.length
    const totalWidth = totalItems * NODE_WIDTH + (totalItems - 1) * NODE_GAP
    const startX = Math.max(60, (canvasSize.width - totalWidth) / 2)
    const y = canvasSize.height / 2 - NODE_HEIGHT / 2

    const pos: { x: number; y: number }[] = []
    for (let i = 0; i < totalItems; i++) {
      pos.push({ x: startX + i * (NODE_WIDTH + NODE_GAP), y })
    }
    return pos
  }, [nodes.length, canvasSize.width, canvasSize.height])

  // Connection points: right edge of item i to left edge of item i+1
  const edges = useMemo(() => {
    const result: { from: { x: number; y: number }; to: { x: number; y: number } }[] = []
    for (let i = 0; i < positions.length - 1; i++) {
      const from = { x: positions[i].x + NODE_WIDTH, y: positions[i].y + NODE_HEIGHT / 2 }
      const to = { x: positions[i + 1].x, y: positions[i + 1].y + NODE_HEIGHT / 2 }
      result.push({ from, to })
    }
    return result
  }, [positions])

  const handleMouseUp = (e: React.MouseEvent) => {
    if (!isDragOver) return
    const rect = canvasRef.current?.getBoundingClientRect()
    if (!rect) return
    const x = e.clientX - rect.left

    // Find nearest insertion gap
    const totalItems = 1 + nodes.length
    let bestIndex = totalItems
    let bestDist = Infinity

    // Check gaps: before trigger (0), between items, after last (totalItems)
    for (let i = 0; i <= totalItems; i++) {
      const gapX = i === 0
        ? positions[0]?.x - NODE_GAP / 2 || 0
        : i === totalItems
          ? (positions[totalItems - 1]?.x + NODE_WIDTH + NODE_GAP / 2) || canvasSize.width
          : (positions[i - 1].x + NODE_WIDTH + positions[i].x) / 2
      const dist = Math.abs(x - gapX)
      if (dist < bestDist) {
        bestDist = dist
        bestIndex = i
      }
    }
    onDrop(bestIndex)
  }

  return (
    <div
      ref={canvasRef}
      style={{
        flex: 1, position: 'relative', overflow: 'hidden',
        background: 'var(--bg)',
      }}
      onMouseUp={handleMouseUp}
    >
      {/* Dot grid background */}
      <svg style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', pointerEvents: 'none' }}>
        <defs>
          <pattern id="dotGrid" width="20" height="20" patternUnits="userSpaceOnUse">
            <circle cx="1" cy="1" r="1" fill="var(--border)" opacity="0.15" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#dotGrid)" />
      </svg>

      {/* Drop zone indicator */}
      {isDragOver && (
        <div style={{
          position: 'absolute', inset: 0,
          border: '2px dashed var(--green)', background: 'rgba(34,197,94,.05)',
          pointerEvents: 'none', zIndex: 5,
        }} />
      )}

      {/* SVG Connections */}
      <svg style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', pointerEvents: 'none', zIndex: 1 }}>
        <defs>
          <marker id="arrowhead" markerWidth="8" markerHeight="6" refX="7" refY="3" orient="auto">
            <polygon points="0 0, 8 3, 0 6" fill="#4b5563" />
          </marker>
        </defs>
        {edges.map((edge, i) => {
          const mx = (edge.from.x + edge.to.x) / 2
          const d = `M ${edge.from.x} ${edge.from.y} C ${mx} ${edge.from.y}, ${mx} ${edge.to.y}, ${edge.to.x} ${edge.to.y}`
          return <path key={i} d={d} fill="none" stroke="#4b5563" strokeWidth={2} markerEnd="url(#arrowhead)" />
        })}
      </svg>

      {/* Nodes */}
      {positions.map((pos, i) => {
        const isTriggerNode = i === 0
        const nodeIndex = i - 1
        return (
          <div
            key={isTriggerNode ? 'trigger' : nodes[nodeIndex].id}
            style={{
              position: 'absolute', left: pos.x, top: pos.y, zIndex: 2,
            }}
          >
            {isTriggerNode ? (
              <FlowNode
                isTrigger={true}
                triggerType={trigger?.type}
                isSelected={selectedIndex === null}
                index={0}
                onSelect={onSelectTrigger}
              />
            ) : (
              <FlowNode
                node={nodes[nodeIndex]}
                isTrigger={false}
                isSelected={selectedIndex === nodeIndex}
                index={nodeIndex}
                onSelect={() => onSelectNode(nodeIndex)}
                onRemove={() => onRemoveNode(nodeIndex)}
                onDragStart={onDragNodeStart}
              />
            )}
          </div>
        )
      })}
    </div>
  )
}
