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
            ],
            path: "IrohLib/Sources/IrohLib"),
        .binaryTarget(
            name: "Iroh",
            url: "https://github.com/n0-computer/iroh-ffi/releases/download/v0.0.6/IrohLib.xcframework.zip",
            checksum: "be78667c39c36778c1f3f0624b9f4e8f47829f79fa0a69ca94c1192984b9188b")
    ]
)
