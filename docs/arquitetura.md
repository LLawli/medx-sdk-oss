# Arquitetura do SDK

Como o `medx-sdk` fala com a MedX e por que cada peça é como é. O servidor
MCP tem o próprio documento: [medx-mcp/docs/decisoes.md](../medx-mcp/docs/decisoes.md).

## Visão geral

```
medx-cli (src/bin/cli.rs)     medx-mcp (medx-mcp/)
            \                   /
             MedxClient (client.rs)       módulos de domínio: agenda, contacts,
               |   bearer, re-login em 401   prontuario, financas, hoje, chat, ...
               |                             (cada um um `impl MedxClient`)
             auth.rs  ->  crypto.rs (RSA-OAEP)
               |
             session.rs  ->  session.json
```

- **Stack:** Rust (edição 2021), `reqwest` **blocking**, `serde`, `thiserror`,
  `rsa`. Sem runtime assíncrono no SDK; o `medx-mcp`, que é assíncrono, mantém
  o `MedxClient` numa thread própria e manda as chamadas para ela, uma por vez.
- **Um `impl MedxClient` por domínio.** Cada módulo (`agenda.rs`,
  `contacts.rs`, ...) traz os DTOs daquele pedaço da API e os métodos que os
  usam. `lib.rs` reexporta os tipos públicos.
- **A CLI** é um binário só (`src/bin/cli.rs`), com um `dispatch_*` por
  recurso, sobre a mesma API pública que qualquer usuário do SDK vê.
- **TLS com rustls**, não OpenSSL. Os binários do release são estáticos
  (musl no Linux) e não dependem da `libssl` do sistema; com OpenSSL, o
  `medx-cli` de Linux ligava com a `libssl.so.3` e quebrava onde ela não
  existe. Medido em outubro de 2026: os dois hosts da MedX fecham o
  handshake com rustls (v65 em HTTP/1.1, care-app65 em HTTP/2) e o binário
  passa de 5,5 para 7,5 MB. Os certificados raiz são os do `webpki-roots`,
  embutidos, e não os do sistema. Reabre se a MedX trocar para um
  certificado que o `webpki-roots` não aceite, ou se alguém precisar de uma
  CA corporativa instalada no sistema.
- **Histórico:** o projeto começou em Go e foi migrado para Rust.

## Login (`auth.rs`, `crypto.rs`)

O login reproduz o do webapp, em quatro chamadas:

1. `GET LoginUnificado/VerificaEmailCripto?Email=` devolve o `dbId` (a base
   da clínica) a partir do e-mail.
2. `GET security/getkeys` devolve uma chave RSA pública, no XML
   `<RSAKeyValue>` do .NET, e o id dela.
3. A senha é cifrada com RSA-OAEP e SHA-1, o mesmo esquema do
   `RSACryptoServiceProvider.Encrypt(data, true)` do .NET, e vai em Base64.
4. `POST LoginUnificado/loginV3` devolve o token.

Se a conta já tem sessão aberta, o passo 4 responde 400 com "usuário já
logado" e o token antigo na mensagem. O SDK chama
`security/removetokeninuse` com esse token e tenta de novo. É por isso que
um login pelo SDK derruba a sessão do navegador: a MedX só aceita uma por
conta.

O SDK só **cifra** com chave pública; nunca tem chave privada. Por isso o
aviso RUSTSEC-2023-0071 do `rsa` (ataque de tempo ao decifrar) não se
aplica e está ignorado em `.cargo/audit.toml`.

`login_at_with_progress` relata cada etapa (`LoginStep`), o que a CLI usa
para mostrar o progresso.

## Cliente (`client.rs`)

- Toda request passa por `MedxClient::execute`, que põe
  `Authorization: Bearer <token>` e mapeia o status: 2xx segue, 401 vira
  `MedxError::InvalidCredentials`, o resto vira `MedxError::Api` com a
  `Message` do corpo, quando houver.
- **Re-login em 401.** Se o cliente foi criado com credenciais (`login`,
  `from_session_with_credentials`), um 401 dispara um login novo no mesmo
  host e a request é repetida. Sem credenciais (`from_session`), o 401 sobe
  como erro.
- **Host.** O webapp alterna entre `v65.medx.med.br` e
  `care-app65.medx.med.br`, e o token só vale no host em que foi emitido.
  `DEFAULT_HOST` aparece uma vez no código (`tests/host_tests.rs` reprova
  qualquer outra ocorrência); `normalize_host` aceita a origem com ou sem
  `/api`; o re-login sempre volta para o host do próprio cliente.

## Sessão (`session.rs`)

Todo login bem-sucedido grava `session.json` (token, e-mail, `dbId`, host)
em `MEDX_CONFIG_DIR`, ou em `medx-sdk/` dentro da pasta de configuração do
sistema. A CLI e o `medx-mcp` reaproveitam o mesmo arquivo. Um
`session.json` antigo, sem o campo `host`, cai no host padrão.

## Convenções da API da MedX

Coisas que parecem estranhas no código e são exigência da API:

- **Datas em horário local, sem fuso.** A MedX trabalha com
  `YYYY-MM-DDTHH:MM:SS` local (`util::current_datetime_str`). Mandar UTC
  desloca notas e mensagens em horas.
- **Query string em codificação de formulário.** Todo texto que entra numa
  query passa por `util::encode_query_value` (espaço vira `+`, o resto
  `%XX`), como o ASP.NET da MedX espera.
- **Respostas tolerantes.** A API devolve `null`, string vazia ou número
  dentro de string onde se esperaria número ou lista. Os módulos têm
  desserializadores (`de_null_i64`, `de_null_f64`, `de_null_str`, ...) que
  transformam isso no valor padrão, e vários métodos tratam corpo `"null"`
  como lista vazia.
- **Recusa com status 200.** Alguns endpoints respondem 200 com um texto de
  recusa (por exemplo `"negado"`) em vez de status de erro; para esses, o
  cliente tem as variantes `*_text`, que deixam o corpo ser inspecionado.

## Testes

| Camada | Onde | Rede |
|---|---|---|
| Unitários | `#[cfg(test)]` nos módulos | não |
| Serialização | `tests/serializacao_tests.rs`, com fixtures sintéticas | não |
| HTTP local | `tests/{login,host,ics,urlencode}_tests.rs`, com servidor HTTP em `127.0.0.1` | não |
| CLI | `tests/cli_e2e_tests.rs`, todos `#[ignore]` | MedX real |
| Ao vivo | os demais `tests/*_tests.rs`, `#[ignore]` | MedX real, com a própria conta |

Os testes ao vivo leem credenciais e paciente de teste do ambiente ou do
`.env` (ver o [README](../README.md#desenvolvimento)) e desfazem o que criam:
agendamentos e contatos saem por guardas (`AppointmentGuard`,
`ContactGuard`), e o histórico médico volta ao estado original. Como a MedX
aceita uma sessão por conta, `tests/common` serializa o login entre os
binários de teste com um arquivo de trava e um diretório de sessão
compartilhado.

Nenhum dado real entra no repositório: fixtures são sintéticas, e paciente e
arquivo de teste vêm só do ambiente.
