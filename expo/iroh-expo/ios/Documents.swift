import ExpoModulesCore
import IrohLib

/**
 * Options when creating a ticket
 */
public enum AddrInfoOptionsString: String, Enumerable {
    /**
     * Only the Node ID is added.
     *
     * This usually means that iroh-dns discovery is used to find address information.
     */
    case id
    /**
     * Include both the relay URL and the direct addresses.
     */
    case relayAndAddresses
    /**
     * Only include the relay URL.
     */
    case relay
    /**
     * Only include the direct addresses.
     */
    case addresses
}

// convert to AddrInfoOptions
extension AddrInfoOptionsString {
  func toAddrInfoOptions() -> AddrInfoOptions {
    switch self {
    case .id:
      return AddrInfoOptions.id
    case .addresses:
      return AddrInfoOptions.addresses
    case .relay:
      return AddrInfoOptions.relay
    case .relayAndAddresses:
      return AddrInfoOptions.relayAndAddresses
    }
  }
}

/**
 * Intended capability for document share tickets
 */
public enum ShareModeString: String, Enumerable {
    /**
     * Read-only access
     */
    case read
    /**
     * Write access
     */
    case write
}

// convert to ShareMode
extension ShareModeString {
  func toShareMode() -> ShareMode {
    switch self {
    case .read:
      return ShareMode.read
    case .write:
      return ShareMode.write
    }
  }
}