# Changelog

Mudanças visíveis do medx-mcp. Formato baseado em
[Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).

## [Unreleased]

## [0.2.0] - 2026-10-08

### Adicionado

- `anexar_ao_prontuario` (escrita): anexa um arquivo do disco ao prontuário
  de um paciente, pelo caminho absoluto, até 22 MB. Uma resposta da MedX
  diferente de `Success` vira erro.

## [0.1.1] - 2026-10-03

Sem mudanças nesta crate.

## [0.1.0] - 2026-10-03

### Adicionado

- Servidor MCP sobre stdio para a MedX, como crate do workspace do medx-sdk.
- 45 ferramentas de leitura: usuários, agenda e relatórios, painel do dia,
  pacientes (com foto como imagem), prontuário, convênios e procedimentos,
  financeiro, configurações e chat interno.
- 14 ferramentas de escrita atrás de `MEDX_MCP_ALLOW_WRITE`: cadastro e
  atualização de paciente, agendamentos (criar, bloquear, mudar status,
  remarcar, confirmar pelo WhatsApp), notas, prontuário (registrar, editar,
  sumário) e chat.
- Login e re-login automáticos com `MEDX_LOGIN_CREDENTIAL` e
  `MEDX_PASSWORD_CREDENTIAL`, reaproveitando o `session.json` do medx-cli.
- `limite` em toda ferramenta de lista, com aviso quando há corte.

[Unreleased]: https://github.com/LLawli/medx-sdk-oss/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/LLawli/medx-sdk-oss/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/LLawli/medx-sdk-oss/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/LLawli/medx-sdk-oss/releases/tag/v0.1.0
