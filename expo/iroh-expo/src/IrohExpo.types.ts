export type ChangeEventPayload = {
  value: string;
};

/// Options passed to [`IrohNode.new`]. Controls the behaviour of an iroh node.
export type NodeOptions = {
  /// How frequently the blob store should clean up unreferenced blobs, in milliseconds.
  /// Set to 0 to disable gc
  GCIntervalMillis: number
};


/// Intended capability for document share tickets
export enum ShareMode {
  /// Read-only access
  read = "read",
  /// Write access
  write = "write",
};


/// Options when creating a ticket
export enum AddrInfoOptions {
  /// Only the Node ID is added.
  ///
  /// This usually means that iroh-dns discovery is used to find address information.
  id = "id",
  /// Include both the relay URL and the direct addresses.
  relayAndAddresses = "relayAndAddresses",
  /// Only include the relay URL.
  relay = "relay",
  /// Only include the direct addresses.
  addresses = "addresses",
};
