//
//  Iroh.swift
//  iroh
//
//  Created by Brendan O'Brien on 5/15/23.
//
//  This is a Swift wrapper for the external C wrapper for the Rust static library.
//  Making this a separate class allows for better error handling and isolation,
//  as well as the ability to do automated testing.
//

import Foundation

public enum IrohError: Error {
    case unexpected(UInt32)
}

public class Iroh {
    public static func get(cid: String, peer: String, peerAddr:String, outPath:String) throws {
        let status = iroh_get(cid, peer, peerAddr, outPath)
        guard status == errSecSuccess else {
            throw IrohError.unexpected(status)
        }
    }

    public static func getTicket(ticket: String, outPath: String) throws {
        let status = iroh_get_ticket(ticket, outPath)
        guard status == errSecSuccess else {
            throw IrohError.unexpected(status)
        }
    }
}

public func apples() {
    print("i am apples")
}
