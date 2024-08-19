import XCTest
@testable import IrohLib

final class IrohLibTests: XCTestCase {
    func testNodeId() async throws {
        let node = try await IrohLib.Iroh.memory()
        let nodeId = try await node.net().nodeId()
        print(nodeId)

        XCTAssertEqual(true, true)
    }

    func testProviderEvents() async throws {
        let blobEvents = BlobEventHandler()
        let options = NodeOptions.init(gcIntervalMillis: 0, blobEvents: blobEvents)
        let a = try await Iroh.memoryWithOptions(options: options)

        let blob = "oh hello".data(using: String.Encoding.utf8)!
        let result = try await a.blobs().addBytes(bytes: blob)
        let ticket = try await a.blobs().share(
                hash: result.hash,
                blobFormat: BlobFormat.raw,
                ticketOptions: AddrInfoOptions.addresses
            )

        let b = try await Iroh.memory()
        let progressManager = DownloadProgressManager()
        try await b.blobs().download(hash: ticket.hash(), opts: ticket.asDownloadOptions(), cb: progressManager)

        let completedProvides = await blobEvents.transfersCompleted
        XCTAssertEqual(completedProvides, 1)
        let completedFetches = await progressManager.completedFetches
        XCTAssertEqual(completedFetches, 1)
    }
}

actor BlobEventHandler: BlobProvideEventCallback {
    private(set) var transfersCompleted: UInt = 0

    func blobEvent(event: IrohLib.BlobProvideEvent) async throws {
        if event.type() == IrohLib.BlobProvideEventType.transferCompleted {
            transfersCompleted += 1
        }
    }
}

actor DownloadProgressManager: DownloadCallback {
    private(set) var completedFetches: UInt = 0

    func progress(progress: DownloadProgress) throws {
        if progress.type() == .allDone {
            completedFetches += 1
        }
    }
}
