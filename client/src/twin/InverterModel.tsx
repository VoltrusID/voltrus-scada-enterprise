interface Props {
  position: [number, number, number]
  rotation: number
  color?: string
  onClick?: () => void
}

export function InverterModel({ position, rotation, color = '#059669', onClick }: Props) {
  return (
    <group position={position} rotation={[0, rotation, 0]} onClick={onClick}>
      {/* Main body */}
      <mesh position={[0, 0.5, 0]}>
        <boxGeometry args={[0.8, 1.0, 0.4]} />
        <meshStandardMaterial color="#d1d5db" />
      </mesh>
      {/* Screen */}
      <mesh position={[0, 0.65, 0.21]}>
        <boxGeometry args={[0.5, 0.3, 0.01]} />
        <meshStandardMaterial color={color} emissive={color} emissiveIntensity={0.3} />
      </mesh>
      {/* Status LED */}
      <mesh position={[0.3, 0.2, 0.21]}>
        <circleGeometry args={[0.03, 16]} />
        <meshStandardMaterial color="#22c55e" emissive="#22c55e" emissiveIntensity={0.5} />
      </mesh>
      {/* Mounting bracket */}
      <mesh position={[0, 0.02, 0]}>
        <boxGeometry args={[1.0, 0.04, 0.6]} />
        <meshStandardMaterial color="#9ca3af" />
      </mesh>
    </group>
  )
}
