# Decisões de design do medx-mcp

Cada entrada registra o que foi decidido, a evidência e o que reabriria a
decisão. Medições feitas em 2026-10-02, nunca contra a MedX real: fixtures
sintéticas, o servidor fake dos testes e protótipos descartáveis.

## Onde mora: crate do repositório medx-sdk

O servidor é a crate `medx-mcp`, membro do workspace cargo do medx-sdk, e
depende do SDK por `path`. Decisão do usuário: um repositório só, sem
dependência git por SSH, e uma mudança no SDK chega ao MCP no mesmo commit.

Comentários, documentação e commits em português, como o resto do
repositório. Nomes e descrições das ferramentas e dos parâmetros (o que o
modelo lê) também em português.

## Stack: Rust, rmcp, stdio

- rmcp 3.5 (SDK oficial de MCP em Rust), tokio, schemars para o schema dos
  parâmetros, edition 2024. O mesmo conjunto do jujutsu-mcp, que serve de
  modelo de processo.
- Transporte stdio apenas. O stdout leva só JSON-RPC; logs vão para o stderr
  via `tracing`, filtrados por `RUST_LOG`.
- O SDK é blocking (reqwest blocking). Toda chamada ao SDK roda em
  `spawn_blocking`. O cliente reqwest blocking não pode ser criado nem
  destruído dentro do runtime async (ele entra em pânico), então o
  `MedxClient` também é criado dentro do `spawn_blocking`.

Reabre se: o SDK ganhar uma API async.

## Mudanças que o SDK precisou (PR #18)

- **Login silencioso.** `auth::login_at` imprimia o progresso com `println!`.
  Num servidor stdio isso corrompe o JSON-RPC no primeiro login e em cada
  re-login depois de um 401 (`try_relogin` passa por `login_at`). O progresso
  virou `LoginStep`, e quem imprime é o medx-cli.
- **Tipos de resposta serializáveis.** Os 39 tipos de resposta só derivavam
  `Deserialize`. Agora serializam com os nomes do SDK (snake_case), não com
  os nomes crus da API, via `rename(deserialize = "...")`.

## Superfície: uma ferramenta por operação

- As anotações MCP (`readOnlyHint`, `destructiveHint`, `openWorldHint`) e as
  regras de permissão dos clientes valem por ferramenta. Uma ferramenta
  genérica com um parâmetro `operacao` misturaria leitura e escrita sob a
  mesma permissão.
- Leituras: sempre disponíveis.
- Escritas: só com `MEDX_MCP_ALLOW_WRITE=1`. Sem a variável, as ferramentas
  de escrita **nem aparecem** no `tools/list`: o modelo não gasta contexto com
  elas nem tenta chamá-las para receber um erro.
- Fora de propósito: exclusões (`delete_contact`, `delete_nota`,
  `delete_appointment`, `remove_agenda_block`) e troca de senha. Também ficam
  fora as chamadas internas do webapp sem valor para o modelo
  (`is_otp_or_expired`, `sync_version`, `new_contact_id`, que reserva um id).
- Também ficam fora a avaliação da MedX (`insert_nota_cliente`, que manda
  feedback ao fornecedor, não uma nota de paciente), o período de teste da
  conta (`trial_info`) e o registro de e-mail enviado (`log_email`, só log).
- Escritas que falam com terceiros (WhatsApp de confirmação para o paciente)
  levam `openWorldHint`. Atualizações que sobrescrevem dados levam
  `destructiveHint`; criações, não.
- Atualizações leem o registro atual e mudam só o que o modelo informou, numa
  única chamada à fila: `atualizar_paciente`, `remarcar_agendamento`,
  `editar_nota`, `editar_registro_prontuario`, `atualizar_sumario_prontuario`.
  O webapp manda o objeto inteiro; mandar só o que mudou apagaria o resto.
- O autor de nota, registro de prontuário e mensagem de chat é o usuário
  logado (`current_user().user_id`), a mesma convenção do medx-cli.
- O registro de prontuário chega como texto puro e vira HTML (o prontuário do
  webapp é um editor rico): parágrafos, quebras de linha e escape de `<`,
  `>`, `&`, `"`.

Reabre se: o usuário quiser alguma exclusão, ou o modelo cair repetidamente
numa operação que ficou de fora.

### Custo do catálogo

Medido no `tools/list` do binário (JSON compacto, cerca de 3 bytes por
token): 45 leituras somam 22,5 KB, perto de 7,5 mil tokens; com as 14
escritas, 36,2 KB, perto de 12 mil. Esconder as escritas sem
`MEDX_MCP_ALLOW_WRITE` economiza cerca de 4,6 mil tokens por sessão. Para
comparação, um servidor de jj com 55 ferramentas medido para o jujutsu-mcp
custava 9,5 mil.

Reabre se: o catálogo passar de 15 mil tokens, ou o cliente não adiar a
carga das ferramentas (o Claude Code atual carrega as de MCP sob demanda,
pela busca de ferramentas).

### Escritas que esperam decisão do usuário

Estas escritas existem no SDK e ficaram de fora desta versão, porque cada uma
tem um risco que não é técnico:

- **Faturas e cobranças** (`update_invoice`, `create_pre_payment`). Mexem em
  dinheiro: a cobrança gera um link de pagamento Stone/Pagar.me para o
  paciente. O `AttendanceDto` também carrega procedimentos, rateio entre
  profissionais e pagamentos, e um erro do modelo ali é caro de desfazer.
- **Questionários e local de atendimento** (`insert_quest`,
  `update_local_atendimento`). Mudam a configuração de marketing da clínica,
  usada em mensagens que vão para todos os pacientes.
- **Anexar arquivo ao prontuário** (`attach_files`). Exige o arquivo inteiro
  em base64 no parâmetro da ferramenta, o que custa contexto e raramente o
  modelo tem o arquivo.

## Saída: JSON compacto em texto, sem `outputSchema`

Medição: um schema de saída emulado no formato do schemars para as 43
ferramentas de leitura, a partir das structs reais do SDK (520 campos), soma
38 KB, cerca de 12,7 mil tokens, no `tools/list` de toda sessão. O Claude
Code entrega ao modelo o `content` em texto, então o schema não traria ganho a
esse cliente, e exigiria `schemars` como dependência do SDK.

- Cada resposta é um bloco de texto com o JSON do tipo do SDK.
- Listas saem como array JSON, sem objeto em volta (o embrulho só seria
  exigido por `structuredContent`).
- JSON compacto: nas fixtures do SDK, o compacto tem 79% do tamanho do
  indentado, sem perder nada.
- Campos vazios ficam. Cortar `""`, `null`, `[]` e `{}` economizaria mais 12%
  nas fixtures, mas apagaria a diferença entre "vazio" e "não veio", e o
  modelo não teria como saber quais campos existem.

Reabre se: algum cliente passar a usar `structuredContent`, ou as respostas
reais ficarem grandes demais (aí o corte de vazios é o primeiro passo).

## Segredos ficam fora da resposta

Algumas respostas da MedX trazem credenciais de terceiros. A chave de API do
RD Station no usuário atual (`rd_station_key`) é tirada antes de responder.
Nada que o modelo faça precisa dela, e o que entra no contexto pode acabar
repetido numa resposta ou num log.

Pelo mesmo motivo não existe ferramenta do calendário ICS. A varredura real
(2026-10-02) mostrou que o endpoint do SDK (`localizadorICS/GetLocalizadorICS`)
responde 404; o webapp usa `ICS/GetLocalizador`, que devolve só um id, e o
link do feed é `ics/getics?id=<id>`. Esse id é a credencial do feed: quem tem
o link lê a agenda da clínica. Sem ele, a ferramenta não teria o que mostrar.

Reabre se: alguma tarefa real precisar de um desses valores.

## Listas: `limite`, com aviso de corte

O MedX não pagina nada, e o Claude Code avisa ou corta saída de ferramenta
acima de 25 mil tokens (`MAX_MCP_OUTPUT_TOKENS`). Toda ferramenta que devolve
lista aceita `limite` (padrão 50, de 1 a 500). Quando a lista passa do
limite, a resposta traz um segundo bloco de texto dizendo quantos itens
vieram e quantos foram mostrados, para o modelo refinar a busca ou pedir
mais.

Reabre se: o limite padrão se mostrar curto (o modelo pedindo mais quase
sempre) ou longo (respostas cortadas pelo cliente).

### Medido contra a MedX real

Na varredura de leitura de 2026-10-02 (só tamanhos, nenhum dado impresso),
um paciente com 30 registros de prontuário deu 35,9 KB de JSON, cerca de 12
mil tokens; o maior registro tinha 11,6 KB, a mediana, 40 bytes. Tirar o HTML
do `content` reduziria o JSON em 31% (para 24,6 KB), ao custo de perder
tabelas e negrito. A redução não muda a ordem de grandeza, então o HTML fica.
`listar_usuarios` com 23 usuários deu 21,9 KB (cerca de 7 mil tokens), por
causa das permissões e horários de cada um.

Reabre se: prontuários reais passarem com frequência dos 25 mil tokens com o
limite padrão.

## Sessão e credenciais

- Variáveis: `MEDX_LOGIN_CREDENTIAL` e `MEDX_PASSWORD_CREDENTIAL` (login e
  re-login em 401), `MEDX_BASE_URL` (escolhe o host), `MEDX_CONFIG_DIR` (onde
  fica o `session.json`, do SDK).
- O servidor reaproveita o `session.json` do medx-cli. O host segue a mesma
  regra do CLI: `MEDX_BASE_URL` vence; sem ela, o host em que o token foi
  emitido; sem sessão, o host padrão do SDK.
- O cliente é criado na primeira chamada de ferramenta, não na partida: o
  handshake não depende de rede, e um servidor sem credenciais ainda sobe e
  explica o que falta no erro da ferramenta.
- Sem sessão salva e sem credenciais, toda ferramenta devolve um erro dizendo
  como configurar.
- Conta: a do próprio usuário. A MedX aceita uma sessão por conta; o login do
  MCP derruba a do navegador e vice-versa. O usuário aceitou isso.

## Uma chamada à MedX por vez

As chamadas ao SDK passam por uma fila de um lugar só. Com duas chamadas em
paralelo e o token vencido, as duas recebem 401 e as duas fazem re-login; como
a MedX aceita uma sessão por conta, o segundo login derruba o primeiro, e a
repetição da primeira chamada falha de novo. Serializar custa pouco: o
modelo raramente dispara chamadas em paralelo, e cada uma leva o tempo de uma
request HTTP.

Reabre se: o SDK passar a coordenar o re-login entre threads.

## Erros

Qualquer falha (MedX fora, 4xx/5xx, credencial inválida, parâmetro inválido,
sessão ausente) volta como resultado de ferramenta com `isError: true` e uma
mensagem em português que o modelo consegue usar. Erro de protocolo fica só
para ferramenta desconhecida. A spec (2025-11-25, SEP-1303) trata validação de
entrada como erro de execução da ferramenta, para o modelo corrigir a chamada.

## Testes

- Servidor HTTP fake da MedX (só `std::net`, sem dependência) com fixtures
  sintéticas, nunca dado de paciente. Ele registra cada request (método,
  caminho, corpo) para os testes conferirem o que o servidor enviou.
- Testes de integração rodam o servidor em processo atrás de um cliente rmcp
  num canal duplex em memória, com o SDK real falando com o fake.
- Testes do binário sobem o `medx-mcp` como processo filho, com
  `MEDX_CONFIG_DIR` num diretório temporário, e conferem que o stdout só leva
  JSON-RPC, inclusive durante um login.
- Um smoke test e uma varredura de todas as leituras sem efeito colateral,
  ambos `#[ignore]`, contra a MedX real, rodados só quando o usuário pede.
  A varredura imprime só status e tamanhos, nenhum dado.
- Validado no Windows 11 (VM dockur/windows, MSVC) em 2026-10-02: o
  workspace inteiro contra as fixtures, clippy e fmt do MCP, e as leituras
  ao vivo, com o mesmo resultado do Linux.
- Gates (`just check` na raiz): `cargo fmt` e `cargo clippy -D warnings` da
  crate do MCP, e `cargo test --workspace --locked`. O SDK não entra no fmt e
  no clippy estritos: o código dele não está formatado nem limpo hoje, e
  isso é outra mudança.
