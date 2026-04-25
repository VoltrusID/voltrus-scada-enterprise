interface Props {
  position: [number, number, number]
  rotation: number
  color?: string
  onClick?: () => void
}

export function SolarPanelModel({ position, rotation, color = '#1e40af', onClick }: Props) {
  return (
    <group position={position} rotation={[0, rotation, 0]} onClick={onClick}>
      {/* Panel frame */}
      <mesh position={[0, 1.2, 0]} rotation={[-Math.PI / 6, 0, 0]}>
        <boxGeometry args={[2.5, 0.05, 1.8]} />
        <meshStandardMaterial color="#374151" />
      </mesh>
      {/* Panel cells (grid pattern) */}
      {Array.from({ length: 2 }).map((_, row) =>
        Array.from({ length: 4 }).map((_, col) => (
          <mesh key={`${row}-${col}`} position={[-0.9 + col * 0.6, 1.22, -0.55 + row * 0.6]} rotation={[-Math.PI / 6, 0, 0]}>
            <boxGeometry args={[0.5, 0.02, 0.5]} />
            <meshStandardMaterial color={color} />
          </mesh>
        ))
      )}
      {/* Legs */}
      <mesh position={[-0.8, 0.5, 0.3]}>
        <boxGeometry args={[0.08, 1.0, 0.08]} />
        <meshStandardMaterial color="#6b7280" />
      </mesh>
      <mesh position={[0.8, 0.5, 0.3]}>
        <boxGeometry args={[0.08, 1.0, 0.08]} />
        <meshStandardMaterial color="#6b7280" />
      </mesh>
      <mesh position={[-0.8, 0.3, -0.5]}>
        <boxGeometry args={[0.08, 0.6, 0.08]} />
        <meshStandardMaterial color="#6b7280" />
      </mesh>
      <mesh position={[0.8, 0.3, -0.5]}>
        <boxGeometry args={[0.08, 0.6, 0.08]} />
        <meshStandardMaterial color="#6b7280" />
      </mesh>
    </group>
  )
}
