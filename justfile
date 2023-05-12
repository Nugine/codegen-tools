dev:
    cargo fmt
    cargo clippy
    cargo test

doc:
    cargo doc --open --no-deps

sync-version:
    cargo set-version   -p codegen-writer   0.1.0-dev
    cargo set-version   -p codegen-cfg      0.1.0-dev
    cargo set-version   -p codegen-libc     0.1.0-dev

publish:
    cargo publish       -p codegen-writer   
    cargo publish       -p codegen-cfg      
    cargo publish       -p codegen-libc     

codegen-libc *ARGS:
    #!/bin/bash -e
    cd {{ justfile_directory() }}
    ./scripts/download-libc.sh
    cargo run -p codegen-libc --features binary --release -- --libc temp/libc {{ ARGS }} | rustfmt
