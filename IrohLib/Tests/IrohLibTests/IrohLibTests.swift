import XCTest
@testable import IrohLib

final class IrohLibTests: XCTestCase {
    func testNodeId() async throws {
        let node = try await IrohLib.Iroh.memory()
        let nodeId = try await node.node().nodeId()
        print(nodeId)

        XCTAssertEqual(true, true)
    }
}
