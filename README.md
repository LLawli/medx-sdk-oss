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

## Testes de integração

Requer `.env` com `MEDX_LOGIN_CREDENTIAL` e `MEDX_PASSWORD_CREDENTIAL`:

```bash
cargo test --test paciente_real_tests -- --nocapture
```

## Python bindings

Ver [`medx-python`](https://github.com/LLawli/medx-python).
