// SPIKE: Endpoint minted by IrohLib's native library, used by IrohPing's.
import Foundation
import IrohLib
import IrohPing

// A protocol handler from the PLUGIN module, registered on a CORE endpoint.
final class PingCreator: IrohLib.ProtocolCreator {
    let ping: IrohPing.Ping
    init(ping: IrohPing.Ping) { self.ping = ping }
    func create(endpoint: IrohLib.Endpoint) -> IrohLib.ProtocolHandler { ping.handler() }
}

let ping = IrohPing.Ping()
print("  alpn = \(String(data: Data(IrohPing.alpn()), encoding: .utf8) ?? "?")")

let server = try await IrohLib.Endpoint.bind(
    options: IrohLib.EndpointOptions(
        preset: IrohLib.presetMinimal(),
        protocols: [IrohPing.alpn(): PingCreator(ping: ping)]
    )
)
let serverAddr = server.addr()
print("  server bound: \(serverAddr)")

let client = try await IrohLib.Endpoint.bind(
    options: IrohLib.EndpointOptions(preset: IrohLib.presetMinimal())
)
print("  client bound: \(client.addr())")

let rtt = try await ping.ping(endpoint: client, addr: serverAddr)
print("  PING -> PONG round trip: \(rtt) ms")

try await client.close()
try await server.close()
print("  OK: Endpoint crossed the Swift module + library boundary")
