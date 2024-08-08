/* tslint:disable */
/* eslint-disable */
/* prettier-ignore */

/* auto-generated by NAPI-RS */

const { existsSync, readFileSync } = require('fs')
const { join } = require('path')

const { platform, arch } = process

let nativeBinding = null
let localFileExisted = false
let loadError = null

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      const lddPath = require('child_process').execSync('which ldd').toString().trim()
      return readFileSync(lddPath, 'utf8').includes('musl')
    } catch (e) {
      return true
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header
    return !glibcVersionRuntime
  }
}

switch (platform) {
  case 'android':
    switch (arch) {
      case 'arm64':
        localFileExisted = existsSync(join(__dirname, 'iroh.android-arm64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./iroh.android-arm64.node')
          } else {
            nativeBinding = require('@number0/iroh-android-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm':
        localFileExisted = existsSync(join(__dirname, 'iroh.android-arm-eabi.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./iroh.android-arm-eabi.node')
          } else {
            nativeBinding = require('@number0/iroh-android-arm-eabi')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Android ${arch}`)
    }
    break
  case 'win32':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(
          join(__dirname, 'iroh.win32-x64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./iroh.win32-x64-msvc.node')
          } else {
            nativeBinding = require('@number0/iroh-win32-x64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'ia32':
        localFileExisted = existsSync(
          join(__dirname, 'iroh.win32-ia32-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./iroh.win32-ia32-msvc.node')
          } else {
            nativeBinding = require('@number0/iroh-win32-ia32-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'iroh.win32-arm64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./iroh.win32-arm64-msvc.node')
          } else {
            nativeBinding = require('@number0/iroh-win32-arm64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`)
    }
    break
  case 'darwin':
    localFileExisted = existsSync(join(__dirname, 'iroh.darwin-universal.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./iroh.darwin-universal.node')
      } else {
        nativeBinding = require('@number0/iroh-darwin-universal')
      }
      break
    } catch {}
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(join(__dirname, 'iroh.darwin-x64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./iroh.darwin-x64.node')
          } else {
            nativeBinding = require('@number0/iroh-darwin-x64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'iroh.darwin-arm64.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./iroh.darwin-arm64.node')
          } else {
            nativeBinding = require('@number0/iroh-darwin-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`)
    }
    break
  case 'freebsd':
    if (arch !== 'x64') {
      throw new Error(`Unsupported architecture on FreeBSD: ${arch}`)
    }
    localFileExisted = existsSync(join(__dirname, 'iroh.freebsd-x64.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./iroh.freebsd-x64.node')
      } else {
        nativeBinding = require('@number0/iroh-freebsd-x64')
      }
    } catch (e) {
      loadError = e
    }
    break
  case 'linux':
    switch (arch) {
      case 'x64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'iroh.linux-x64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./iroh.linux-x64-musl.node')
            } else {
              nativeBinding = require('@number0/iroh-linux-x64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'iroh.linux-x64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./iroh.linux-x64-gnu.node')
            } else {
              nativeBinding = require('@number0/iroh-linux-x64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'iroh.linux-arm64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./iroh.linux-arm64-musl.node')
            } else {
              nativeBinding = require('@number0/iroh-linux-arm64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'iroh.linux-arm64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./iroh.linux-arm64-gnu.node')
            } else {
              nativeBinding = require('@number0/iroh-linux-arm64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'iroh.linux-arm-musleabihf.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./iroh.linux-arm-musleabihf.node')
            } else {
              nativeBinding = require('@number0/iroh-linux-arm-musleabihf')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'iroh.linux-arm-gnueabihf.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./iroh.linux-arm-gnueabihf.node')
            } else {
              nativeBinding = require('@number0/iroh-linux-arm-gnueabihf')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'riscv64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'iroh.linux-riscv64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./iroh.linux-riscv64-musl.node')
            } else {
              nativeBinding = require('@number0/iroh-linux-riscv64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'iroh.linux-riscv64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./iroh.linux-riscv64-gnu.node')
            } else {
              nativeBinding = require('@number0/iroh-linux-riscv64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 's390x':
        localFileExisted = existsSync(
          join(__dirname, 'iroh.linux-s390x-gnu.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./iroh.linux-s390x-gnu.node')
          } else {
            nativeBinding = require('@number0/iroh-linux-s390x-gnu')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`)
    }
    break
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`)
}

if (!nativeBinding) {
  if (loadError) {
    throw loadError
  }
  throw new Error(`Failed to load native binding`)
}

const { AuthorId, Author, Authors, Blobs, SetTagOption, Hash, BlobFormat, BlobDownloadOptions, BlobExportFormat, BlobExportMode, RangeSpec, Collection, CapabilityKind, Docs, Doc, DownloadPolicy, FilterKind, ShareMode, SortBy, SortDirection, Query, SyncReason, Origin, ContentStatus, DocImportProgressType, PublicKey, ConnType, Iroh, Node, BlobTicket, AddrInfoOptions, LogLevel, setLogLevel, startMetricsCollection, keyToPath, pathToKey } = nativeBinding

module.exports.AuthorId = AuthorId
module.exports.Author = Author
module.exports.Authors = Authors
module.exports.Blobs = Blobs
module.exports.SetTagOption = SetTagOption
module.exports.Hash = Hash
module.exports.BlobFormat = BlobFormat
module.exports.BlobDownloadOptions = BlobDownloadOptions
module.exports.BlobExportFormat = BlobExportFormat
module.exports.BlobExportMode = BlobExportMode
module.exports.RangeSpec = RangeSpec
module.exports.Collection = Collection
module.exports.CapabilityKind = CapabilityKind
module.exports.Docs = Docs
module.exports.Doc = Doc
module.exports.DownloadPolicy = DownloadPolicy
module.exports.FilterKind = FilterKind
module.exports.ShareMode = ShareMode
module.exports.SortBy = SortBy
module.exports.SortDirection = SortDirection
module.exports.Query = Query
module.exports.SyncReason = SyncReason
module.exports.Origin = Origin
module.exports.ContentStatus = ContentStatus
module.exports.DocImportProgressType = DocImportProgressType
module.exports.PublicKey = PublicKey
module.exports.ConnType = ConnType
module.exports.Iroh = Iroh
module.exports.Node = Node
module.exports.BlobTicket = BlobTicket
module.exports.AddrInfoOptions = AddrInfoOptions
module.exports.LogLevel = LogLevel
module.exports.setLogLevel = setLogLevel
module.exports.startMetricsCollection = startMetricsCollection
module.exports.keyToPath = keyToPath
module.exports.pathToKey = pathToKey
