import XCTest
@testable import IrohLib

private let ALPN = Data("iroh-ffi/test/0".utf8)

final class KeyTests: XCTestCase {
    func testEndpointId() throws {
        let keyStr = "523c7996bad77424e96786cf7a7205115337a5b4565cd25506a0f297b191a5ea"
        let fmtStr = "523c7996ba"
        let bytes = Data([
            0x52, 0x3c, 0x79, 0x96, 0xba, 0xd7, 0x74, 0x24,
            0xe9, 0x67, 0x86, 0xcf, 0x7a, 0x72, 0x05, 0x11,
            0x53, 0x37, 0xa5, 0xb4, 0x56, 0x5c, 0xd2, 0x55,
            0x06, 0xa0, 0xf2, 0x97, 0xb1, 0x91, 0xa5, 0xea,
        ])

        let id = try EndpointId.fromString(s: keyStr)
        XCTAssertEqual(id.description, keyStr)
        XCTAssertEqual(id.toBytes(), bytes)
        XCTAssertEqual(id.fmtShort(), fmtStr)

        let id2 = try EndpointId.fromBytes(bytes: bytes)
        XCTAssertEqual(id, id2)
    }

    func testEndpointIdRejectsBadBytes() {
        XCTAssertThrowsError(try EndpointId.fromBytes(bytes: Data([1, 2, 3]))) { error in
            guard let err = error as? IrohError else {
                return XCTFail("expected IrohError, got \(error)")
            }
            XCTAssertEqual(err.kind(), .invalidInput)
            XCTAssertTrue(err.isKind(kind: .invalidInput))
            XCTAssertTrue(err.message().contains("32 bytes"))
            XCTAssertEqual(err.debugMessage(), err.message())
        }
    }

    func testEndpointIdParseErrorKind() {
        XCTAssertThrowsError(try EndpointId.fromString(s: "not-an-endpoint-id")) { error in
            guard let err = error as? IrohError else {
                return XCTFail("expected IrohError, got \(error)")
            }
            XCTAssertEqual(err.kind(), .keyParsing)
            XCTAssertTrue(err.isKind(kind: .keyParsing))
            XCTAssertFalse(err.message().isEmpty)
            XCTAssertFalse(err.debugMessage().isEmpty)
        }
    }

    func testSecretKeyRoundtrip() throws {
        let secret = SecretKey.generate()
        let raw = secret.toBytes()
        XCTAssertEqual(raw.count, 32)
        let secret2 = try SecretKey.fromBytes(bytes: raw)
        XCTAssertEqual(secret.toBytes(), secret2.toBytes())
        XCTAssertEqual(secret.`public`().toBytes(), secret2.`public`().toBytes())
    }

    func testSignVerifyRoundtrip() throws {
        let secret = SecretKey.generate()
        let pub = secret.`public`()
        let msg = Data("hello iroh".utf8)
        let sig = secret.sign(message: msg)

        let raw = sig.toBytes()
        XCTAssertEqual(raw.count, 64)
        let sig2 = try Signature.fromBytes(bytes: raw)
        XCTAssertEqual(sig2.toBytes(), raw)

        try pub.verify(message: msg, signature: sig)
        try pub.verify(message: msg, signature: sig2)
    }

    func testVerifyRejectsTampered() {
        let secret = SecretKey.generate()
        let pub = secret.`public`()
        let sig = secret.sign(message: Data("original".utf8))
        XCTAssertThrowsError(try pub.verify(message: Data("tampered".utf8), signature: sig))
    }
}

final class RelayTests: XCTestCase {
    func testRelayMapCrud() throws {
        let m = RelayMap.empty()
        XCTAssertTrue(m.isEmpty())

        let cfg = RelayConfig(
            url: "https://relay.example.org/",
            quicPort: 7842,
            authToken: "hunter2"
        )
        try m.insert(config: cfg)
        XCTAssertEqual(m.len(), 1)
        XCTAssertTrue(try m.contains(url: "https://relay.example.org/"))

        let got = try m.get(url: "https://relay.example.org/")
        XCTAssertEqual(got?.url, "https://relay.example.org/")
        XCTAssertEqual(got?.quicPort, 7842)
        XCTAssertEqual(got?.authToken, "hunter2")

        XCTAssertTrue(try m.remove(url: "https://relay.example.org/"))
        XCTAssertTrue(m.isEmpty())
    }

    func testRelayModeConstructors() throws {
        _ = RelayMode.disabled()
        _ = RelayMode.defaultMode()
        _ = RelayMode.staging()
        let m = try RelayMap.fromUrls(urls: ["https://r1.example.org/"])
        let custom = RelayMode.custom(map: m)
        XCTAssertEqual(custom.relayMap().len(), 1)
        _ = try RelayMode.customFromUrls(urls: ["https://r2.example.org/"])
    }
}

/// A user-implemented Preset: minimal baseline + a custom ALPN.
final class CustomPreset: Preset {
    func apply(builder: EndpointBuilder) {
        builder.applyMinimal()
        builder.alpns(alpns: [Data("custom/preset/1".utf8)])
    }
}

final class EndpointTests: XCTestCase {
    func testCustomPreset() async throws {
        let ep = try await Endpoint.bind(options: EndpointOptions(preset: CustomPreset()))
        XCTAssertFalse(ep.boundSockets().isEmpty)
        try await ep.close()
    }

    func testBuilderBind() async throws {
        let builder = EndpointBuilder()
        builder.applyMinimal()
        let ep = try await builder.bind()
        XCTAssertFalse(ep.boundSockets().isEmpty)
        try await ep.close()
    }

    func testBuilderBindConsumes() async throws {
        let builder = EndpointBuilder()
        builder.applyMinimal()
        let ep = try await builder.bind()
        try await ep.close()
        do {
            _ = try await builder.bind()
            XCTFail("expected error on second bind()")
        } catch {
            XCTAssertTrue("\(error)".contains("already consumed"), "got: \(error)")
        }
    }

    func testBindLifecycle() async throws {
        let ep = try await Endpoint.bind(options: EndpointOptions(preset: presetMinimal()))
        let id = ep.id()
        XCTAssertFalse(id.description.isEmpty)
        XCTAssertEqual(ep.addr().id(), id)
        XCTAssertFalse(ep.boundSockets().isEmpty)
        XCTAssertEqual(ep.secretKey().`public`().toBytes(), id.toBytes())
        try await ep.close()
        XCTAssertTrue(ep.isClosed())
    }

    func testEndpointTicketRoundtrip() async throws {
        let ep = try await Endpoint.bind(options: EndpointOptions(preset: presetMinimal()))
        let addr = ep.addr()
        let ticket = try EndpointTicket.fromAddr(addr: addr)
        let s = ticket.description
        XCTAssertTrue(s.hasPrefix("endpoint"))
        let parsed = try EndpointTicket.fromString(str: s)
        XCTAssertEqual(parsed.endpointAddr().id(), addr.id())
        try await ep.close()
    }

    func testEndpointTicketRejectsGarbage() throws {
        XCTAssertThrowsError(try EndpointTicket.fromString(str: "not-a-ticket"))
    }

    func testConnectEchoRoundtrip() async throws {
        let server = try await Endpoint.bind(
            options: EndpointOptions(
                preset: presetN0(),
                alpns: [ALPN],
                relayMode: RelayMode.disabled()
            )
        )
        let serverAddr = server.addr()
        let serverId = server.id()

        let serverTask = Task {
            let incoming = try await server.acceptNext()!
            let conn = try await incoming.accept().connect()
            XCTAssertEqual(conn.alpn(), ALPN)
            let bi = try await conn.acceptBi()
            let msg = try await bi.recv().readToEnd(sizeLimit: 64)
            try await bi.send().writeAll(buf: msg)
            try await bi.send().finish()
            let dg = try await conn.readDatagram()
            try conn.sendDatagram(data: dg)
            _ = await conn.closed()
        }

        let client = try await Endpoint.bind(
            options: EndpointOptions(preset: presetN0(), relayMode: RelayMode.disabled())
        )
        let conn = try await client.connect(addr: serverAddr, alpn: ALPN)
        XCTAssertEqual(conn.remoteId(), serverId)
        XCTAssertFalse(conn.paths().isEmpty)

        let bi = try await conn.openBi()
        try await bi.send().writeAll(buf: Data("hello iroh".utf8))
        try await bi.send().finish()
        let echoed = try await bi.recv().readToEnd(sizeLimit: 64)
        XCTAssertEqual(String(decoding: echoed, as: UTF8.self), "hello iroh")

        try conn.sendDatagram(data: Data("ping".utf8))
        let pong = try await conn.readDatagram()
        XCTAssertEqual(String(decoding: pong, as: UTF8.self), "ping")

        let stats = conn.stats()
        XCTAssertGreaterThan(stats.udpTxDatagrams, 0)

        try conn.close(errorCode: 0, reason: Data("bye".utf8))
        _ = try await serverTask.value
        try await client.close()
        try await server.close()
    }

    func testUniStream() async throws {
        let server = try await Endpoint.bind(
            options: EndpointOptions(
                preset: presetN0(),
                alpns: [ALPN],
                relayMode: RelayMode.disabled()
            )
        )
        let serverAddr = server.addr()

        let serverTask = Task {
            let incoming = try await server.acceptNext()!
            let conn = try await incoming.accept().connect()
            let recv = try await conn.acceptUni()
            let msg = try await recv.readToEnd(sizeLimit: 32)
            XCTAssertEqual(String(decoding: msg, as: UTF8.self), "unidirectional")
        }

        let client = try await Endpoint.bind(
            options: EndpointOptions(preset: presetN0(), relayMode: RelayMode.disabled())
        )
        let conn = try await client.connect(addr: serverAddr, alpn: ALPN)
        let send = try await conn.openUni()
        try await send.writeAll(buf: Data("unidirectional".utf8))
        try await send.finish()

        _ = try await serverTask.value
        try await client.close()
        try await server.close()
    }
}

// Well-formed (but fake) API secret — the remote does not exist, but the
// client connects lazily so construction still succeeds.
private let FAKE_API_SECRET =
    "servicesaaqaobyha4dqobyha4dqobyha4dqobyha4dqobyha4dqobyha4dqob"
    + "75c4sdqwvay5nwj63yzvqc7iozsh66x53lcpcy5vyc5ledl2pwdaaa"

final class ServicesTests: XCTestCase {
    private func endpoint() async throws -> Endpoint {
        try await Endpoint.bind(options: EndpointOptions(preset: presetMinimal()))
    }

    func testBootsWithFakeSecret() async throws {
        let ep = try await endpoint()
        _ = try await ServicesClient.create(
            endpoint: ep,
            options: ServicesOptions(apiSecret: FAKE_API_SECRET)
        )
        try await ep.close()
    }

    func testRejectsNoCredentials() async throws {
        let ep = try await endpoint()
        do {
            _ = try await ServicesClient.create(endpoint: ep, options: ServicesOptions())
            XCTFail("expected rejection")
        } catch {}
        try await ep.close()
    }

    func testRejectsTwoCredentials() async throws {
        let ep = try await endpoint()
        do {
            _ = try await ServicesClient.create(
                endpoint: ep,
                options: ServicesOptions(apiSecret: FAKE_API_SECRET, apiSecretFromEnv: true)
            )
            XCTFail("expected rejection")
        } catch {}
        try await ep.close()
    }

    func testRejectsMalformedSecret() async throws {
        let ep = try await endpoint()
        do {
            _ = try await ServicesClient.create(
                endpoint: ep,
                options: ServicesOptions(apiSecret: "not-a-valid-ticket")
            )
            XCTFail("expected rejection")
        } catch {}
        try await ep.close()
    }
}
