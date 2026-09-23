set -e

if [[ $1 = "-h" || $1 = "--help" ]]; then
    cat << EOF
Usage: $(basename "$0") [-h]

Wrapper for \`cargo test\` with settings required for this crate.

Options:
    -h, --help  Display this message and exit.
EOF
    exit
fi

cargo test --tests -- --test-threads=1
