# Trabalhando no medx-sdk-oss

Workspace cargo com duas crates:

- **`medx-sdk`** (raiz, biblioteca `medx` e binário `medx-cli`): o SDK da
  MedX. Arquitetura em [docs/arquitetura.md](docs/arquitetura.md).
- **`medx-mcp`** ([medx-mcp/](medx-mcp/)): servidor MCP sobre o SDK. Regras
  próprias em [medx-mcp/AGENTS.md](medx-mcp/AGENTS.md).

## Fluxo

- **Meça antes de perguntar**, nunca contra a MedX real: protótipos, o
  servidor fake do medx-mcp, servidores HTTP locais. Quando a medição deixa
  uma opção claramente melhor, use-a e registre (em `docs/` do SDK ou em
  `medx-mcp/docs/decisoes.md`) com a evidência e o que reabriria a decisão.
  Quando não deixa, pergunte com prós, contras e números: escolhas que não
  são triviais são do usuário.
- **TDD com implementador delegado**, nas duas crates: interface com
  `todo!()`, testes, ver falhar pelo motivo certo, congelar, delegar o verde.
  O ciclo completo está em [medx-mcp/AGENTS.md](medx-mcp/AGENTS.md#fluxo-tdd-com-implementador-delegado).
- **Gates antes de todo commit:** `just check` (fmt, clippy `-D warnings` e
  testes do workspace). Quando mexer em dependência, também `just audit`.

## Regras

- **Nenhum dado real no repositório.** O repositório é público. Fixtures são
  sintéticas; testes ao vivo leem credenciais, paciente e arquivo de teste do
  ambiente ou do `.env` (ignorado pelo git). Antes de cada push, nada de
  e-mail, nome, id, telefone ou documento reais, nem em mensagem de commit.
- **Nada do frontend da MedX.** Engenharia reversa da API, sim; código do
  webapp da MedX, não, nem em trecho.
- **MedX real:** os testes ao vivo (`#[ignore]`) só rodam quando o usuário
  pede. Leitura ao vivo para validar é aceitável; escrita ao vivo, só com
  autorização explícita.
- Comentários, documentação e commits em português; identificadores em
  inglês.
- Toda mudança visível ganha uma linha em `[Unreleased]` no CHANGELOG da
  crate: [CHANGELOG.md](CHANGELOG.md) ou
  [medx-mcp/CHANGELOG.md](medx-mcp/CHANGELOG.md).
- Controle de versão com jj (colocado com git). Commits convencionais, um por
  mudança lógica, sem atribuição de IA em commit, PR ou issue. Push, PR e
  merge só quando o usuário liberar; merge com merge commit.
- O `target/` do workspace mora no disco de dados (`offload target` depois
  do primeiro build).
