// Vite config for building the official client (core + enterprise module).
// Run it through `npm run build` / `npm run dev` here (scripts/vite.mjs), or
// directly from the core client directory:
//   VOLTRUS_ENTERPRISE_CLIENT=<abs path to src/index.ts> npm run build
import path from 'node:path'
import { pathToFileURL } from 'node:url'
import type { UserConfig } from 'vite'
import { enterpriseClientDir, resolveCoreClientSrc } from './scripts/resolve-core.mjs'

const coreSrc = resolveCoreClientSrc()
process.env.VOLTRUS_CORE_CLIENT_SRC = coreSrc
process.env.VOLTRUS_ENTERPRISE_CLIENT ??= path.join(enterpriseClientDir, 'src/index.ts')

// The core config owns everything else (root, plugins, aliases, chunking).
const core = await import(pathToFileURL(path.join(path.dirname(coreSrc), 'vite.config.ts')).href)
const config: UserConfig = core.default
export default config
