set -e

if [[ $1 = "-h" || $1 = "--help" ]]; then
    cat << EOF
Usage: $(basename "$0") [-h]

Runs a series of lints and checks on the codebase to ensure Maximum Quality™.

Options:
    -h, --help  Display this message and exit.
EOF
    exit
fi

set -x

cargo clippy --examples
cargo doc --no-deps
