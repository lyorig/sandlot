set -e

if [[ $1 = "-h" || $1 = "--help" ]]; then
    cat << EOF
Usage: $(basename "$0") [-h]

Options:
    -h, --help  Display this message and exit.
EOF
    exit
fi

set -x

cargo doc --no-deps
cargo clippy --examples
