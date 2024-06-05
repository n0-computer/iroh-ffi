import ExpoModulesCore
import IrohLib

public class IrohExpoModule: Module {
  private var node: IrohNode?

  private func irohPath() -> URL {
    let paths = FileManager.default.urls(for: .libraryDirectory, in: .userDomainMask)
    let irohPath = paths[0].appendingPathComponent("iroh")
    mkdirP(path: irohPath.path)
    return irohPath
  }

  public func definition() -> ModuleDefinition {
    Name("IrohExpo")

    OnCreate {
      do {
        // IrohLib.setLogLevel(level: .debug)
        // try IrohLib.startMetricsCollection()
        let path = self.irohPath()
        self.node = try IrohNode(path: path.path)
      } catch {
        print("error creating iroh node \(error)")
      }
    }

    AsyncFunction("nodeId") {
      return self.node?.nodeId()
    }

    AsyncFunction("docCreate") {
      guard let doc = try self.node?.docCreate() else {
        throw IrohError.Doc(description: "error creating doc")
      }
      return doc.id();
    }

    AsyncFunction("docDrop") { (docId: String) in
      return try self.node?.docDrop(id: docId)
    }

    AsyncFunction("docOpen") { (docId: String) in
      return try self.node?.docOpen(id: docId)
    }

    AsyncFunction("docJoin") { (ticket: String) in 
      return try self.node?.docJoin(ticket: ticket)
    }

    AsyncFunction("docShare") { (docId: String, mode: ShareModeString, addrOptions: AddrInfoOptionsString) in
      guard let doc = try self.node?.docOpen(id: docId) else {
        throw IrohError.Doc(description: "error opening doc")
      }
      return try doc.share(mode: mode.toShareMode(), addrOptions: addrOptions.toAddrInfoOptions())
    }

    AsyncFunction("docSetBytes") { (docId: string, author: AuthorId, key: Data, bytes: Data) in
      guard let doc = try self.node?.docOpen(id: docId) else {
        throw IrohError.Doc(description: "error opening doc")
      }

      return try doc.setBytes(author: author, key: key, bytes: bytes)
    }
  }
}

func mkdirP(path: String) {
    let fileManager = FileManager.default
    do {
        try fileManager.createDirectory(atPath: path,
                                        withIntermediateDirectories: true,
                                        attributes: nil)
    } catch {
        print("Error creating directory: \(error)")
    }
}



/// A representation of a mutable, synchronizable key-value store.
interface Doc {
  /// Get the document id of this doc.
  string id();
  /// Close the document.
  void close();
  /// Set the content of a key to a byte array.
  Hash set_bytes([ByRef] AuthorId author, bytes key, bytes value);
  /// Set an entries on the doc via its key, hash, and size.
  void set_hash(AuthorId author, bytes key, Hash hash, u64 size);
  /// Add an entry from an absolute file path
  void import_file(AuthorId author, bytes key, string path, boolean in_place, DocImportFileCallback? cb);
  /// Export an entry as a file to a given absolute path
  void export_file(Entry entry, string path, DocExportFileCallback? cb);
  /// Delete entries that match the given `author` and key `prefix`.
  ///
  /// This inserts an empty entry with the key set to `prefix`, effectively clearing all other
  /// entries whose key starts with or is equal to the given `prefix`.
  ///
  /// Returns the number of entries deleted.
  u64 del(AuthorId author_id, bytes prefix);
  /// Get the latest entry for a key and author.
  Entry? get_one(Query query);
  /// Get entries.
  ///
  /// Note: this allocates for each `Entry`, if you have many `Entry`s this may be a prohibitively large list.
  /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
  sequence<Entry> get_many(Query query);
  /// Get an entry for a key and author.
  ///
  /// Optionally also get the entry if it is empty (i.e. a deletion marker)
  Entry? get_exact(AuthorId author, bytes key, boolean include_empty);

  /// Share this document with peers over a ticket.
  string share(ShareMode mode, AddrInfoOptions addr_options);
  /// Start to sync this document with a list of peers.
  void start_sync(sequence<NodeAddr> peers);
  /// Stop the live sync for this document.
  void leave();
  /// Subscribe to events for this document.
  void subscribe(SubscribeCallback cb);
  /// Get status info for this document
  OpenState status();
  /// Set the download policy for this document
  void set_download_policy(DownloadPolicy policy);
  /// Get the download policy for this document
  DownloadPolicy get_download_policy();
};