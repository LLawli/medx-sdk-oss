# MedX SDK — Roadmap de Implementação

> Ordem definida por **dependência técnica** e **prioridade de reuso**.
> Cada etapa só pode ser iniciada quando todas as suas dependências estiverem concluídas.

---

## Etapa 0 — Autenticação ✅ (concluída)

**Bloco fundamental. Nada mais funciona sem ele.**

| # | Item | Endpoints | Status |
|---|------|-----------|--------|
| 0.1 | Login com RSA-OAEP | `POST /api/LoginUnificado/loginV3` | ✅ |
| 0.2 | Resolução de dbId por e-mail | `GET /api/LoginUnificado/VerificaEmailCripto` | ✅ |
| 0.3 | Invalidação automática de sessão ativa | `POST /api/security/removetokeninuse` | ✅ |
| 0.4 | Persistência de sessão em disco | `~/.config/medx-sdk/session.json` | ✅ |

---

## Etapa 1 — Cliente HTTP base

**Dependência:** Etapa 0
**Motivo:** Todas as etapas seguintes precisam de um cliente com autenticação injetada, retry automático e mapeamento de erros.

| # | Item | Detalhe |
|---|------|---------|
| 1.1 | `MedxClient` struct central | Encapsula `reqwest::Client` + `Session` + `base_url` |
| 1.2 | Injeção automática de `Authorization: Bearer <token>` | Middleware / wrapper sobre todos os métodos HTTP |
| 1.3 | Mapeamento de erros HTTP → `MedxError` | 401 → `InvalidCredentials`, 400 → `Api { .. }`, etc. |
| 1.4 | Retry automático em 401 | Re-login silencioso usando credenciais salvas |
| 1.5 | Helpers internos `get / post / put / delete` | Tipados, evitam repetição de lógica em cada módulo |

---

## Etapa 2 — Usuário & Configurações globais

**Dependência:** Etapa 1
**Motivo:** `security/getcurrentuser` e `settings/GetGeneralParameters` são chamados em praticamente todos os módulos para montar contexto inicial. Devem existir antes de qualquer módulo de negócio.

| # | Item | Endpoints |
|---|------|-----------|
| 2.1 | Dados do usuário logado | `GET security/getcurrentuser` |
| 2.2 | Parâmetros gerais da conta | `GET settings/GetGeneralParameters` |
| 2.3 | Parâmetros de cores/temas | `GET parametrosCores/getallparametroscores` |
| 2.4 | Lista de usuários da clínica | `GET Users/GetUsers` |
| 2.5 | Troca de senha | `GET security/getkeys` + `POST usuarios/ChangeMePassword` |

---

## Etapa 3 — Contatos (entidade central)

**Dependência:** Etapa 2
**Motivo:** `contatos/GetContatosFichaById` é referenciado em **todos** os módulos (agenda, prontuário, finanças). Sem o módulo de contatos não é possível operar nenhum outro.

| # | Item | Endpoints |
|---|------|-----------|
| 3.1 | Buscar contato por ID | `GET contatos/GetContatosFichaById?Id=` |
| 3.2 | Foto do contato (base64) | `GET contatos/GetFotoBase64?Id=` |
| 3.3 | Convênios do contato | `GET contatos/GetContatosConvenios` |
| 3.4 | Criar contato | `POST contatos/InsertContato` |
| 3.5 | Atualizar contato | `PUT contatos/UpdateContato` |
| 3.6 | Excluir contato | `DELETE contatos/DeleteContatoById?Id=` |
| 3.7 | Upload de foto (Azure Blob) | `POST Upload/PostFileAzure?replaceFile=true` |
| 3.8 | URL de arquivo Azure | `GET azure/getfileurl?blobname=` |

---

## Etapa 4 — Agenda

**Dependência:** Etapa 3 (usa contatos), Etapa 2 (usa usuários e parâmetros)
**Motivo:** Segundo módulo mais central — prontuário e finanças dependem dos atendimentos gerados pela agenda.

| # | Item | Endpoints |
|---|------|-----------|
| 4.1 | Parâmetros da agenda | `GET agenda/GetAllParametersAgenda` |
| 4.2 | Setores da agenda | `GET agenda/getagendasetores` |
| 4.3 | Agenda do dia por usuário | `GET hoje/GetAgendaDiaUsuario?Id=&Dt=` |
| 4.4 | Criar agendamento | `POST agenda/` |
| 4.5 | Atualizar agendamento | `PUT agenda/UpdateAgendamento` |
| 4.6 | Atualizar status do agendamento | `PUT agenda/updatestatus` |
| 4.7 | Confirmar via WhatsApp | `POST agenda/ConfirmaAgendamentoWhatsapp?Iddoagendamento=` |
| 4.8 | Relatório de agenda | `POST report/ReportAgenda` |
| 4.9 | Relatório no-show | `POST report/ReportAgendaNoShow` |

---

## Etapa 5 — Prontuário

**Dependência:** Etapa 4 (usa agenda), Etapa 3 (usa contatos)
**Motivo:** Opera sobre o atendimento criado na agenda. Requer contexto de convênios, procedimentos e formulários.

| # | Item | Endpoints |
|---|------|-----------|
| 5.1 | Sumário do histórico médico | `GET prontuario/GetMedicalHistorySummary?Pacid=` |
| 5.2 | Busca em prontuários | `GET prontuario/GetProntuarioBusca` |
| 5.3 | Palavras-chave do prontuário | `GET prontuario/GetMedicalKeywords` |
| 5.4 | Criar registro de prontuário | `POST prontuario/InsertMedicalHistory` |
| 5.5 | Atualizar registro de prontuário | `PUT prontuario/UpdateMedicalHistory` |
| 5.6 | Anexar arquivos ao prontuário | `POST prontuario/AttachFiles` |
| 5.7 | Relatório de prontuário | `POST /report/reportprontuario` |
| 5.8 | Convênios disponíveis | `GET Convenios/GetAllConvenios` |
| 5.9 | Procedimentos por convênio | `GET Convenios/GetProcedimentosByIdConvenio?IddoConvenio=` |
| 5.10 | Todos os procedimentos | `GET Procedimentos/GetAllProcedimentos` |
| 5.11 | Formulários da clínica | `GET formularios/getformularios` |
| 5.12 | HTML de formulário específico | `GET formularios/GetFormulariosHTML?iddoformulario=` |
| 5.13 | Módulos/registros por paciente | `GET modulos/GetRecords?pacid=&modulo=` |
| 5.14 | Unidades de negócio (UN) | `GET UN/GetAllUN` |

---

## Etapa 6 — Finanças / Atendimentos

**Dependência:** Etapa 5 (prontuário gera atendimento), Etapa 3 (vinculado ao paciente)
**Motivo:** Faturamento só existe sobre atendimentos já realizados (criados via agenda/prontuário).

| # | Item | Endpoints |
|---|------|-----------|
| 6.1 | Atendimentos por paciente | `GET atendimentos/GetAtendimentosByIdPac?Id=` |
| 6.2 | Todos os atendimentos (filtrado) | `GET atendimentos/GetAllAtendimentos?filterstring=&filter=` |
| 6.3 | Atualizar fatura geral | `POST Atendimentos/UpdateFaturaGeral` |
| 6.4 | Gerar cobrança Stone/Pagar.me | `POST stone/CreatePrePayment` |

---

## Etapa 7 — Notificações & Comunicação

**Dependência:** Etapa 3 (precisa de contato), Etapa 4 (contexto de agendamento)
**Motivo:** Notificações são sempre disparadas sobre entidades já existentes (contato + agendamento).

| # | Item | Endpoints |
|---|------|-----------|
| 7.1 | Configurações de mensagens da clínica | `GET marketing/GetClienteSettings` |
| 7.2 | Envio de e-mail (log) | `POST notifications/InsertMailLogger` |
| 7.3 | Envio de SMS | `POST notifications/InsertSMSGateWay` |
| 7.4 | SMS de confirmação de consulta | `POST marketing/SMSConfirmaConsulta` |
| 7.5 | Link WhatsApp em lote | `POST whatsapp/gerarLinkWhatsappAppLote` |
| 7.6 | Confirmação de consulta via WhatsApp Oficial | `POST whatsApp/EnviaConfirmacaoConsultaWhatsappOficial` |

---

## Etapa 8 — Marketing & Eventos

**Dependência:** Etapa 7 (usa notificações), Etapa 3 (usa contatos)
**Motivo:** Módulo de relacionamento secundário; depende de comunicação e cadastro de pacientes.

| # | Item | Endpoints |
|---|------|-----------|
| 8.1 | Lista todos os eventos | `GET eventos/getAllEventos` |
| 8.2 | Aplicar questionário | `POST marketing/InsertQuests` |
| 8.3 | Questionários disponíveis | `GET settings/XML_GetQuests` |
| 8.4 | Diagnóstico QP | `GET diagnosticoqp/GetAllDiagnosticoQP` |
| 8.5 | Atualizar nome da clínica no local de atendimento | `POST marketing/UpdateLocalAtendimentoNomeClinica` |

---

## Etapa 9 — Dashboard (Hoje)

**Dependência:** Etapas 2–6 (agrega dados de todos os módulos)
**Motivo:** O dashboard é uma composição de dados de vários módulos. Implementar por último garante que todas as entidades dependentes já existam.

| # | Item | Endpoints |
|---|------|-----------|
| 9.1 | Notificações do dia | `GET hoje/GetHojeNotificacoes` |
| 9.2 | Últimos atendidos | `GET hoje/GetUltimosAtendidos` |
| 9.3 | Informações trial da conta | `GET adm/infosTrial` |
| 9.4 | Notas internas | `GET hoje/GetNotas` |
| 9.5 | Criar nota | `POST hoje/InsertNota` |
| 9.6 | Criar nota de paciente | `POST hoje/InsertNotaCliente` |
| 9.7 | Atualizar nota | `PUT hoje/UpdateNota` |
| 9.8 | Excluir nota | `DELETE hoje/DeleteNotaById?Id=` |
| 9.9 | Verificar OTP/expiração | `GET hoje/IsOTPOrExpired` |
| 9.10 | Sync de versão | `GET SyncVersion/SyncData55To60` |

---

## Etapa 10 — MedX IA (Assistente de Prontuário)

**Dependência:** Etapa 5 (prontuário), Etapa 2 (usuário logado)
**Motivo:** Feature avançada sobre prontuário existente. Menor prioridade, maior complexidade.

| # | Item | Endpoints |
|---|------|-----------|
| 10.1 | Buscar histórico do assistente | `GET medxia/GetMessagesAssistenteProntuario?pacienteId=` |
| 10.2 | Processar prontuário com IA | `POST medxia/IAProntuario` |
| 10.3 | Enviar mensagem ao assistente | `POST medxia/EnviaMensagenAssisitenteProntuario` |
| 10.4 | Mensagem padrão do assistente | `POST medxia/EnviaMensagemPadraoAssisente` |

---

## Etapa 11 — Ajustes & Administração

**Dependência:** Todas as etapas anteriores (configurações afetam todos os módulos)
**Motivo:** Tipicamente acessado por admins. Não bloqueia uso do sistema.

| # | Item | Endpoints |
|---|------|-----------|
| 11.1 | Parâmetros avançados da clínica | `GET settings/GetGeneralParameters` (modo escrita) |
| 11.2 | Troca de senha | `GET security/getkeys` + `POST usuarios/ChangeMePassword` |
| 11.3 | Localizador ICS (calendário externo) | endpoints do `ajustes_localizadorICS.js` |
| 11.4 | Relatórios disponíveis | `GET report/listarelatorios` |
| 11.5 | Autodocs / pastas de documentos | `GET autodocs/getfoldersdocs?filter=` |

---

## Visão geral das dependências

```
0. Auth ──────────────────────────────────────────────────────────┐
1. HTTP Client ────────────────────────────────────────────────── │
2. Usuário & Config ─────────────────────────────────────────┐   │
3. Contatos ─────────────────────────────────────────────┐   │   │
4. Agenda ──────────────────────────────────────────┐    │   │   │
5. Prontuário ──────────────────────────────────┐   │    │   │   │
6. Finanças ────────────────────────────────┐   │   │    │   │   │
7. Notificações ────────────────────────┐   │   │   │    │   │   │
8. Marketing ───────────────────────┐   │   │   │   │    │   │   │
9. Dashboard ───────────────────┐   │   │   │   │   │    │   │   │
10. MedX IA ────────────────┐   │   │   │   │   │   │    │   │   │
11. Ajustes ─────────────┐  │   │   │   │   │   │   │    │   │   │
                         └──┴───┴───┴───┴───┴───┴───┴────┴───┘
                                     depende de →
```

---

## Transversais (implementar junto com cada etapa)

| Item | Quando |
|------|--------|
| Paginação (`limit` / `offset` / cursor) | Etapa 3 em diante |
| Upload de arquivos (`multipart/form-data`) | Etapa 3 (foto contato) |
| Cache local de respostas estáticas | Etapa 2 (parâmetros e usuário) |
| Logging estruturado | Etapa 1 |
| Testes de integração por módulo | Junto com cada etapa |
