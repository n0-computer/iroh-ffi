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
            url: "https://github.com/n0-computer/iroh-ffi/releases/download/v0.20.0/IrohLib.xcframework.zip",
            checksum: "8123c2d43690c423e9bc8993c935b2fe009731f3b65b95754358570077037858")
    ]
)
