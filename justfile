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
