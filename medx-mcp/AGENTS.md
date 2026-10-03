# Trabalhando no medx-mcp

Servidor MCP (Rust, rmcp, stdio) que expõe a plataforma MedX pelo medx-sdk,
a crate da raiz deste repositório. O design e as medições que o sustentam
estão em [docs/decisoes.md](docs/decisoes.md).

## Decisões

- Meça antes de perguntar. Para qualquer escolha que não seja trivial
  (stack, design do servidor, formato de saída, mudança no SDK, credenciais,
  publicação), construa e meça as alternativas primeiro: protótipos
  descartáveis, benchmarks locais, o servidor fake da MedX. **Nunca contra a
  MedX real.**
- Se a medição deixar uma alternativa claramente melhor, use-a sem perguntar
  e registre em docs/decisoes.md.
- Se não deixar, pergunte ao usuário com os prós e contras de cada opção e os
  números. Escolhas que não são triviais são do usuário. As triviais (nomes
  internos, organização de módulos, fixtures, texto de commit) não pedem
  pergunta.
- Cada decisão em docs/decisoes.md tem a evidência e o que a reabriria. Leia
  antes de mudar comportamento; atualize quando a decisão mudar.

## Fluxo: TDD com implementador delegado

Toda fatia de trabalho passa por este ciclo. O agente principal nunca escreve
o código que faz os próprios testes passarem.

1. **Interface primeiro.** Tipos, structs de parâmetros e assinaturas com
   `todo!()`, para os testes compilarem. Teste que não compila não é um
   vermelho útil: o vermelho tem de ser uma falha em tempo de execução, pelo
   motivo esperado.
2. **Testes.** Unitários para lógica pura, e de integração que sobem o
   servidor em processo atrás de um cliente rmcp num canal duplex, com o SDK
   real falando com o servidor fake da MedX (`tests/common/fake_medx.rs`,
   fixtures sintéticas, nunca dado de paciente). O que grava o `session.json`
   (login) só roda pelo binário, com `MEDX_CONFIG_DIR` temporário.
3. **Ver falhar** pelo motivo certo (o `todo!()` ou a asserção, não um erro de
   fixture). Anote os testes vermelhos.
4. **Congelar os testes.** Daqui até o verde, ninguém mexe nos arquivos de
   teste, nem o agente principal.
5. **Delegar o verde** ao subagente `implementador-tdd`
   (`.claude/agents/implementador-tdd.md`, fora do controle de versão): a
   fatia, os arquivos de implementação que ele pode tocar, os testes
   congelados, os vermelhos esperados e as dependências permitidas.
6. **Revisar.** Os testes congelados não mudaram, não há atalho que só engana
   o teste, e `just check` passa com as contagens esperadas em `test result`.
7. **Commitar** com jj: um commit convencional por fatia.

Se um teste estiver errado, o subagente para e relata; o agente principal
decide. Mudar um teste é uma rodada nova: corrigir, ver falhar, congelar,
delegar.

## Regras

- Gates: `just check` na raiz (fmt, clippy `-D warnings` e testes do
  workspace inteiro). Rode antes de todo commit. As regras gerais do
  repositório estão no [AGENTS.md da raiz](../AGENTS.md).
- Comentários, documentação e commits em português, como o resto do
  repositório; identificadores em inglês, como no SDK. Nomes e descrições das
  ferramentas e dos parâmetros (o que o modelo lê) em português do Brasil.
- Erros com `thiserror`; sem `unwrap` nem `expect` fora de testes.
- O stdout leva só JSON-RPC. Logs vão para o stderr via `tracing`.
- Toda mudança visível ao usuário ganha uma linha em `[Unreleased]` no
  CHANGELOG.md.
- Controle de versão com jj (colocado com git). Commits convencionais, sem
  atribuição de IA de nenhum tipo. Push só quando o usuário liberar.
- O `target/` do workspace mora no disco de dados (`offload target` depois do
  primeiro build).
