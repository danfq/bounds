default:
    just --list

# build release version
build:
    cargo build --release

# trigger the GitHub Actions release workflow
release:
    @command -v gh >/dev/null 2>&1 || { echo "error: GitHub CLI is required (https://cli.github.com/)" >&2; exit 1; }
    gh workflow run release.yml --ref main

# run tests
test:
    cargo nextest run

# run tests with coverage
coverage:
    cargo llvm-cov --html

# format and lint
check:
    cargo fmt --check
    cargo clippy -- -D warnings

# build and immediately run
run *args:
    cargo run -- {{ args }}
