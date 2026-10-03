# Gates de validação, os mesmos do CI.
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked --no-deps -- -D warnings
    cargo test --workspace --locked

# Vulnerabilidades conhecidas nas dependências (baixa a base do RustSec).
audit:
    cargo audit

test *args:
    cargo test --workspace --locked {{args}}
