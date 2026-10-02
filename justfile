# Gates de validação. O SDK ainda não passa no fmt nem no clippy estrito,
# então esses dois valem só para a crate do MCP; os testes valem para tudo.
check:
    cargo fmt -p medx-mcp --check
    cargo clippy -p medx-mcp --all-targets --locked --no-deps -- -D warnings
    cargo test --workspace --locked

test *args:
    cargo test --workspace --locked {{args}}
