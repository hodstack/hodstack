const { createHash } = require('node:crypto')
const { execFileSync } = require('node:child_process')
const { chmodSync, copyFileSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } = require('node:fs')
const { tmpdir } = require('node:os')
const { join } = require('node:path')

const REPOSITORY = 'hodstack/hodstack'
const TAG = process.env.HOD_TAG
const RELEASE = TAG
  ? `https://github.com/${REPOSITORY}/releases/download/${TAG}`
  : `https://github.com/${REPOSITORY}/releases/latest/download`

const TARGETS = {
  'darwin arm64': 'aarch64-apple-darwin',
  'darwin x64': 'x86_64-apple-darwin',
  'linux arm64': 'aarch64-unknown-linux-musl',
  'linux x64': 'x86_64-unknown-linux-musl',
  'win32 x64': 'x86_64-pc-windows-msvc',
}

function target() {
  const platform = `${process.platform} ${process.arch}`
  const found = TARGETS[platform]

  if (!found) {
    throw new Error(`no build for ${platform}`)
  }

  return found
}

async function download(address) {
  const response = await fetch(address)

  if (!response.ok) {
    throw new Error(`${address} gives ${response.status}`)
  }

  return Buffer.from(await response.arrayBuffer())
}

function verify(archive, checksums, file) {
  const line = checksums
    .toString()
    .split('\n')
    .find((row) => row.endsWith(` ${file}`))

  if (!line) {
    throw new Error(`checksums.txt names no ${file}`)
  }

  const wanted = line.split(' ')[0]
  const found = createHash('sha256').update(archive).digest('hex')

  if (wanted !== found) {
    throw new Error(`the checksum of ${file} does not agree with checksums.txt`)
  }
}

async function install() {
  const name = target()
  const file = `hod-${name}.tar.gz`
  const binary = process.platform === 'win32' ? 'hod.exe' : 'hod'

  const [archive, checksums] = await Promise.all([
    download(`${RELEASE}/${file}`),
    download(`${RELEASE}/checksums.txt`),
  ])

  verify(archive, checksums, file)

  const work = mkdtempSync(join(tmpdir(), 'hodstack-'))

  try {
    writeFileSync(join(work, file), archive)
    execFileSync('tar', ['-xzf', join(work, file), '-C', work, binary])

    const directory = join(__dirname, 'bin')
    mkdirSync(directory, { recursive: true })
    copyFileSync(join(work, binary), join(directory, binary))
    chmodSync(join(directory, binary), 0o755)
  } finally {
    rmSync(work, { recursive: true, force: true })
  }
}

install().catch((error) => {
  process.stderr.write(`error: ${error.message}\n`)
  process.exitCode = 1
})
