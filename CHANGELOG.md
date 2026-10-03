# Changelog

Mudanças visíveis do medx-sdk (a crate `medx`) e do `medx-cli`. As do
servidor MCP ficam em [medx-mcp/CHANGELOG.md](medx-mcp/CHANGELOG.md).
Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).

## [Unreleased]

Ainda sem versão publicada: tudo o que existe até aqui.

### Adicionado

- Login como o do webapp (e-mail, `dbId`, senha cifrada com RSA-OAEP),
  derrubando a sessão aberta quando a conta já tem uma, e `session.json`
  reaproveitado entre execuções.
- `MedxClient` com token injetado e re-login automático em 401.
- Módulos de agenda, pacientes, prontuário, financeiro, painel do dia,
  chat interno, notificações, marketing, configurações, usuários e ajustes
  (lista em [README.md](README.md#módulos)).
- `medx-cli`, com os mesmos recursos pelo terminal, para Linux e Windows.
- Host da API configurável por `MEDX_BASE_URL`; o `session.json` guarda o
  host em que o token foi emitido e o re-login volta para ele.
- Diretório da sessão configurável por `MEDX_CONFIG_DIR`.
- Tipos de resposta serializáveis, com os nomes do SDK.
- Exemplo `agenda_do_dia`.

### Corrigido

- Todo texto que entra numa query string é codificado como o webapp faz.
- Calendário ICS pelo mesmo endpoint do webapp.
- O login não escreve mais no terminal; quem quiser o progresso usa
  `auth::login_at_with_progress`.
- Datas enviadas em horário local, sem fuso, como a MedX espera.
- O `medx-cli` liga o modo VT do console no Windows, que mostrava as cores
  como texto cru.

### Alterado

- TLS com rustls em vez de OpenSSL: o `medx-cli` não depende mais da
  `libssl` do sistema. Os certificados raiz passam a ser os do
  `webpki-roots`, embutidos no binário.

### Segurança

- Dependências com avisos do RustSec atualizadas (`h2`, `rustls`,
  `rustls-webpki`, `anyhow`, `rand`, `spin`), e `cargo audit` no CI.

[Unreleased]: https://github.com/LLawli/medx-sdk-oss/commits/master
