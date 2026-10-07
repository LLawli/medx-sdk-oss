# Changelog

Mudanças visíveis do medx-sdk (a crate `medx`) e do `medx-cli`. As do
servidor MCP ficam em [medx-mcp/CHANGELOG.md](medx-mcp/CHANGELOG.md).
Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).

## [Unreleased]

### Adicionado

- `ArquivoDto::filetype_for`: o tipo MIME de um anexo pela extensão do
  nome, com mais formatos (webp, heic, bmp, tiff, csv, rtf, odt, planilhas
  e mp4).

### Corrigido

- `medx-cli prontuario upload` não trata mais um nome sem ponto (como
  `pdf`) como se fosse a extensão.

## [0.1.1] - 2026-10-03

### Adicionado

- Pacote `medx` no Chocolatey, com o `medx-cli` e o `medx-mcp` para
  Windows: `choco install medx`.

## [0.1.0] - 2026-10-03

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

### Removido

- A release móvel `latest`, refeita a cada push no `master`: os binários
  saem só nos releases com tag, para Linux, macOS e Windows, com o
  `medx-mcp` junto e SHA256.

### Segurança

- Dependências com avisos do RustSec atualizadas (`h2`, `rustls`,
  `rustls-webpki`, `anyhow`, `rand`, `spin`), e `cargo audit` no CI.

[Unreleased]: https://github.com/LLawli/medx-sdk-oss/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/LLawli/medx-sdk-oss/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/LLawli/medx-sdk-oss/releases/tag/v0.1.0
