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
        localFileExisted = existsSync(join(__dirname, 'index.android-arm64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./index.android-arm64.node')
          } else {
            nativeBinding = require('undefined-android-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm':
        localFileExisted = existsSync(join(__dirname, 'index.android-arm-eabi.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./index.android-arm-eabi.node')
          } else {
            nativeBinding = require('undefined-android-arm-eabi')
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
          join(__dirname, 'index.win32-x64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./index.win32-x64-msvc.node')
          } else {
            nativeBinding = require('undefined-win32-x64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'ia32':
        localFileExisted = existsSync(
          join(__dirname, 'index.win32-ia32-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./index.win32-ia32-msvc.node')
          } else {
            nativeBinding = require('undefined-win32-ia32-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'index.win32-arm64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./index.win32-arm64-msvc.node')
          } else {
            nativeBinding = require('undefined-win32-arm64-msvc')
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
    localFileExisted = existsSync(join(__dirname, 'index.darwin-universal.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./index.darwin-universal.node')
      } else {
        nativeBinding = require('undefined-darwin-universal')
      }
      break
    } catch {}
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(join(__dirname, 'index.darwin-x64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./index.darwin-x64.node')
          } else {
            nativeBinding = require('undefined-darwin-x64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'index.darwin-arm64.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./index.darwin-arm64.node')
          } else {
            nativeBinding = require('undefined-darwin-arm64')
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
    localFileExisted = existsSync(join(__dirname, 'index.freebsd-x64.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./index.freebsd-x64.node')
      } else {
        nativeBinding = require('undefined-freebsd-x64')
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
            join(__dirname, 'index.linux-x64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./index.linux-x64-musl.node')
            } else {
              nativeBinding = require('undefined-linux-x64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'index.linux-x64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./index.linux-x64-gnu.node')
            } else {
              nativeBinding = require('undefined-linux-x64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'index.linux-arm64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./index.linux-arm64-musl.node')
            } else {
              nativeBinding = require('undefined-linux-arm64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'index.linux-arm64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./index.linux-arm64-gnu.node')
            } else {
              nativeBinding = require('undefined-linux-arm64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm':
        localFileExisted = existsSync(
          join(__dirname, 'index.linux-arm-gnueabihf.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./index.linux-arm-gnueabihf.node')
          } else {
            nativeBinding = require('undefined-linux-arm-gnueabihf')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'riscv64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'index.linux-riscv64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./index.linux-riscv64-musl.node')
            } else {
              nativeBinding = require('undefined-linux-riscv64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'index.linux-riscv64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./index.linux-riscv64-gnu.node')
            } else {
              nativeBinding = require('undefined-linux-riscv64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 's390x':
        localFileExisted = existsSync(
          join(__dirname, 'index.linux-s390x-gnu.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./index.linux-s390x-gnu.node')
          } else {
            nativeBinding = require('undefined-linux-s390x-gnu')
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

const { AuthorId, Hash, JsAddProgress, JsDownloadProgress, AddProgressType, BlobFormat, Collection, LinkAndName, CapabilityKind, NamespaceAndCapability, Doc, DocSubscriber, JsDocImportProgress, JsDocExportProgress, NodeAddr, ShareMode, Entry, SortBy, SortDirection, Query, PublicKey, DirectAddrInfo, LatencyAndControlMsg, ConnectionInfo, ConnType, ConnectionTypeMixed, IrohNode, NodeStatusResponse, LogLevel, setLogLevel, startMetricsCollection, keyToPath, pathToKey } = nativeBinding

module.exports.AuthorId = AuthorId
module.exports.Hash = Hash
module.exports.JsAddProgress = JsAddProgress
module.exports.JsDownloadProgress = JsDownloadProgress
module.exports.AddProgressType = AddProgressType
module.exports.BlobFormat = BlobFormat
module.exports.Collection = Collection
module.exports.LinkAndName = LinkAndName
module.exports.CapabilityKind = CapabilityKind
module.exports.NamespaceAndCapability = NamespaceAndCapability
module.exports.Doc = Doc
module.exports.DocSubscriber = DocSubscriber
module.exports.JsDocImportProgress = JsDocImportProgress
module.exports.JsDocExportProgress = JsDocExportProgress
module.exports.NodeAddr = NodeAddr
module.exports.ShareMode = ShareMode
module.exports.Entry = Entry
module.exports.SortBy = SortBy
module.exports.SortDirection = SortDirection
module.exports.Query = Query
module.exports.PublicKey = PublicKey
module.exports.DirectAddrInfo = DirectAddrInfo
module.exports.LatencyAndControlMsg = LatencyAndControlMsg
module.exports.ConnectionInfo = ConnectionInfo
module.exports.ConnType = ConnType
module.exports.ConnectionTypeMixed = ConnectionTypeMixed
module.exports.IrohNode = IrohNode
module.exports.NodeStatusResponse = NodeStatusResponse
module.exports.LogLevel = LogLevel
module.exports.setLogLevel = setLogLevel
module.exports.startMetricsCollection = startMetricsCollection
module.exports.keyToPath = keyToPath
module.exports.pathToKey = pathToKey
