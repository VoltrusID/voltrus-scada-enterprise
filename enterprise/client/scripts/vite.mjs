// Runs core's vite with the enterprise config:  node scripts/vite.mjs build --outDir /tmp/out
import path from 'node:path'
import { spawnSync } from 'node:child_process'
import { enterpriseClientDir, resolveCoreClientSrc } from './resolve-core.mjs'

const coreSrc = resolveCoreClientSrc()
const coreRoot = path.dirname(coreSrc)
const vite = path.join(coreRoot, 'node_modules', 'vite', 'bin', 'vite.js')

const result = spawnSync(
  process.execPath,
  [vite, ...process.argv.slice(2), '--config', path.join(enterpriseClientDir, 'vite.config.ts')],
  { cwd: coreRoot, stdio: 'inherit', env: { ...process.env, VOLTRUS_CORE_CLIENT_SRC: coreSrc } },
)
process.exitCode = result.status ?? 1
