// swift-tools-version:5.8
import PackageDescription

let package = Package(
    name: "IrohLib",
    platforms: [
        .iOS(.v15),
        .macOS(.v12)
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
            linkerSettings: [
              .linkedFramework("SystemConfiguration")
            ]),
        .binaryTarget(
            name: "Iroh",
            path: "artifacts/Iroh.xcframework"),
        .testTarget(
          name: "IrohLibTests",
          dependencies: ["IrohLib"]),
    ]
)
