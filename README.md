# medx-sdk

SDK Rust para a API MedX, com CLI incluída.

> Projeto originalmente em Go, migrado para Rust.

## Módulos

- **Agenda** — agendamentos, parâmetros, setores, relatórios
- **Contatos** — criação, busca, atualização, exclusão de pacientes
- **Prontuário** — registros médicos, histórico, convênios, procedimentos
- **Finanças** — atendimentos, faturamento, pré-pagamento
- **Hoje** — notificações, notas, últimos atendidos
- **Marketing** — eventos, quests, diagnóstico QP
- **Notificações** — configurações de cliente, log de e-mail
- **Settings** — parâmetros gerais e de cor
- **Usuários** — usuário atual, listagem
- **Ajustes** — relatórios, pastas de documentos, ICS
- **Chat** — usuários, mensagens, envio

## Build

```bash
cargo build --release
```

## CLI

```bash
cargo run --bin medx -- --help
```

### Host da API

O webapp do MedX alterna sem aviso entre `v65.medx.med.br` e
`care-app65.medx.med.br`, e cada host tem sessão própria. O padrão é o `v65`;
para usar outro, defina `MEDX_BASE_URL` com a origem (com ou sem `/api` no fim):

```bash
MEDX_BASE_URL=https://care-app65.medx.med.br medx-cli auth login <email> <senha>
```

O `session.json` guarda o host em que o token foi emitido, e os comandos
seguintes reabrem a sessão nesse host. No SDK, use `MedxClient::login_at`,
`from_session_with_credentials_at` ou `auth::login_at`; o re-login em 401 vai
sempre para o host do próprio client.

### Onde fica a sessão

`session.json` fica em `~/.config/medx-sdk/` no Linux e em
`%APPDATA%\medx-sdk\` no Windows. `MEDX_CONFIG_DIR` troca esse diretório em
qualquer sistema; os testes usam essa variável para não tocar a sessão real
(o Windows ignora `XDG_CONFIG_HOME`).

## Testes de integração

Requer `.env` com `MEDX_LOGIN_CREDENTIAL` e `MEDX_PASSWORD_CREDENTIAL`:

```bash
cargo test --test paciente_real_tests -- --nocapture
```

## Python bindings

Ver [`medx-python`](https://github.com/LLawli/medx-python).
