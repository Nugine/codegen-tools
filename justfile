dev:
    cargo fmt
    cargo clippy
    cargo test

doc:
    cargo doc --open --no-deps

sync-version:
    cargo set-version   -p codegen-writer   0.1.4-dev
    cargo set-version   -p bool-logic       0.2.0
    cargo set-version   -p codegen-cfg      0.2.0
    cargo set-version   -p codegen-libc     0.2.1

publish:
    # cargo publish       -p codegen-writer   
    # cargo publish       -p bool-logic
    # cargo publish       -p codegen-cfg      
    cargo publish       -p codegen-libc     

codegen-libc *ARGS:
    #!/bin/bash -e
    cd {{ justfile_directory() }}
    ./scripts/download-libc.sh
    cargo build -p codegen-libc --features binary --release
    ./target/release/codegen-libc --libc temp/libc {{ ARGS }} | rustfmt
