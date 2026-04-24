import { useState, useEffect, useCallback, useRef, Suspense } from 'react'
import { Canvas, useThree } from '@react-three/fiber'
import { OrbitControls } from '@react-three/drei'
import * as THREE from 'three'
import { SensorConfig, SensorData, TwinEquipment, EquipmentType } from './types'
import { SolarPanelModel } from './twin/SolarPanelModel'
import { InverterModel } from './twin/InverterModel'
import { GatewayModel } from './twin/GatewayModel'
import { ConnectionLines } from './twin/ConnectionLines'
import { EquipmentLabel } from './twin/EquipmentLabel'
import { EquipmentPalette } from './twin/EquipmentPalette'
import { DEFAULT_EQUIPMENT } from './twin/defaults'

interface Props {
  sensors: SensorConfig[]
  currentValues: Record<string, SensorData>
  activeScreen: string
  twinLayouts: any[]
  onEquipmentClick: (equipmentId: string) => void
  onSwitchToCards?: () => void
  onSensorUpdate: (sensor: SensorConfig) => void
  onSaveLayouts: (equipment: TwinEquipment[]) => void
}

function GroundPlane() {
  return (
    <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -0.01, 0]} receiveShadow>
      <planeGeometry args={[20, 20]} />
      <meshStandardMaterial color="#1a1a2e" />
    </mesh>
  )
}

function GridHelper() {
  return <gridHelper args={[20, 20, '#333355', '#222244']} />
}

function DraggableEquipment({ equipment, onDragEnd, onClick, sensors, currentValues, editable, children }: {
  equipment: TwinEquipment
  onDragEnd: (id: string, pos: [number, number, number]) => void
  onClick: () => void
  sensors: SensorConfig[]
  currentValues: Record<string, SensorData>
  editable: boolean
  children: React.ReactNode
}) {
  const [hovered, setHovered] = useState(false)
  const [dragging, setDragging] = useState(false)
  const { camera, raycaster } = useThree()
  const plane = useRef(new THREE.Plane(new THREE.Vector3(0, 1, 0), 0))
  const intersection = useRef(new THREE.Vector3())
  const offset = useRef(new THREE.Vector3())
  const groupRef = useRef<THREE.Group>(null)

  const handlePointerDown = useCallback((e: any) => {
    if (!editable) return
    e.stopPropagation()
    setDragging(true)
    raycaster.ray.intersectPlane(plane.current, intersection.current)
    offset.current.copy(intersection.current).sub(groupRef.current!.position)
    offset.current.y = 0
  }, [editable, raycaster])

  const handlePointerMove = useCallback((e: any) => {
    if (!dragging || !groupRef.current) return
    e.stopPropagation()
    raycaster.ray.intersectPlane(plane.current, intersection.current)
    const newPos = intersection.current.sub(offset.current)
    groupRef.current.position.x = Math.round(newPos.x * 2) / 2
    groupRef.current.position.z = Math.round(newPos.z * 2) / 2
  }, [dragging, raycaster])

  const handlePointerUp = useCallback((e: any) => {
    if (!dragging || !groupRef.current) return
    setDragging(false)
    onDragEnd(equipment.id, [groupRef.current.position.x, 0, groupRef.current.position.z])
  }, [dragging, equipment.id, onDragEnd])

  return (
    <group
      ref={groupRef}
      position={equipment.position}
      rotation={[0, equipment.rotation, 0]}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerOver={() => setHovered(true)}
      onPointerOut={() => setHovered(false)}
      onClick={(e: any) => { e.stopPropagation(); onClick() }}
    >
      {children}
      {editable && hovered && (
        <mesh>
          <sphereGeometry args={[1.5, 16, 16]} />
          <meshBasicMaterial color="#06b6d4" wireframe transparent opacity={0.15} />
        </mesh>
      )}
      <EquipmentLabel
        name={equipment.name}
        sensors={sensors}
        currentValues={currentValues}
      />
    </group>
  )
}

export function DigitalTwinView({ sensors, currentValues, activeScreen, twinLayouts, onEquipmentClick, onSwitchToCards, onSensorUpdate, onSaveLayouts }: Props) {
  const [equipment, setEquipment] = useState<TwinEquipment[]>(DEFAULT_EQUIPMENT.map(e => ({ ...e })))
  const [twinEditMode, setTwinEditMode] = useState(false)
  const [bindingEquipmentId, setBindingEquipmentId] = useState<string | null>(null)

  const [webglSupported] = useState(() => {
    try {
      const canvas = document.createElement('canvas')
      return !!(canvas.getContext('webgl') || canvas.getContext('experimental-webgl'))
    } catch {
      return false
    }
  })

  if (!webglSupported) {
    return (
      <div style={{ height: 400, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10, gap: 12, color: 'var(--fg)' }}>
        <div style={{ fontSize: 16, fontWeight: 600 }}>WebGL Not Supported</div>
        <div style={{ fontSize: 13, color: 'var(--muted)' }}>Your browser does not support WebGL, which is required for the 3D view.</div>
        {onSwitchToCards && (
          <button onClick={onSwitchToCards} style={{ padding: '8px 16px', borderRadius: 6, border: '1px solid var(--cyan)', background: 'transparent', color: 'var(--cyan)', cursor: 'pointer', fontSize: 13 }}>Switch to Card View</button>
        )}
      </div>
    )
  }

  useEffect(() => {
    if (twinLayouts.length > 0) {
      setEquipment(prev => prev.map(eq => {
        const saved = twinLayouts.find((l: any) => l.equipment_id === eq.id)
        if (saved) return { ...eq, position: [saved.position_x, 0, saved.position_z], rotation: saved.rotation_y }
        return eq
      }))
    }
  }, [twinLayouts])

  const handleDragEnd = useCallback((id: string, pos: [number, number, number]) => {
    setEquipment(prev => prev.map(eq => eq.id === id ? { ...eq, position: pos } : eq))
  }, [])

  const handleAddEquipment = useCallback((type: EquipmentType) => {
    const id = `${type}_${Date.now()}`
    const names: Record<EquipmentType, string> = { solar_panel: 'Solar Panel', inverter: 'Inverter', gateway: 'Gateway' }
    setEquipment(prev => [...prev, { id, type, name: names[type], position: [Math.random() * 4 - 2, 0, Math.random() * 4 - 2] as [number, number, number], rotation: 0 }])
  }, [])

  const handleRemoveEquipment = useCallback((id: string) => {
    setEquipment(prev => prev.filter(eq => eq.id !== id))
  }, [])

  const getSensorsForEquipment = useCallback((eqId: string) => {
    return sensors.filter(s => s.equipment_id === eqId)
  }, [sensors])

  const toggleSensorBinding = useCallback((sensorId: number) => {
    const sensor = sensors.find(s => s.id === sensorId)
    if (!sensor) return
    const newEquipmentId = sensor.equipment_id === bindingEquipmentId ? null : bindingEquipmentId
    onSensorUpdate({ ...sensor, equipment_id: newEquipmentId })
  }, [sensors, bindingEquipmentId, onSensorUpdate])

  const renderModel = (eq: TwinEquipment) => {
    const eqSensors = getSensorsForEquipment(eq.id)
    let statusColor: string | undefined
    if (eqSensors.length > 0) {
      const hasCrit = eqSensors.some(s => {
        const cv = currentValues[s.key]
        return cv && cv.value >= cv.crit
      })
      const hasWarn = eqSensors.some(s => {
        const cv = currentValues[s.key]
        return cv && cv.value >= cv.warn
      })
      statusColor = hasCrit ? '#ef4444' : hasWarn ? '#eab308' : undefined
    }
    const common = {
      position: [0, 0, 0] as [number, number, number],
      rotation: 0,
      color: statusColor,
      onClick: () => { onEquipmentClick(eq.id) },
    }
    switch (eq.type) {
      case 'solar_panel': return <SolarPanelModel {...common} />
      case 'inverter': return <InverterModel {...common} />
      case 'gateway': return <GatewayModel {...common} />
    }
  }

  return (
    <div style={{ position: 'relative', width: '100%', height: 'calc(100vh - 180px)', minHeight: 500, background: '#0f0f1a', borderRadius: 10, overflow: 'hidden', border: '1px solid var(--border)' }}>
      {twinEditMode && <EquipmentPalette equipment={equipment} onAdd={handleAddEquipment} onRemove={handleRemoveEquipment} />}
      <div style={{ position: 'absolute', top: 8, right: 8, zIndex: 10, display: 'flex', gap: 6 }}>
        <button onClick={() => {
          if (twinEditMode) onSaveLayouts(equipment)
          setTwinEditMode(!twinEditMode)
        }} style={{
          padding: '6px 14px', borderRadius: 6, fontSize: 12, fontWeight: 600, cursor: 'pointer',
          border: twinEditMode ? '1px solid #06b6d4' : '1px solid rgba(255,255,255,0.2)',
          background: twinEditMode ? 'rgba(6,182,212,0.15)' : 'rgba(15,15,20,0.9)',
          color: twinEditMode ? '#06b6d4' : 'rgba(255,255,255,0.6)',
          fontFamily: 'Inter, sans-serif',
        }}>
          {twinEditMode ? 'Done Editing' : 'Edit Layout'}
        </button>
      </div>
      <Suspense fallback={<div style={{ position: 'absolute', inset: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--muted)' }}>Loading 3D...</div>}>
        <Canvas
          camera={{ position: [6, 6, 6], fov: 50 }}
          frameloop="always"
          dpr={[1, 2]}
          onCreated={({ camera }) => { camera.lookAt(0, 0, 0) }}
        >
          <ambientLight intensity={0.6} />
          <directionalLight position={[5, 8, 5]} intensity={0.8} />
          <GroundPlane />
          <GridHelper />
          <OrbitControls makeDefault enabled={!twinEditMode} enableDamping dampingFactor={0.1} maxPolarAngle={Math.PI / 2.2} />
          <ConnectionLines equipment={equipment} />
          {equipment.map(eq => (
            <DraggableEquipment key={eq.id} equipment={eq} onDragEnd={handleDragEnd} onClick={() => { twinEditMode ? setBindingEquipmentId(eq.id) : onEquipmentClick(eq.id) }} sensors={getSensorsForEquipment(eq.id)} currentValues={currentValues} editable={twinEditMode}>
              {renderModel(eq)}
            </DraggableEquipment>
          ))}
        </Canvas>
      </Suspense>
      {twinEditMode && bindingEquipmentId && (() => {
        const eq = equipment.find(e => e.id === bindingEquipmentId)
        if (!eq) return null
        const boundIds = new Set(sensors.filter(s => s.equipment_id === bindingEquipmentId).map(s => s.id))
        return (
          <div style={{
            position: 'absolute', bottom: 16, right: 16, background: 'rgba(15,15,20,0.95)',
            border: '1px solid rgba(255,255,255,0.15)', borderRadius: 10, padding: 16,
            fontFamily: 'Inter, sans-serif', width: 280, maxHeight: '60%', overflow: 'auto',
          }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 }}>
              <div>
                <div style={{ fontSize: 13, fontWeight: 600, color: '#e5e7eb' }}>{eq.name}</div>
                <div style={{ fontSize: 10, color: 'rgba(255,255,255,0.35)', marginTop: 2 }}>Select sensors to display</div>
              </div>
              <button onClick={() => setBindingEquipmentId(null)} style={{ background: 'none', border: 'none', color: '#9ca3af', cursor: 'pointer', fontSize: 18, lineHeight: 1 }}>&times;</button>
            </div>
            {sensors.length === 0 ? (
              <div style={{ fontSize: 11, color: 'rgba(255,255,255,0.3)' }}>No sensors available</div>
            ) : sensors.map(s => {
              const bound = boundIds.has(s.id)
              const cv = currentValues[s.key]
              return (
                <label key={s.id} style={{
                  display: 'flex', alignItems: 'center', gap: 10, padding: '6px 8px', marginBottom: 2,
                  borderRadius: 6, cursor: 'pointer',
                  background: bound ? 'rgba(6,182,212,0.1)' : 'transparent',
                  border: `1px solid ${bound ? 'rgba(6,182,212,0.3)' : 'rgba(255,255,255,0.05)'}`,
                }}>
                  <input type="checkbox" checked={bound} onChange={() => toggleSensorBinding(s.id)} style={{ accentColor: '#06b6d4' }} />
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div style={{ fontSize: 12, color: bound ? '#e5e7eb' : 'rgba(255,255,255,0.4)' }}>{s.name}</div>
                    <div style={{ fontSize: 10, color: 'rgba(255,255,255,0.25)' }}>{s.key} · {s.unit || '—'}</div>
                  </div>
                  {cv && (
                    <div style={{ fontFamily: 'JetBrains Mono, monospace', fontSize: 11, color: s.color, fontWeight: 600 }}>
                      {cv.value.toFixed(s.decimals)}
                    </div>
                  )}
                </label>
              )
            })}
          </div>
        )
      })()}
    </div>
  )
}
