/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

/** A format identifier */
export const enum BlobFormat {
  /** Raw blob */
  Raw = 'Raw',
  /** A sequence of BLAKE3 hashes */
  HashSeq = 'HashSeq'
}
export const enum CapabilityKind {
  /** A writable replica. */
  Write = 1,
  /** A readable replica. */
  Read = 2
}
/** Intended capability for document share tickets */
export const enum ShareMode {
  /** Read-only access */
  Read = 0,
  /** Write access */
  Write = 1
}
/** Fields by which the query can be sorted */
export const enum SortBy {
  /** Sort by key, then author. */
  KeyAuthor = 0,
  /** Sort by author, then key. */
  AuthorKey = 1
}
/** Sort direction */
export const enum SortDirection {
  /** Sort ascending */
  Asc = 0,
  /** Sort descending */
  Desc = 1
}
/** Options for sorting and pagination for using [`Query`]s. */
export interface QueryOptions {
  /**
   * Sort by author or key first.
   *
   * Default is [`SortBy::AuthorKey`], so sorting first by author and then by key.
   */
  sortBy: SortBy
  /**
   * Direction by which to sort the entries
   *
   * Default is [`SortDirection::Asc`]
   */
  direction: SortDirection
  /** Offset */
  offset: number
  /**
   * Limit to limit the pagination.
   *
   * When the limit is 0, the limit does not exist.
   */
  limit: number
}
/**
 * Stats counter
 * Counter stats
 */
export interface CounterStats {
  /** The counter value */
  value: number
  /** The counter description */
  description: string
}
export interface ConnectionType {
  typ: ConnType
  data0?: string
  data1?: string
}
/** The type of the connection */
export const enum ConnType {
  /** Indicates you have a UDP connection. */
  Direct = 'Direct',
  /** Indicates you have a DERP relay connection. */
  Relay = 'Relay',
  /** Indicates you have an unverified UDP connection, and a relay connection for backup. */
  Mixed = 'Mixed',
  /** Indicates you have no proof of connection. */
  None = 'None'
}
/** The logging level. See the rust (log crate)[https://docs.rs/log] for more information. */
export const enum LogLevel {
  Trace = 'Trace',
  Debug = 'Debug',
  Info = 'Info',
  Warn = 'Warn',
  Error = 'Error',
  Off = 'Off'
}
/** Set the logging level. */
export function setLogLevel(level: LogLevel): void
/** Initialize the global metrics collection. */
export function startMetricsCollection(): void
/**
 * Helper function that translates a key that was derived from the [`path_to_key`] function back
 * into a path.
 *
 * If `prefix` exists, it will be stripped before converting back to a path
 * If `root` exists, will add the root as a parent to the created path
 * Removes any null byte that has been appened to the key
 */
export function keyToPath(key: Array<number>, prefix?: string | undefined | null, root?: string | undefined | null): string
/**
 * Helper function that creates a document key from a canonicalized path, removing the `root` and adding the `prefix`, if they exist
 *
 * Appends the null byte to the end of the key.
 */
export function pathToKey(path: string, prefix?: string | undefined | null, root?: string | undefined | null): Array<number>
/** Identifier for an [`Author`] */
export class AuthorId {
  /** Get an [`AuthorId`] from a String. */
  static fromString(str: string): this
  /** Returns true when both AuthorId's have the same value */
  equal(other: AuthorId): boolean
  /** String representation */
  toString(): string
}
/** Hash type used throughout Iroh. A blake3 hash. */
export class Hash {
  /** Calculate the hash of the provide bytes. */
  constructor(buf: Array<number>)
  /** Bytes of the hash. */
  toBytes(): Array<number>
  /** Create a `Hash` from its raw bytes representation. */
  static fromBytes(bytes: Array<number>): this
  /** Make a Hash from hex string */
  static fromString(s: string): this
  /** Convert the hash to a hex string. */
  toString(): string
  /** Returns true if the Hash's have the same value */
  equal(other: Hash): boolean
}
/** A collection of blobs */
export class Collection {
  /** Create a new empty collection */
  constructor()
  /** Add the given blob to the collection */
  push(name: string, hash: Hash): void
  /** Check if the collection is empty */
  isEmpty(): boolean
  /** Get the names of the blobs in this collection */
  names(): Array<string>
  /** Get the links to the blobs in this collection */
  links(): Array<Hash>
  /** Get the blobs associated with this collection */
  blobs(): Array<JsLinkAndName>
  /** Returns the number of blobs in this collection */
  len(): number
}
export type JsLinkAndName = LinkAndName
/** `LinkAndName` includes a name and a hash for a blob in a collection */
export class LinkAndName {
  /** The name associated with this [`Hash`]. */
  name: string
  /** The [`Hash`] of the blob. */
  link: string
}
/** The namespace id and CapabilityKind (read/write) of the doc */
export class NamespaceAndCapability {
  /** The namespace id of the doc */
  namespace: string
  /** The capability you have for the doc (read/write) */
  capability: CapabilityKind
}
export type JsDoc = Doc
/** A representation of a mutable, synchronizable key-value store. */
export class Doc {
  constructor(node: IrohNode)
  /** Get the document id of this doc. */
  get id(): string
  /** Close the document. */
  close(): Promise<void>
  /** Set the content of a key to a byte array. */
  setBytes(authorId: AuthorId, key: Array<number>, value: Array<number>): Promise<Hash>
  /** Set an entries on the doc via its key, hash, and size. */
  setHash(authorId: AuthorId, key: Array<number>, hash: Hash, size: bigint): Promise<void>
  /** Add an entry from an absolute file path */
  importFile(author: AuthorId, key: Array<number>, path: string, inPlace: boolean, cb?: (err: Error | null, arg: any) => any | undefined | null): Promise<void>
  /** Export an entry as a file to a given absolute path */
  exportFile(entry: Entry, path: string, cb?: (err: Error | null, arg: any) => any | undefined | null): Promise<void>
  /**
   * Delete entries that match the given `author` and key `prefix`.
   *
   * This inserts an empty entry with the key set to `prefix`, effectively clearing all other
   * entries whose key starts with or is equal to the given `prefix`.
   *
   * Returns the number of entries deleted.
   */
  del(authorId: AuthorId, prefix: Array<number>): Promise<bigint>
  /** Get an entry for a key and author. */
  getExact(author: AuthorId, key: Array<number>, includeEmpty: boolean): Promise<Entry | null>
  /**
   * Get entries.
   *
   * Note: this allocates for each `Entry`, if you have many `Entry`s this may be a prohibitively large list.
   * Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
   */
  getMany(query: Query): Promise<Array<Entry>>
  /** Get the latest entry for a key and author. */
  getOne(query: Query): Promise<Entry | null>
  /** Share this document with peers over a ticket. */
  share(mode: ShareMode): Promise<string>
  /** Start to sync this document with this peer. */
  startSync(peer: NodeAddr): Promise<void>
  /** Stop the live sync for this document. */
  leave(): Promise<void>
  /** Subscribe to events for this document. */
  subscribe(cb: (err: Error | null, arg: any) => any): Promise<void>
  /** Get status info for this document */
  status(): Promise<any>
  /** Get the download policy for this document */
  getDownloadPolicy(): Promise<any>
}
/** A peer and it's addressing information. */
export class NodeAddr {
  /** Create a new [`NodeAddr`] with empty [`AddrInfo`]. */
  constructor(nodeId: PublicKey, derpUrl: string | undefined | null, addresses: Array<string>)
  /** Get the direct addresses of this peer. */
  get directAddresses(): Array<string>
  /** Get the derp region of this peer. */
  get derpUrl(): string | null
  /** Returns true if both NodeAddr's have the same values */
  equal(other: NodeAddr): boolean
}
/**
 * A single entry in a [`Doc`]
 *
 * An entry is identified by a key, its [`AuthorId`], and the [`Doc`]'s
 * namespace id. Its value is the 32-byte BLAKE3 [`hash`]
 * of the entry's content data, the size of this content data, and a timestamp.
 */
export class Entry {
  /** Get the [`AuthorId`] of this entry. */
  author(): AuthorId
  /** Get the content_hash of this entry. */
  contentHash(): Hash
  /** Get the content_length of this entry. */
  contentLen(): number | null
  /** Get the key of this entry. */
  key(): Array<number>
  /** Get the namespace id of this entry. */
  namespace(): string
  /**
   * Read all content of an [`Entry`] into a buffer.
   * This allocates a buffer for the full entry. Use only if you know that the entry you're
   * reading is small. If not sure, use [`Self::content_len`] and check the size with
   * before calling [`Self::content_bytes`].
   */
  contentBytes(doc: Doc): Promise<Array<number>>
}
/**
 * Build a Query to search for an entry or entries in a doc.
 *
 * Use this with `QueryOptions` to determine sorting, grouping, and pagination.
 */
export class Query {
  /**
   * Query all records.
   *
   * If `opts` is `None`, the default values will be used:
   *     sort_by: SortBy::AuthorKey
   *     direction: SortDirection::Asc
   *     offset: None
   *     limit: None
   */
  static all(opts?: QueryOptions | undefined | null): Query
  /**
   * Query only the latest entry for each key, omitting older entries if the entry was written
   * to by multiple authors.
   *
   * If `opts` is `None`, the default values will be used:
   *     direction: SortDirection::Asc
   *     offset: None
   *     limit: None
   */
  static singleLatestPerKey(opts?: QueryOptions | undefined | null): Query
  /**
   * Query all entries for by a single author.
   *
   * If `opts` is `None`, the default values will be used:
   *     sort_by: SortBy::AuthorKey
   *     direction: SortDirection::Asc
   *     offset: None
   *     limit: None
   */
  static author(author: AuthorId, opts?: QueryOptions | undefined | null): Query
  /**
   * Query all entries that have an exact key.
   *
   * If `opts` is `None`, the default values will be used:
   *     sort_by: SortBy::AuthorKey
   *     direction: SortDirection::Asc
   *     offset: None
   *     limit: None
   */
  static keyExact(key: Array<number>, opts?: QueryOptions | undefined | null): Query
  /** Create a Query for a single key and author. */
  static authorKeyExact(author: AuthorId, key: Array<number>): Query
  /**
   * Create a query for all entries with a given key prefix.
   *
   * If `opts` is `None`, the default values will be used:
   *     sort_by: SortBy::AuthorKey
   *     direction: SortDirection::Asc
   *     offset: None
   *     limit: None
   */
  static keyPrefix(prefix: Array<number>, opts?: QueryOptions | undefined | null): Query
  /**
   * Create a query for all entries of a single author with a given key prefix.
   *
   * If `opts` is `None`, the default values will be used:
   *     direction: SortDirection::Asc
   *     offset: None
   *     limit: None
   */
  static authorKeyPrefix(author: AuthorId, prefix: Array<number>, opts?: QueryOptions | undefined | null): Query
  /** Get the limit for this query (max. number of entries to emit). */
  limit(): number | null
  /** Get the limit for this query (max. number of entries to emit). */
  offset(): number | null
}
/**
 * A public key.
 *
 * The key itself is just a 32 byte array, but a key has associated crypto
 * information that is cached for performance reasons.
 */
export class PublicKey {
  /** Returns true if the PublicKeys are equal */
  equal(other: PublicKey): boolean
  /** Express the PublicKey as a byte array */
  toBytes(): Array<number>
  /** Make a PublicKey from base32 string */
  static fromString(s: string): this
  /** Make a PublicKey from byte array */
  static fromBytes(bytes: Array<number>): this
  /**
   * Convert to a base32 string limited to the first 10 bytes for a friendly string
   * representation of the key.
   */
  fmtShort(): string
  /** String representation */
  toString(): string
}
/** Information about a direct address. */
export class DirectAddrInfo {
  /** Get the reported address */
  addr(): string
  /** Get the reported latency, if it exists, in milliseconds */
  latency(): number | null
  /** Get the last control message received by this node */
  lastControl(): JsLatencyAndControlMsg | null
  /** Get how long ago the last payload message was received for this node in milliseconds. */
  lastPayload(): number | null
}
export type JsLatencyAndControlMsg = LatencyAndControlMsg
/** The latency and type of the control message */
export class LatencyAndControlMsg {
  /** The latency of the control message, in milliseconds. */
  latency: number
  /** The type of control message, represented as a string */
  controlMsg: string
  constructor(latency: number, controlMsg: string)
}
export type JsConnectionInfo = ConnectionInfo
export class ConnectionInfo {
  /** Derp url, if available. */
  derpUrl?: string
  /** The type of connection we have to the peer, either direct or over relay. */
  connType: JsConnectionType
  /** The latency of the `conn_type` (in milliseconds). */
  latency?: number
  /** Duration since the last time this peer was used (in milliseconds). */
  lastUsed?: number
}
/** The socket address and url of the mixed connection */
export class ConnectionTypeMixed {
  /** Address of the node */
  addr: string
  /** Url of the DERP node to which the node is connected */
  derpUrl: string
  constructor(addr: string, derpUrl: string)
}
/** An Iroh node. Allows you to sync, store, and transfer data. */
export class IrohNode {
  /** Create a new author. */
  authorCreate(): Promise<AuthorId>
  /** List all the AuthorIds that exist on this node. */
  authorList(): Promise<Array<AuthorId>>
  /**
   * List all complete blobs.
   *
   * Note: this allocates for each `BlobListResponse`, if you have many `BlobListReponse`s this may be a prohibitively large list.
   * Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
   */
  blobsList(): Promise<Array<string>>
  /**
   * Get the size information on a single blob.
   *
   * Method only exist in FFI
   */
  blobsSize(hash: Hash): Promise<number>
  /**
   * Read all bytes of single blob.
   *
   * This allocates a buffer for the full blob. Use only if you know that the blob you're
   * reading is small.
   */
  blobsReadToBytes(hash: Hash): Promise<Array<number>>
  /**
   * Read all bytes of single blob at `offset` for length `len`.
   *
   * This allocates a buffer for the full length `len`. Use only if you know that the blob you're
   * reading is small.
   */
  blobsReadAtToBytes(hash: Hash, offset: number, len?: number | undefined | null): Promise<Array<number>>
  /**
   * Import a blob from a filesystem path.
   *
   * `path` should be an absolute path valid for the file system on which
   * the node runs.
   * If `in_place` is true, Iroh will assume that the data will not change and will share it in
   * place without copying to the Iroh data directory.
   */
  blobsAddFromPath(path: string, inPlace: boolean, tag: Array<number> | undefined | null, wrap: boolean, cb: (err: Error | null, arg: any) => any): Promise<void>
  /**
   * Export the blob contents to a file path
   * The `path` field is expected to be the absolute path.
   */
  blobsWriteToPath(hash: Hash, path: string): Promise<void>
  /** Write a blob by passing bytes. */
  blobsAddBytes(bytes: Array<number>, tag?: Array<number> | undefined | null): Promise<any>
  /** Download a blob from another node and add it to the local database. */
  blobsDownload(hash: Hash, format: BlobFormat, node: NodeAddr, tag: Array<number> | undefined | null, out: string | undefined | null, inPlace: boolean, cb: (err: Error | null, arg: any) => any): Promise<void>
  /**
   * List all incomplete (partial) blobs.
   *
   * Note: this allocates for each `BlobListIncompleteResponse`, if you have many `BlobListIncompleteResponse`s this may be a prohibitively large list.
   * Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
   */
  blobsListIncomplete(): Promise<Array<any>>
  /**
   * List all collections.
   *
   * Note: this allocates for each `BlobListCollectionsResponse`, if you have many `BlobListCollectionsResponse`s this may be a prohibitively large list.
   * Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
   */
  blobsListCollection(): Promise<Array<any>>
  /** Read the content of a collection */
  blobsGetCollection(hash: Hash): Promise<Collection>
  /**
   * Create a collection from already existing blobs.
   *
   * To automatically clear the tags for the passed in blobs you can set
   * `tags_to_delete` on those tags, and they will be deleted once the collection is created.
   */
  blobsCreateCollection(collection: Collection, tag: Array<number> | undefined | null, tagsToDelete: Array<string>): Promise<any>
  /** Delete a blob. */
  blobsDeleteBlob(hash: Hash): Promise<void>
  docCreate(): Promise<JsDoc>
  /** Join and sync with an already existing document. */
  docJoin(ticket: string): Promise<JsDoc>
  /** List all the docs we have access to on this node. */
  docList(): Promise<Array<NamespaceAndCapability>>
  /**
   * Get a [`Doc`].
   *
   * Returns None if the document cannot be found.
   */
  docOpen(id: string): Promise<JsDoc | null>
  /**
   * Delete a document from the local node.
   *
   * This is a destructive operation. Both the document secret key and all entries in the
   * document will be permanently deleted from the node's storage. Content blobs will be delted
   * through garbage collection unless they are referenced from another document or tag.
   */
  docDrop(docId: string): Promise<void>
  /**
   * Create a new iroh node. The `path` param should be a directory where we can store or load
   * iroh data from a previous session.
   */
  static withPath(path: string): Promise<IrohNode>
  /** The string representation of the PublicKey of this node. */
  nodeId(): string
  /** Get statistics of the running node. */
  stats(): Promise<Record<string, CounterStats>>
  /** Return `ConnectionInfo`s for each connection we have to another iroh node. */
  connections(): Promise<Array<ConnectionInfo>>
  /** Return connection information on the currently running node. */
  connectionInfo(nodeId: PublicKey): Promise<ConnectionInfo | null>
  /** Get status information about a node */
  status(): Promise<NodeStatusResponse>
}
/** The response to a status request */
export class NodeStatusResponse {
  /** The node id and socket addresses of this node. */
  nodeAddr(): NodeAddr
  /** The bound listening addresses of the node */
  listenAddrs(): Array<string>
  /** The version of the node */
  version(): string
}
