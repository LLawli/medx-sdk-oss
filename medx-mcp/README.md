# medx-mcp

Servidor [MCP](https://modelcontextprotocol.io) que dá a um agente (Claude
Code, Codex e outros) acesso à MedX, o sistema de gestão da clínica: agenda,
pacientes, prontuário, financeiro, notas, chat interno e configurações. Fala
com a MedX pelo [medx-sdk](../README.md), a crate da raiz deste repositório.

- 46 ferramentas de leitura, sempre disponíveis.
- 14 ferramentas de escrita, só com `MEDX_MCP_ALLOW_WRITE=1`.
- Nenhuma exclusão e nenhuma troca de senha.

A lista completa, com os parâmetros, está em
[docs/ferramentas.md](docs/ferramentas.md). As decisões de design e as
medições por trás delas, em [docs/decisoes.md](docs/decisoes.md).

## Instalação

Precisa do Rust 1.88 ou mais novo e de acesso ao
repositório privado `LLawli/medx-sdk`.

```bash
git clone git@github.com:LLawli/medx-sdk.git
cd medx-sdk
cargo install --locked --path medx-mcp
```

O binário vai para `~/.cargo/bin/medx-mcp`.

## Configuração no Claude Code

```bash
claude mcp add medx -s user \
  -e MEDX_LOGIN_CREDENTIAL=medico@clinica.com.br \
  -e MEDX_PASSWORD_CREDENTIAL='senha' \
  -- medx-mcp
```

Para liberar as ferramentas de escrita, acrescente
`-e MEDX_MCP_ALLOW_WRITE=1`.

## Variáveis de ambiente

| Variável | Para quê |
|---|---|
| `MEDX_LOGIN_CREDENTIAL`, `MEDX_PASSWORD_CREDENTIAL` | Login na primeira chamada e de novo quando o token vence. Sem elas, o servidor usa a sessão salva pelo `medx-cli auth login`. |
| `MEDX_BASE_URL` | Host da MedX (com ou sem `/api`). Sem ela, o host em que a sessão foi aberta, ou o padrão do SDK. |
| `MEDX_CONFIG_DIR` | Onde fica o `session.json`, o mesmo do `medx-cli`. |
| `MEDX_MCP_ALLOW_WRITE` | `1` ou `true` liga as escritas. `0`, `false` ou ausente deixam só as leituras. Qualquer outro valor impede o servidor de subir. |
| `RUST_LOG` | Nível dos logs, que vão para o stderr (padrão `info`). |

## Coisas que é bom saber

- **Uma sessão por conta.** A MedX aceita uma sessão aberta por conta. Quando
  o servidor faz login, a sessão do navegador com a mesma conta cai, e
  vice-versa.
- **Os dados são de pacientes reais.** Tudo o que uma ferramenta devolve entra
  no contexto do modelo. Segredos de terceiros que a MedX devolve (a chave do
  RD Station, o token do calendário ICS) são retirados antes.
- **Listas têm limite.** Toda ferramenta de lista devolve até 50 itens por
  padrão (`limite` vai até 500) e avisa quando cortou.
- **O servidor não precisa de rede para subir.** O login acontece na primeira
  chamada de ferramenta. Sem sessão e sem credenciais, cada ferramenta
  responde com um erro explicando o que configurar.

## Desenvolvimento

O fluxo de trabalho (TDD, servidor fake da MedX, gates) está em
[AGENTS.md](AGENTS.md). Na raiz do repositório:

```bash
just check   # fmt e clippy estrito do medx-mcp, testes do workspace
```

Os testes nunca falam com a MedX real: usam um servidor HTTP fake com
fixtures sintéticas.
