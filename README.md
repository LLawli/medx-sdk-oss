# medx-sdk

A MedX, sistema de gestão de clínicas médicas, só se usa pelo navegador: não
tem API pública nem documentação. Este repositório dá acesso programático a
ela, falando com a mesma API que o webapp usa.

- **SDK em Rust** (crate `medx`): login, sessão e cerca de 80 operações de
  agenda, pacientes, prontuário, financeiro, notas, chat interno e
  configurações.
- **CLI** (`medx-cli`): as mesmas operações pelo terminal.
- **Servidor MCP** ([`medx-mcp`](medx-mcp/README.md)): dá a um agente (Claude
  Code, Codex e outros) acesso à MedX, com as escritas desligadas por padrão.

Casos de uso: consultar a agenda do dia sem abrir o navegador, automatizar
cadastros e confirmações, montar relatórios próprios, deixar um assistente
responder "quem eu atendo amanhã?".

## Instalação

### CLI

Binários para Linux e Windows saem a cada mudança no `master`, na release
[`latest`](https://github.com/LLawli/medx-sdk-oss/releases/tag/latest):

```bash
gh release download latest --repo LLawli/medx-sdk-oss --pattern medx-cli \
  --output ~/.local/bin/medx-cli && chmod +x ~/.local/bin/medx-cli
```

Ou compilando (Rust 1.88 ou mais novo):

```bash
cargo install --locked --git https://github.com/LLawli/medx-sdk-oss medx-sdk --bin medx-cli
```

### SDK

```toml
[dependencies]
medx-sdk = { git = "https://github.com/LLawli/medx-sdk-oss" }
```

A crate se chama `medx-sdk`, e a biblioteca é importada como `medx`.

### Servidor MCP

Ver [medx-mcp/README.md](medx-mcp/README.md).

## Uso

### CLI

```bash
medx-cli auth login medico@clinica.com.br 'senha'
medx-cli agenda daily 3 2026-03-17     # agenda do profissional 3 no dia
medx-cli help agenda                   # comandos de um recurso
```

### SDK

O cliente é síncrono (`reqwest::blocking`). Com credenciais, ele renova o
token sozinho quando a MedX responde 401.

```rust
use medx::{MedxClient, MedxError};

fn main() -> Result<(), MedxError> {
    let client = MedxClient::login("medico@clinica.com.br", "senha")?;
    let eu = client.current_user()?;
    let hoje = chrono::Local::now().format("%Y-%m-%d").to_string();

    for consulta in client.daily_agenda(eu.user_id, &hoje)? {
        println!("{}  {}", consulta.start, consulta.description);
    }
    Ok(())
}
```

O mesmo programa, lendo as credenciais do ambiente, está em
[`examples/agenda_do_dia.rs`](examples/agenda_do_dia.rs)
(`cargo run --example agenda_do_dia`).

### Módulos

| Módulo | O que cobre |
|---|---|
| `auth`, `client`, `session` | Login, cliente autenticado, `session.json` |
| `users`, `settings` | Usuário atual, usuários da clínica, parâmetros gerais e de cor |
| `contacts` | Pacientes: busca, ficha, foto, convênios, cadastro e atualização |
| `agenda` | Agendamentos, bloqueios, status, setores, relatórios |
| `prontuario` | Registros, histórico, formulários, arquivos, convênios e procedimentos |
| `financas` | Atendimentos, faturamento, pré-pagamento |
| `hoje` | Painel do dia: notificações, notas, últimos atendidos |
| `chat` | Chat interno entre profissionais |
| `notificacoes`, `marketing` | Configurações de envio, log de e-mail, eventos, questionários |
| `ajustes` | Relatórios disponíveis, pastas de documentos, calendário ICS |

## Coisas que é bom saber

- **Uma sessão por conta.** A MedX aceita uma sessão aberta por conta. Fazer
  login pelo SDK, pela CLI ou pelo MCP derruba a sessão do navegador com a
  mesma conta, e vice-versa.
- **Os dados são de pacientes reais.** Cuidado com logs e com o que sai do
  computador.
- **Host da API.** O webapp alterna sem aviso entre `v65.medx.med.br` e
  `care-app65.medx.med.br`, e cada host tem sessão própria. O padrão é o
  `v65`; `MEDX_BASE_URL` (com ou sem `/api`) troca o host. O `session.json`
  guarda o host em que o token foi emitido, e o re-login volta para ele. No
  SDK: `MedxClient::login_at`, `from_session_with_credentials_at`,
  `auth::login_at`.
- **Onde fica a sessão.** `session.json` fica em `~/.config/medx-sdk/` no
  Linux e em `%APPDATA%\medx-sdk\` no Windows. `MEDX_CONFIG_DIR` troca esse
  diretório em qualquer sistema (o Windows ignora `XDG_CONFIG_HOME`).

## Desenvolvimento

```bash
just check   # fmt, clippy -D warnings e testes do workspace
just audit   # vulnerabilidades conhecidas nas dependências
```

Os testes que rodam por padrão não falam com a MedX: usam fixtures sintéticas
e servidores HTTP locais. Os testes ao vivo são `#[ignore]` e rodam com a
própria conta; alguns **criam e apagam** pacientes, agendamentos e notas
nela. Eles leem do ambiente, ou de um `.env` na raiz:

| Variável | Para quê |
|---|---|
| `MEDX_LOGIN_CREDENTIAL`, `MEDX_PASSWORD_CREDENTIAL` | Login |
| `MEDX_TEST_PATIENT_ID`, `MEDX_TEST_PATIENT_NAME` | Paciente usado pelos testes que leem e restauram dados de um paciente |
| `MEDX_TEST_FILE_CLASSE` | Campo `classe` de um registro com arquivo, para os testes de arquivo do prontuário |

```bash
cargo test --test paciente_real_tests -- --ignored --nocapture
```

Arquitetura e decisões do SDK em [docs/arquitetura.md](docs/arquitetura.md);
o que falta cobrir da API em
[docs/implementation-roadmap.md](docs/implementation-roadmap.md). Como
contribuir em [CONTRIBUTING.md](CONTRIBUTING.md); mudanças em
[CHANGELOG.md](CHANGELOG.md).

## Licença

[GNU Affero General Public License v3.0](LICENSE) ou posterior
(`AGPL-3.0-or-later`). Vale para o SDK e para o `medx-mcp`: quem distribuir
uma versão modificada, ou oferecê-la como serviço pela rede, precisa
publicar o código-fonte dela sob a mesma licença.
