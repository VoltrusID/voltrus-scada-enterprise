import * as THREE from 'three'

interface Props {
  position: [number, number, number]
  rotation: number
  color?: string
  onClick?: () => void
}

export function GatewayModel({ position, rotation, color = '#7c3aed', onClick }: Props) {
  return (
    <group position={position} rotation={[0, rotation, 0]} onClick={onClick}>
      {/* Main body */}
      <mesh position={[0, 0.3, 0]}>
        <boxGeometry args={[0.5, 0.6, 0.3]} />
        <meshStandardMaterial color="#e5e7eb" />
      </mesh>
      {/* Antenna */}
      <mesh position={[0.15, 0.8, 0]}>
        <cylinderGeometry args={[0.02, 0.02, 0.4, 8]} />
        <meshStandardMaterial color="#6b7280" />
      </mesh>
      <mesh position={[0.15, 1.02, 0]}>
        <sphereGeometry args={[0.04, 8, 8]} />
        <meshStandardMaterial color={color} emissive={color} emissiveIntensity={0.4} />
      </mesh>
      {/* LED indicators */}
      <mesh position={[-0.1, 0.45, 0.16]}>
        <circleGeometry args={[0.025, 16]} />
        <meshStandardMaterial color="#22c55e" emissive="#22c55e" emissiveIntensity={0.5} />
      </mesh>
      <mesh position={[-0.18, 0.45, 0.16]}>
        <circleGeometry args={[0.025, 16]} />
        <meshStandardMaterial color="#f59e0b" emissive="#f59e0b" emissiveIntensity={0.5} />
      </mesh>
      {/* Mounting base */}
      <mesh position={[0, 0.01, 0]}>
        <boxGeometry args={[0.6, 0.02, 0.4]} />
        <meshStandardMaterial color="#9ca3af" />
      </mesh>
    </group>
  )
}
