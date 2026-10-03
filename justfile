# Gates de validação, os mesmos do CI.
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked --no-deps -- -D warnings
    cargo test --workspace --locked

test *args:
    cargo test --workspace --locked {{args}}
