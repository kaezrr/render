run:
    RUST_LOG=info cargo run

bench:
    RUST_LOG=info mangohud cargo run --release

test:
    RUST_LOG=info cargo test -- --show-output
