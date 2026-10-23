import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
export const enterpriseClientDir = path.resolve(here, '..')

// Core client checkout layouts, relative to enterprise/client:
//   official private repo: core is the git submodule   -> ../../core/client/src
//   current monorepo:      core client sits next to it -> ../../client/src
const LAYOUTS = ['../../core/client/src', '../../client/src']

/**
 * Absolute path of the core client `src` directory.
 * VOLTRUS_CORE_CLIENT_SRC (absolute, or relative to the cwd) wins; otherwise the
 * first known layout that contains the edition seam is used.
 */
export function resolveCoreClientSrc() {
  const fromEnv = process.env.VOLTRUS_CORE_CLIENT_SRC
  const candidates = fromEnv ? [path.resolve(fromEnv)] : LAYOUTS.map(p => path.resolve(enterpriseClientDir, p))
  const found = candidates.find(dir => fs.existsSync(path.join(dir, 'edition', 'types.ts')))
  if (!found) {
    throw new Error(
      `Core client src not found (looked in: ${candidates.join(', ')}). ` +
      'Set VOLTRUS_CORE_CLIENT_SRC to the absolute path of core/client/src.',
    )
  }
  return found
}
