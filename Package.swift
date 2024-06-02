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
            url: "https://github.com/n0-computer/iroh-ffi/releases/download/0.17.0/IrohLib.xcframework.zip",
            checksum: "2909fa5d0f2db938c946defb64d602120c76ee5e9c88940b191db4548f15b365")
    ]
)
