dev:
    cargo fmt
    cargo clippy
    cargo test

doc:
    cargo doc --open --no-deps

sync-version:
    cargo set-version -p codegen-writer  0.1.0-dev
    cargo set-version -p target-cfg     0.1.0-dev
