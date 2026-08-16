#!/usr/bin/env node

const { spawnSync } = require('node:child_process')
const { existsSync } = require('node:fs')
const { join } = require('node:path')

const binary = join(__dirname, process.platform === 'win32' ? 'hod.exe' : 'hod')

if (!existsSync(binary)) {
  process.stderr.write('error: the hod binary is absent. Run `npm rebuild hodstack`.\n')
  process.exit(1)
}

const result = spawnSync(binary, process.argv.slice(2), { stdio: 'inherit' })

process.exit(result.status === null ? 1 : result.status)
