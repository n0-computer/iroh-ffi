# Release Process

Each release process is broken down per langauge target:

## Swift

Swift packages have additional required edits _after_ the release is cut:

1. Cut the release on github.
2. run `make.sh`. confirm that all `cargo build` invocations are run with the `--release` flag
3. build a zip archive of `Iroh.xcframework`: `zip -r IrohLib.xcframework.zip Iroh.xcframework/*`
4. Compute the checksum for the zip archive: `swift package compute-checksum MyLibrary.xcframework.zip`
4. Upload the resulting zip archive as a release artifact. Copy the URL to the release.
5. edit the _root_ `Package.swift`, setting `targets[1](path:, checksum:)`:

```swift
// swift-tools-version:5.8
import PackageDescription

let package = Package(
    name: "IrohLib",
    platforms: [
        .iOS(.v15)
    ],
    products: [
        .library(
            name: "IrohLib",
            targets: ["IrohLib", "Iroh"]),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "IrohLib",
            dependencies: [
                .byName(name: "Iroh")
            ]),
        .binaryTarget(
            name: "Iroh",
            // SET THIS:
            path: "https://github.com/n0-computer/iroh-ffi/releases/download/v0.0.6/IrohLib.xcframework.zip"),
            // AND THIS:
            checksum: "4e612297d935332562ed8038ab6a66bde32dd644daf5f4d4f64e24f3bdf961e8",
    ]
)
```

6. Commit the result & push