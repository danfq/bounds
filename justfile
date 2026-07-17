default:
    just --list

# build release version
build:
    cargo build --release

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

# bump version and tag a release
release version:
    @echo "Releasing version {{ version }}"
    cargo set-version {{ version }}
    git commit -am "chore: release v{{ version }}"
    git tag v{{ version }}
    git push --follow-tags
