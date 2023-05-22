//
//  Wrapper.swift
//  Iroh
//
//  Created by Brendan O'Brien on 5/21/23.
//

import Foundation

public func irohDescribeCollection(ticket: String) -> IrohCollection {
//    var col = iroh_describe_collection(ticket, "");
//    var blobs: [IrohBlob] = []
//    let len = col.blobs.len
//    for i in 0 ..< len {
//        blobs.append(IrohBlob())
//    }
//    return IrohCollection(items: [], totalBlobsSize: col.total_blobs_size)
    let items = [
        IrohBlob(name: "foo", cid: "bafyFoo"),
        IrohBlob(name: "bar", cid: "bafyBar"),
    ]
    return IrohCollection(items: items, totalBlobsSize: 22)
}

//public func irohGetTicket(ticket: String, outPath: String) {
//   iroh_get_ticket(ticket, outPath)
//}

public struct IrohCollection {
    var items: [IrohBlob] = []
    var totalBlobsSize: UInt64 = 0
}

public class IrohBlob {
    var name: String = ""
    var cid: String = ""
    init(name: String, cid: String) {
        self.name = name
        self.cid = cid
    }
}
