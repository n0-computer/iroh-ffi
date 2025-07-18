// prettier-ignore
/* eslint-disable */
// @ts-nocheck
/* auto-generated by NAPI-RS */

const { createRequire } = require('node:module')
require = createRequire(__filename)

const { readFileSync } = require('node:fs')
let nativeBinding = null
const loadErrors = []

const isMusl = () => {
  let musl = false
  if (process.platform === 'linux') {
    musl = isMuslFromFilesystem()
    if (musl === null) {
      musl = isMuslFromReport()
    }
    if (musl === null) {
      musl = isMuslFromChildProcess()
    }
  }
  return musl
}

const isFileMusl = (f) => f.includes('libc.musl-') || f.includes('ld-musl-')

const isMuslFromFilesystem = () => {
  try {
    return readFileSync('/usr/bin/ldd', 'utf-8').includes('musl')
  } catch {
    return null
  }
}

const isMuslFromReport = () => {
  let report = null
  if (typeof process.report?.getReport === 'function') {
    process.report.excludeNetwork = true
    report = process.report.getReport()
  }
  if (!report) {
    return null
  }
  if (report.header && report.header.glibcVersionRuntime) {
    return false
  }
  if (Array.isArray(report.sharedObjects)) {
    if (report.sharedObjects.some(isFileMusl)) {
      return true
    }
  }
  return false
}

const isMuslFromChildProcess = () => {
  try {
    return require('child_process').execSync('ldd --version', { encoding: 'utf8' }).includes('musl')
  } catch (e) {
    // If we reach this case, we don't know if the system is musl or not, so is better to just fallback to false
    return false
  }
}

function requireNative() {
  if (process.env.NAPI_RS_NATIVE_LIBRARY_PATH) {
    try {
      nativeBinding = require(process.env.NAPI_RS_NATIVE_LIBRARY_PATH);
    } catch (err) {
      loadErrors.push(err)
    }
  } else if (process.platform === 'android') {
    if (process.arch === 'arm64') {
      try {
        return require('./iroh.android-arm64.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-android-arm64')
      } catch (e) {
        loadErrors.push(e)
      }

    } else if (process.arch === 'arm') {
      try {
        return require('./iroh.android-arm-eabi.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-android-arm-eabi')
      } catch (e) {
        loadErrors.push(e)
      }

    } else {
      loadErrors.push(new Error(`Unsupported architecture on Android ${process.arch}`))
    }
  } else if (process.platform === 'win32') {
    if (process.arch === 'x64') {
      try {
        return require('./iroh.win32-x64-msvc.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-win32-x64-msvc')
      } catch (e) {
        loadErrors.push(e)
      }

    } else if (process.arch === 'ia32') {
      try {
        return require('./iroh.win32-ia32-msvc.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-win32-ia32-msvc')
      } catch (e) {
        loadErrors.push(e)
      }

    } else if (process.arch === 'arm64') {
      try {
        return require('./iroh.win32-arm64-msvc.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-win32-arm64-msvc')
      } catch (e) {
        loadErrors.push(e)
      }

    } else {
      loadErrors.push(new Error(`Unsupported architecture on Windows: ${process.arch}`))
    }
  } else if (process.platform === 'darwin') {
    try {
        return require('./iroh.darwin-universal.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-darwin-universal')
      } catch (e) {
        loadErrors.push(e)
      }

    if (process.arch === 'x64') {
      try {
        return require('./iroh.darwin-x64.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-darwin-x64')
      } catch (e) {
        loadErrors.push(e)
      }

    } else if (process.arch === 'arm64') {
      try {
        return require('./iroh.darwin-arm64.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-darwin-arm64')
      } catch (e) {
        loadErrors.push(e)
      }

    } else {
      loadErrors.push(new Error(`Unsupported architecture on macOS: ${process.arch}`))
    }
  } else if (process.platform === 'freebsd') {
    if (process.arch === 'x64') {
      try {
        return require('./iroh.freebsd-x64.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-freebsd-x64')
      } catch (e) {
        loadErrors.push(e)
      }

    } else if (process.arch === 'arm64') {
      try {
        return require('./iroh.freebsd-arm64.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-freebsd-arm64')
      } catch (e) {
        loadErrors.push(e)
      }

    } else {
      loadErrors.push(new Error(`Unsupported architecture on FreeBSD: ${process.arch}`))
    }
  } else if (process.platform === 'linux') {
    if (process.arch === 'x64') {
      if (isMusl()) {
        try {
        return require('./iroh.linux-x64-musl.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-linux-x64-musl')
      } catch (e) {
        loadErrors.push(e)
      }

      } else {
        try {
        return require('./iroh.linux-x64-gnu.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-linux-x64-gnu')
      } catch (e) {
        loadErrors.push(e)
      }

      }
    } else if (process.arch === 'arm64') {
      if (isMusl()) {
        try {
        return require('./iroh.linux-arm64-musl.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-linux-arm64-musl')
      } catch (e) {
        loadErrors.push(e)
      }

      } else {
        try {
        return require('./iroh.linux-arm64-gnu.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-linux-arm64-gnu')
      } catch (e) {
        loadErrors.push(e)
      }

      }
    } else if (process.arch === 'arm') {
      if (isMusl()) {
        try {
        return require('./iroh.linux-arm-musleabihf.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-linux-arm-musleabihf')
      } catch (e) {
        loadErrors.push(e)
      }

      } else {
        try {
        return require('./iroh.linux-arm-gnueabihf.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-linux-arm-gnueabihf')
      } catch (e) {
        loadErrors.push(e)
      }

      }
    } else if (process.arch === 'riscv64') {
      if (isMusl()) {
        try {
        return require('./iroh.linux-riscv64-musl.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-linux-riscv64-musl')
      } catch (e) {
        loadErrors.push(e)
      }

      } else {
        try {
        return require('./iroh.linux-riscv64-gnu.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-linux-riscv64-gnu')
      } catch (e) {
        loadErrors.push(e)
      }

      }
    } else if (process.arch === 'ppc64') {
      try {
        return require('./iroh.linux-ppc64-gnu.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-linux-ppc64-gnu')
      } catch (e) {
        loadErrors.push(e)
      }

    } else if (process.arch === 's390x') {
      try {
        return require('./iroh.linux-s390x-gnu.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        return require('@number0/iroh-linux-s390x-gnu')
      } catch (e) {
        loadErrors.push(e)
      }

    } else {
      loadErrors.push(new Error(`Unsupported architecture on Linux: ${process.arch}`))
    }
  } else {
    loadErrors.push(new Error(`Unsupported OS: ${process.platform}, architecture: ${process.arch}`))
  }
}

nativeBinding = requireNative()

if (!nativeBinding || process.env.NAPI_RS_FORCE_WASI) {
  try {
    nativeBinding = require('./iroh.wasi.cjs')
  } catch (err) {
    if (process.env.NAPI_RS_FORCE_WASI) {
      loadErrors.push(err)
    }
  }
  if (!nativeBinding) {
    try {
      nativeBinding = require('@number0/iroh-wasm32-wasi')
    } catch (err) {
      if (process.env.NAPI_RS_FORCE_WASI) {
        loadErrors.push(err)
      }
    }
  }
}

if (!nativeBinding) {
  if (loadErrors.length > 0) {
    // TODO Link to documentation with potential fixes
    //  - The package owner could build/publish bindings for this arch
    //  - The user may need to bundle the correct files
    //  - The user may need to re-install node_modules to get new packages
    throw new Error('Failed to load native binding', { cause: loadErrors })
  }
  throw new Error(`Failed to load native binding`)
}

module.exports = nativeBinding
module.exports.Author = nativeBinding.Author
module.exports.AuthorId = nativeBinding.AuthorId
module.exports.Authors = nativeBinding.Authors
module.exports.BiStream = nativeBinding.BiStream
module.exports.BlobDownloadOptions = nativeBinding.BlobDownloadOptions
module.exports.Blobs = nativeBinding.Blobs
module.exports.BlobTicket = nativeBinding.BlobTicket
module.exports.Collection = nativeBinding.Collection
module.exports.Connecting = nativeBinding.Connecting
module.exports.Connection = nativeBinding.Connection
module.exports.Doc = nativeBinding.Doc
module.exports.Docs = nativeBinding.Docs
module.exports.DocTicket = nativeBinding.DocTicket
module.exports.DownloadPolicy = nativeBinding.DownloadPolicy
module.exports.Endpoint = nativeBinding.Endpoint
module.exports.FilterKind = nativeBinding.FilterKind
module.exports.Gossip = nativeBinding.Gossip
module.exports.Hash = nativeBinding.Hash
module.exports.Iroh = nativeBinding.Iroh
module.exports.Net = nativeBinding.Net
module.exports.Node = nativeBinding.Node
module.exports.PublicKey = nativeBinding.PublicKey
module.exports.Query = nativeBinding.Query
module.exports.RangeSpec = nativeBinding.RangeSpec
module.exports.RecvStream = nativeBinding.RecvStream
module.exports.Sender = nativeBinding.Sender
module.exports.SendStream = nativeBinding.SendStream
module.exports.SetTagOption = nativeBinding.SetTagOption
module.exports.TagInfo = nativeBinding.TagInfo
module.exports.Tags = nativeBinding.Tags
module.exports.AddrInfoOptions = nativeBinding.AddrInfoOptions
module.exports.BlobExportFormat = nativeBinding.BlobExportFormat
module.exports.BlobExportMode = nativeBinding.BlobExportMode
module.exports.BlobFormat = nativeBinding.BlobFormat
module.exports.CapabilityKind = nativeBinding.CapabilityKind
module.exports.ConnType = nativeBinding.ConnType
module.exports.ContentStatus = nativeBinding.ContentStatus
module.exports.DocImportProgressType = nativeBinding.DocImportProgressType
module.exports.keyToPath = nativeBinding.keyToPath
module.exports.LogLevel = nativeBinding.LogLevel
module.exports.NodeDiscoveryConfig = nativeBinding.NodeDiscoveryConfig
module.exports.Origin = nativeBinding.Origin
module.exports.pathToKey = nativeBinding.pathToKey
module.exports.ReadAtLenType = nativeBinding.ReadAtLenType
module.exports.setLogLevel = nativeBinding.setLogLevel
module.exports.ShareMode = nativeBinding.ShareMode
module.exports.SortBy = nativeBinding.SortBy
module.exports.SortDirection = nativeBinding.SortDirection
module.exports.SyncReason = nativeBinding.SyncReason
module.exports.verifyNodeAddr = nativeBinding.verifyNodeAddr
