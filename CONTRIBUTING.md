# Contribuindo

Issues e pull requests são bem-vindos. Este guia cobre o que é preciso para
um PR ser aceito sem idas e vindas.

## Preparar o ambiente

- Rust 1.88 ou mais novo (`rustup`), com `rustfmt` e `clippy`.
- [`just`](https://github.com/casey/just) para os gates.
- [`cargo-audit`](https://github.com/rustsec/rustsec/tree/main/cargo-audit),
  se for mexer em dependência.

```bash
git clone https://github.com/LLawli/medx-sdk-oss.git
cd medx-sdk-oss
just check
```

`just check` roda `cargo fmt --check`, `cargo clippy -D warnings` e os
testes do workspace, os mesmos do CI. Os testes que rodam por padrão não
precisam de conta na MedX.

## Antes de abrir o PR

- **`just check` passa.** Se mexeu em dependência, `just audit` também.
- **Teste para o que mudou.** Fixtures sintéticas e servidor HTTP local
  (`127.0.0.1`), como em `tests/login_tests.rs` ou no servidor fake do
  medx-mcp. Teste contra a MedX real fica `#[ignore]`.
- **Nenhum dado real.** Nada de e-mail, nome, id, telefone, documento ou
  resposta crua da MedX de uma clínica de verdade, nem no código, nem nas
  fixtures, nem na descrição do PR ou da issue. Para reproduzir um bug,
  troque os valores por sintéticos.
- **CHANGELOG.** Mudança visível ganha uma linha em `[Unreleased]` no
  [CHANGELOG.md](CHANGELOG.md) (SDK e CLI) ou no
  [medx-mcp/CHANGELOG.md](medx-mcp/CHANGELOG.md).
- **Commits** em [conventional commits](https://www.conventionalcommits.org/pt-br/),
  em português, um por mudança lógica (`fix(agenda): ...`,
  `feat(mcp): ...`). Identificadores no código continuam em inglês.

## Testes ao vivo

Os testes `#[ignore]` falam com a MedX real, com a sua conta, e alguns
**criam e apagam** pacientes, agendamentos e notas nela. Como a MedX aceita
uma sessão por conta, eles derrubam a sessão do navegador. As variáveis que
eles leem estão no [README](README.md#desenvolvimento).

## Onde ler antes de mudar

- [docs/arquitetura.md](docs/arquitetura.md): login, cliente, sessão e as
  manias da API da MedX que o código respeita.
- [medx-mcp/docs/decisoes.md](medx-mcp/docs/decisoes.md): o design do
  servidor MCP e o que reabriria cada decisão.

## Licença

Ao contribuir, você concorda que sua contribuição seja distribuída sob a
mesma licença do projeto, [AGPL-3.0-or-later](LICENSE).
