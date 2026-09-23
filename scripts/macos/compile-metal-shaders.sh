set -e

# These flags are set by default so that they can be output as part of the help text.
# They're disabled later on if the debug flag is not present.
DEBUGFLAGS=(-frecord-sources -gline-tables-only)

if [[ $1 = "-h" || $1 = "--help" ]]; then
    cat << EOF
Usage: $(basename "$0") [-hg]

Compiles Metal Shading Language sources for use with GPU examples on Apple platforms.
After running this script, \`cargo build --example\` should succeed on macOS.

Options:
    -h, --help  Display this message and exit.
    -g          Add debug flags for Xcode (${DEBUGFLAGS[@]}).
EOF
    exit
fi

# Check that `xcrun` is available on PATH. We don't check for the Metal Toolchain,
# since the utility prints readable errors in case it's missing.
if ! command -v xcrun >/dev/null 2>&1; then
    echo "\`xcrun\` not found. Make sure that you have the Xcode Command Line Tools installed (\`xcode-select --install\`)."
    exit 1
fi

if [[ "$1" != "-g" ]]; then
    DEBUGFLAGS=()
fi


# `examples/gpu.rs`
xcrun -sdk macosx metal -o /tmp/triangle.ir -c examples/shaders/src/triangle.metal "${DEBUGFLAGS[@]}"
xcrun -sdk macosx metal -o examples/shaders/triangle.metallib /tmp/triangle.ir "${DEBUGFLAGS[@]}"
rm /tmp/triangle.ir

# `examples/teapot.rs`
xcrun -sdk macosx metal -o /tmp/teapot.ir -c examples/shaders/src/teapot.metal "${DEBUGFLAGS[@]}"
xcrun -sdk macosx metal -o examples/shaders/teapot.metallib /tmp/teapot.ir "${DEBUGFLAGS[@]}"
rm /tmp/teapot.ir

# `examples/compute.rs`
xcrun -sdk macosx metal -o /tmp/compute.ir -c examples/shaders/src/compute.metal "${DEBUGFLAGS[@]}"
xcrun -sdk macosx metal -o examples/shaders/compute.metallib /tmp/compute.ir "${DEBUGFLAGS[@]}"
rm /tmp/compute.ir
