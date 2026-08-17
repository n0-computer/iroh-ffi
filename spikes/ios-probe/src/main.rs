//! SPIKE: iOS Simulator probe. Exercises the plugin -> core -> libstd dylib chain.
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let opts = iroh_ffi::EndpointOptions {
        preset: Some(iroh_ffi::preset_minimal()),
        ..Default::default()
    };
    let server = std::sync::Arc::new(iroh_ffi::Endpoint::bind(opts).await.expect("bind server"));
    let opts2 = iroh_ffi::EndpointOptions {
        preset: Some(iroh_ffi::preset_minimal()),
        ..Default::default()
    };
    let client = std::sync::Arc::new(iroh_ffi::Endpoint::bind(opts2).await.expect("bind client"));

    let ping = iroh_ffi_ping::Ping::new();
    ping.serve(server.clone()).await.expect("serve");
    println!("  server bound: {}", server.addr());
    println!("  client bound: {}", client.addr());

    let rtt = ping.ping(client.clone(), server.addr()).await.expect("ping");
    println!("  PING -> PONG round trip: {rtt} ms");
    println!("  OK: plugin -> core dylib chain works on iOS");
}
