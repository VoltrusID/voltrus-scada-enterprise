// Type-checks the enterprise module against the core client it will be built with.
// tsconfig.json carries the paths for the official layout; this script rewrites
// them for whichever core checkout resolve-core.mjs finds, then runs core's tsc.
import fs from 'node:fs'
import path from 'node:path'
import { spawnSync } from 'node:child_process'
import { enterpriseClientDir, resolveCoreClientSrc } from './resolve-core.mjs'

const coreSrc = resolveCoreClientSrc()
const coreRoot = path.dirname(coreSrc)
const modules = path.join(coreRoot, 'node_modules')

const generated = path.join(enterpriseClientDir, 'tsconfig.generated.json')
fs.writeFileSync(generated, JSON.stringify({
  extends: './tsconfig.json',
  compilerOptions: {
    paths: {
      '@voltrus/core/*': [path.join(coreSrc, '*')],
      '@voltrus/enterprise-client': [path.join(enterpriseClientDir, 'src/index.ts')],
      // @types first: packages like react ship a typeless index.js that would shadow their typings.
      '*': [path.join(modules, '@types', '*'), path.join(modules, '*')],
    },
  },
  include: ['src'],
}, null, 2))

try {
  const tsc = path.join(modules, 'typescript', 'bin', 'tsc')
  const result = spawnSync(process.execPath, [tsc, '-p', generated, '--noEmit', ...process.argv.slice(2)], { stdio: 'inherit' })
  process.exitCode = result.status ?? 1
} finally {
  fs.rmSync(generated, { force: true })
}
