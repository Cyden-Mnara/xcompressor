#!/usr/bin/env node
import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = dirname(fileURLToPath(import.meta.url))
const rootDir = resolve(scriptDir, '..')
const rawVersion = process.argv[2]

if (!rawVersion) {
  console.error('Usage: pnpm version:set <version>')
  console.error('Example: pnpm version:set 0.1.1')
  process.exit(1)
}

const version = rawVersion.trim().replace(/^v/, '')
const semverPattern = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/

if (!semverPattern.test(version)) {
  console.error(`Invalid version "${rawVersion}". Use semver like 0.1.1 or v0.1.1.`)
  process.exit(1)
}

const files = {
  tauriConfig: resolve(rootDir, 'src-tauri', 'tauri.conf.json'),
  cargoToml: resolve(rootDir, 'src-tauri', 'Cargo.toml'),
  cargoLock: resolve(rootDir, 'src-tauri', 'Cargo.lock')
}

const tauriConfig = JSON.parse(readFileSync(files.tauriConfig, 'utf8'))
tauriConfig.version = version
writeFileSync(files.tauriConfig, `${JSON.stringify(tauriConfig, null, 2)}\n`)

const cargoToml = readFileSync(files.cargoToml, 'utf8')
const cargoTomlVersionPattern = /(\[package\][\s\S]*?\nversion\s*=\s*")[^"]+(")/

if (!cargoTomlVersionPattern.test(cargoToml)) {
  console.error('Could not find [package] version in src-tauri/Cargo.toml.')
  process.exit(1)
}

const updatedCargoToml = cargoToml.replace(cargoTomlVersionPattern, `$1${version}$2`)
writeFileSync(files.cargoToml, updatedCargoToml)

const cargoLock = readFileSync(files.cargoLock, 'utf8')
const cargoLockVersionPattern = /(\[\[package\]\]\nname = "xcompressor"\nversion = ")[^"]+(")/

if (!cargoLockVersionPattern.test(cargoLock)) {
  console.error('Could not find xcompressor package version in src-tauri/Cargo.lock.')
  process.exit(1)
}

const updatedCargoLock = cargoLock.replace(cargoLockVersionPattern, `$1${version}$2`)
writeFileSync(files.cargoLock, updatedCargoLock)

console.log(`Set xcompressor version to ${version}.`)
