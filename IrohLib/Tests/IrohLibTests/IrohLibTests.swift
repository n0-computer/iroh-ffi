import XCTest
@testable import IrohLib

final class IrohLibTests: XCTestCase {
    func testNodeId() async throws {
        let node = try await IrohLib.IrohNode.memory()
        let nodeId = try await node.nodeId()
        print(nodeId)

        XCTAssertEqual(true, true)
    }
}
