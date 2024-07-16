# Release Process

Each release process is broken down per language target:

## Swift

Swift packages have additional required edits _after_ the release is cut:

1. Cut the release on github.
2. run `make_swift.sh`. confirm that all `cargo build` invocations are run with the `--release` flag
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

## Python

The first time:

1) Create an account on [pypi](https://pypi.org/) & [testpipy](https://test.pypi.org/project/iroh/)
2) Get invited to the `iroh` project
3) Install `twine`
4) Upgrade `pkginfo` to at least `1.10`. For more information check out [this issue on twine](https://github.com/pypa/twine/issues/1070)
5) Create an API token on pipy and test pipy
6) Put those tokens into ~/.pypirc:
```
# ~/.pypirc
[pypi]
username = __token__
password = pypi-TOKEN

[testpypi]
username = __token__
password = pypi-TOKEN
```

To release iroh python:

1) Download the artifacts from the [wheels ci workflow](https://github.com/n0-computer/iroh-ffi/actions/workflows/wheels.yml), picking the workflow that was run on the latest `main` branch
2) Extract the artifacts
3) Upload each to testpypi: `twine upload --repository testpypi iroh-$VERSION-*.whl`
4) Dogfood by downloading the latest iroh version from testpipy and using it. The simplest test may be to run the python code in the `iroh-ffi/python` directory.
    - create & activate a new virtual env
    - install iroh from testpypi `pip install -i https://test.pypi.org/simple/ iroh`
    - run `python main.py`
    - ensure it works (and remove the test env)
5) Upload each to pypi: `twine upload iroh-$VERSION-*.whl`
