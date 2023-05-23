
REPO_ROOT=".."
RUST_FFI_DIR="../iroh/iroh-ffi"
OUT_DIR="../build"
mkdir -p $OUT_DIR

echo "Generate Iroh C header, copy Module map"
mkdir -p "${OUT_DIR}/include"
cargo +`cat ${REPO_ROOT}/rust-toolchain` test --features c-headers --locked --manifest-path "${RUST_FFI_DIR}/Cargo.toml" -- generate_headers
cp ${RUST_FFI_DIR}/libiroh.h ${OUT_DIR}/include/iroh.h
cp ${REPO_ROOT}/include/module.modulemap ${OUT_DIR}/include/module.modulemap

echo "Build Iroh Libraries for Apple Platforms"
targets=(
  "aarch64-apple-ios"
  "x86_64-apple-ios"
  "aarch64-apple-ios-sim"
)

for target in "${targets[@]}"; do
  cargo +`cat ${REPO_ROOT}/rust-toolchain` build --package iroh_ffi --release --locked --target "${target}" --manifest-path "${RUST_FFI_DIR}/Cargo.toml"
  mkdir -p "${OUT_DIR}/lib_${target}"
  cp "${RUST_FFI_DIR}/target/${target}/release/libiroh.a" "${OUT_DIR}/lib_${target}/libiroh.a"
done

echo "Run Lipo"
mkdir -p "${OUT_DIR}/lib_ios-simulator-universal"
lipo -create \
  "${OUT_DIR}/lib_x86_64-apple-ios/libiroh.a" \
  "${OUT_DIR}/lib_aarch64-apple-ios-sim/libiroh.a" \
  -output "${OUT_DIR}/lib_ios-simulator-universal/libiroh.a"
          

echo "Create XCFramework"

xcodebuild -create-xcframework \
  -library ${OUT_DIR}/lib_ios-simulator-universal/libiroh.a \
  -headers ${OUT_DIR}/include/ \
  -library ${OUT_DIR}/lib_aarch64-apple-ios/libiroh.a \
  -headers ${OUT_DIR}/include/ \
  -output ${REPO_ROOT}/LibIroh.xcframework

zip -r ${REPO_ROOT}/libiroh-xcframework.zip ${REPO_ROOT}/LibIroh.xcframework

echo "Done"