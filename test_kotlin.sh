set -eu

# $CLASSPATH must include `jna`

# copy cdylib to outdir
cp ./target/debug/libiroh.dylib ./kotlin/libuniffi_iroh.dylib

# Build jar file
kotlinc -Werror -d ./kotlin/iroh.jar ./kotlin/uniffi/**/*.kt -classpath $CLASSPATH

# Execute Tests
kotlinc -Werror -J-ea -classpath $CLASSPATH:./kotlin/iroh.jar:./kotlin -script ./kotlin/*.kts
