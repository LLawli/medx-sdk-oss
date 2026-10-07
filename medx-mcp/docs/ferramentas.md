# Ferramentas do medx-mcp

Gerado a partir do `tools/list` do servidor com `MEDX_MCP_ALLOW_WRITE=1`.
Sem a variável, só as leituras existem. Toda ferramenta de lista aceita
`limite` (padrão 50, até 500). Datas em AAAA-MM-DD; data e hora em
AAAA-MM-DDTHH:MM, hora local.

## Usuários

### `usuario_atual`

Usuário logado na MedX: nome, e-mail, plano e bloqueios da conta.

### `listar_usuarios`

Usuários da clínica (profissionais e equipe), com permissões e horários de trabalho. O `id` de um profissional é o que as ferramentas de agenda pedem.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

## Agenda

### `parametros_agenda`

Parâmetros da agenda: rótulos e cores de cada status (o número do status é a posição do rótulo), profissionais, setores, duração padrão e horário de funcionamento.

### `listar_profissionais_agenda`

Profissionais com agenda na clínica, com setor e horário de trabalho de cada dia da semana.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `listar_setores_agenda`

Setores (salas, consultórios) da agenda.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `agenda_do_dia`

Agendamentos de um profissional num dia, incluindo bloqueios de horário. `contact_id` é o paciente (0 ou ausente: bloqueio); `arrived_at` e `attended_at` em 0001-01-01 significam que ainda não aconteceu.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `data` | sim | Data no formato AAAA-MM-DD. |
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |
| `profissional_id` | sim | Id do profissional (o `id` de `listar_profissionais_agenda`). |

### `bloqueios_do_dia`

Só os bloqueios de horário (sem paciente) de um profissional num dia.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `data` | sim | Data no formato AAAA-MM-DD. |
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |
| `profissional_id` | sim | Id do profissional (o `id` de `listar_profissionais_agenda`). |

### `relatorio_agenda`

Gera na MedX o relatório da agenda de um período e devolve o link do PDF (`file_url`; vazio quando não há agendamentos).

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `busca` | não | Texto para filtrar o relatório. |
| `fim` | sim | Data final, AAAA-MM-DD, inclusive. |
| `incluir_desmarcados` | não | Incluir os agendamentos desmarcados. Padrão: não. |
| `inicio` | sim | Data inicial, AAAA-MM-DD. |
| `profissional_id` | não | Id do profissional. Ausente ou 0: todos. |
| `status` | não | Só agendamentos com este status (a posição do rótulo em `parametros_agenda`). Ausente: todos. |

### `relatorio_faltas`

Gera na MedX o relatório de faltas (no-show) de um período e devolve o link do PDF (`file_url`; vazio quando não há faltas).

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `fim` | sim | Data final, AAAA-MM-DD, inclusive. |
| `inicio` | sim | Data inicial, AAAA-MM-DD. |
| `profissional_id` | não | Id do profissional. Ausente ou 0: todos. |

## Painel do dia

### `notificacoes_de_hoje`

Notificações do painel do dia (aniversários, retornos e avisos).

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `ultimos_atendidos`

Últimos pacientes atendidos, com a data do último atendimento.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `listar_notas`

Notas do painel do usuário logado.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

## Pacientes

### `buscar_pacientes`

Busca pacientes pelo nome (ou lista os aniversariantes do mês), com contato e plano de saúde. Use o `id` nas outras ferramentas de paciente.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `aniversariantes_do_mes` | não | Só os aniversariantes do mês corrente. |
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |
| `nome` | não | Nome ou parte do nome. Obrigatório, a não ser com `aniversariantes_do_mes`. |

### `ver_paciente`

Ficha completa de um paciente: documentos, contatos, endereço, convênio e observações.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |

### `pacientes_homonimos`

Pacientes já cadastrados com o mesmo nome, sexo e nascimento. Use antes de cadastrar alguém, para não duplicar.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |
| `nascimento` | sim | Data de nascimento, AAAA-MM-DD. |
| `nome` | sim | Nome completo. |
| `sexo` | sim | Sexo: M ou F. |

### `listar_planos_de_saude`

Planos de saúde (convênios) aceitos no cadastro de pacientes.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `foto_paciente`

Foto do paciente, como imagem.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |

## Prontuário

### `sumario_prontuario`

Sumário fixo do paciente: diagnóstico, história patológica pregressa (hpp), medicamentos, alergias e campo livre.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |

### `ver_prontuario`

Registros do prontuário de um paciente, do mais recente para o mais antigo. `content` é HTML. Registros com arquivo têm `tipo_doc` e `classe` (use `link_arquivo_prontuario`).

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |

### `galeria_de_fotos`

Fotos e imagens do prontuário de um paciente, da mais recente para a mais antiga. Use `link_arquivo_prontuario` com a `classe` para abrir.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |

### `buscar_no_prontuario`

Busca um texto nos registros do prontuário de um paciente; resultado do mais recente para o mais antigo.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |
| `texto` | sim | Texto a procurar nos registros. |

### `link_arquivo_prontuario`

Link temporário para abrir o arquivo (PDF, imagem) de um registro do prontuário.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `classe` | sim | O campo `classe` do registro de prontuário que tem arquivo. |

### `palavras_chave_prontuario`

Palavras-chave cadastradas para classificar registros do prontuário.

### `listar_convenios`

Convênios cadastrados na clínica.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `procedimentos_do_convenio`

Procedimentos de um convênio, com o valor pago por ele.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `convenio_id` | sim | Id do convênio (o `id` de `listar_convenios`). |
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `listar_procedimentos`

Procedimentos da clínica, com preço base, comissão e sessões.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `listar_formularios`

Formulários do prontuário (anamnese, evolução e outros).

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `ver_formulario`

HTML de um formulário do prontuário.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `formulario_id` | sim | Id do formulário (o `id` de `listar_formularios`). |

### `registros_do_modulo`

Registros de um módulo personalizado do prontuário para um paciente.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |
| `modulo` | sim | Nome do módulo, como a MedX o chama. Módulo inexistente: a MedX responde 404. |
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |

### `relatorio_prontuario`

Gera na MedX o PDF do prontuário de um paciente num período e devolve o link (`file_url`).

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `fim` | sim | Data final, AAAA-MM-DD, inclusive. |
| `inicio` | sim | Data inicial, AAAA-MM-DD. |
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |

### `listar_unidades`

Unidades de negócio (filiais) da clínica.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

## Financeiro

### `atendimentos_do_paciente`

Faturas de atendimento de um paciente, da mais recente para a mais antiga: valor, total pago, desconto, se está fechada (`closed`) e se é orçamento (`budget`).

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |

### `listar_atendimentos`

Faturas de atendimento da clínica, da mais recente para a mais antiga, com os filtros do webapp e busca livre.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `busca` | não | Texto de busca livre (nome do paciente, recibo). |
| `filtro` | não | Filtro do webapp. Ausente: todos. |
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

## Configurações

### `listar_eventos`

Eventos de marketing cadastrados (lembretes, retornos, campanhas).

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `listar_questionarios`

Questionários que a clínica envia aos pacientes.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `listar_diagnosticos_qp`

Diagnósticos de queixa principal (QP) cadastrados, com o tempo de consulta de cada um. O `id` é o `diagnostic_id` dos agendamentos.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `modelos_de_mensagem`

Modelos das mensagens que a clínica manda aos pacientes (SMS, WhatsApp, pré-cadastro, questionário) e as redes sociais.

### `parametros_gerais`

Parâmetros gerais da clínica: horário de funcionamento, duração padrão, remetente de SMS, metas nutricionais e atalhos.

### `parametros_de_cores`

Rótulos e cores dos status de agendamento.

### `listar_relatorios`

Relatórios disponíveis na MedX.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `pastas_de_documentos`

Pastas de documentos automáticos (modelos de documento da clínica).

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `filtro` | não | Filtro das pastas (o `filter_key` de uma pasta). Ausente: todas. |
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

## Chat interno

### `listar_usuarios_chat`

Usuários do chat interno, se estão online e quantas mensagens não lidas cada um mandou (`unread`).

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

### `historico_chat`

Conversa do chat interno com um usuário, da mensagem mais recente para a mais antiga.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |
| `usuario_id` | sim | Id do outro usuário da conversa (o `id` de `listar_usuarios_chat`). |

### `mensagens_nao_lidas_chat`

Total de mensagens não lidas no chat interno.

### `mensagens_recebidas_chat`

Mensagens recebidas no chat interno que ainda não foram lidas.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `limite` | não | Máximo de itens na resposta (padrão 50, até 500). |

## Escrita: pacientes

### `cadastrar_paciente`

Cadastra um paciente novo e devolve o `id`. Antes, confira com `pacientes_homonimos` (ou `buscar_pacientes`) se ele já existe.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `bairro` | não | Bairro. |
| `celular` | não | Celular com DDD, só números. |
| `cep` | não | CEP, só números. |
| `cidade` | não | Cidade. |
| `convenio_id` | não | Id do plano de saúde (o `id` de `listar_planos_de_saude`). |
| `cpf` | não | CPF, só números. |
| `email` | não | E-mail. |
| `endereco` | não | Endereço residencial (rua e número). |
| `estado` | não | UF, duas letras. |
| `estado_civil` | não | Estado civil. |
| `indicado_por` | não | Quem indicou o paciente. |
| `mae` | não | Nome da mãe. |
| `nascimento` | não | Data de nascimento, AAAA-MM-DD. |
| `nome` | sim | Nome completo. |
| `nome_social` | não | Nome social. |
| `numero_carteirinha` | não | Número da carteirinha do plano. |
| `observacoes` | não | Observações do cadastro. |
| `pai` | não | Nome do pai. |
| `profissao` | não | Profissão. |
| `rg` | não | RG. |
| `sexo` | não | Sexo: M ou F. |
| `telefone` | não | Telefone residencial com DDD. |

### `atualizar_paciente` (sobrescreve dados)

Atualiza o cadastro de um paciente. Só os campos informados mudam; o resto fica como está na MedX.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `bairro` | não | Bairro. |
| `celular` | não | Celular com DDD, só números. |
| `cep` | não | CEP, só números. |
| `cidade` | não | Cidade. |
| `convenio_id` | não | Id do plano de saúde (o `id` de `listar_planos_de_saude`). |
| `cpf` | não | CPF, só números. |
| `email` | não | E-mail. |
| `endereco` | não | Endereço residencial (rua e número). |
| `estado` | não | UF, duas letras. |
| `estado_civil` | não | Estado civil. |
| `indicado_por` | não | Quem indicou o paciente. |
| `mae` | não | Nome da mãe. |
| `nascimento` | não | Data de nascimento, AAAA-MM-DD. |
| `nome` | não | Nome completo. |
| `nome_social` | não | Nome social. |
| `numero_carteirinha` | não | Número da carteirinha do plano. |
| `observacoes` | não | Observações do cadastro. |
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |
| `pai` | não | Nome do pai. |
| `profissao` | não | Profissão. |
| `rg` | não | RG. |
| `sexo` | não | Sexo: M ou F. |
| `telefone` | não | Telefone residencial com DDD. |

## Escrita: agenda

### `criar_agendamento`

Cria um agendamento de um paciente com um profissional, com status agendado. Devolve `{"criado": true}`; a MedX pode recusar (horário ocupado, por exemplo), e aí a ferramenta devolve erro.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `descricao` | não | Texto do agendamento. Padrão: o nome do paciente. |
| `diagnostico_id` | não | Id do diagnóstico QP (o `id` de `listar_diagnosticos_qp`). |
| `fim` | sim | Fim, AAAA-MM-DDTHH:MM, hora local. |
| `inicio` | sim | Início, AAAA-MM-DDTHH:MM, hora local. |
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |
| `procedimento_id` | não | Id do procedimento (o `id` de `listar_procedimentos`). |
| `profissional_id` | sim | Id do profissional (o `id` de `listar_profissionais_agenda`). |

### `bloquear_horario`

Bloqueia um horário na agenda de um profissional (almoço, reunião, folga), sem paciente.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `fim` | sim | Fim, AAAA-MM-DDTHH:MM, hora local. |
| `inicio` | sim | Início, AAAA-MM-DDTHH:MM, hora local. |
| `profissional_id` | sim | Id do profissional (o `id` de `listar_profissionais_agenda`). |

### `mudar_status_agendamento` (sobrescreve dados)

Muda o status de um agendamento (desmarcado, agendado, compareceu e os outros rótulos de `parametros_agenda`).

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `agendamento_id` | sim | Id do agendamento (o `id` de `agenda_do_dia`). |
| `status` | sim | Novo status: a posição do rótulo em `parametros_agenda` (por exemplo 0 desmarcado, 1 agendado, 2 compareceu). |

### `remarcar_agendamento` (sobrescreve dados)

Muda o horário de um agendamento (e, se pedido, o profissional), mantendo paciente, descrição e o resto.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `agendamento_id` | sim | Id do agendamento (o `id` de `agenda_do_dia`). |
| `data` | sim | Dia atual do agendamento, AAAA-MM-DD. |
| `novo_fim` | sim | Novo fim, AAAA-MM-DDTHH:MM, hora local. |
| `novo_inicio` | sim | Novo início, AAAA-MM-DDTHH:MM, hora local. |
| `novo_profissional_id` | não | Outro profissional, se o agendamento muda de agenda. |
| `profissional_id` | sim | Profissional atual do agendamento. |

### `confirmar_agendamento_whatsapp` (fala com terceiros)

Manda ao paciente, pelo WhatsApp da clínica, o pedido de confirmação de um agendamento. A mensagem sai para o paciente na hora.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `agendamento_id` | sim | Id do agendamento (o `id` de `agenda_do_dia`). |

## Escrita: notas

### `criar_nota`

Cria uma nota no painel do usuário logado e devolve o `id`.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `texto` | sim | Texto da nota. |

### `editar_nota` (sobrescreve dados)

Edita uma nota do painel: troca o texto e/ou marca como concluída.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `concluida` | não | Marca a nota como concluída. Padrão: não. |
| `nota_id` | sim | Id da nota (o `id` de `listar_notas`). |
| `texto` | não | Novo texto. Ausente: mantém o atual. |

## Escrita: prontuário

### `registrar_no_prontuario`

Registra uma evolução no prontuário de um paciente, em nome do usuário logado. O texto puro vira o HTML que o prontuário guarda.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `data` | não | Data e hora do registro, AAAA-MM-DDTHH:MM, hora local. Padrão: agora. |
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |
| `palavras_chave` | não | Palavras-chave do registro, separadas por vírgula (as de `palavras_chave_prontuario`). |
| `texto` | sim | Texto do registro, em texto puro (linha em branco separa parágrafos). |

### `editar_registro_prontuario` (sobrescreve dados)

Substitui o texto de um registro do prontuário, mantendo data, autor e palavras-chave.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |
| `registro_id` | sim | Id do registro (o `id` de `ver_prontuario`). |
| `texto` | sim | Novo texto do registro, em texto puro. Substitui o anterior. |

### `atualizar_sumario_prontuario` (sobrescreve dados)

Atualiza o sumário fixo do paciente. Só os campos informados mudam.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `alergias` | não | Alergias. |
| `diagnostico` | não | Diagnóstico. |
| `hpp` | não | História patológica pregressa. |
| `livre` | não | Campo livre. |
| `medicamentos` | não | Medicamentos em uso. |
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |

### `anexar_ao_prontuario`

Anexa um arquivo do computador ao prontuário de um paciente (PDF, imagem, documento; até 22 MB). Recebe o caminho absoluto do arquivo, não o conteúdo.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `arquivo` | sim | Caminho absoluto do arquivo no computador onde o servidor roda. Até 22 MB. |
| `descricao` | sim | Descrição do anexo, como aparece no prontuário. |
| `paciente_id` | sim | Id do paciente (o `id` de `buscar_pacientes`). |

## Escrita: chat

### `enviar_mensagem_chat`

Manda uma mensagem no chat interno da equipe, em nome do usuário logado.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `texto` | sim | Texto da mensagem. |
| `usuario_id` | sim | Id do destinatário (o `id` de `listar_usuarios_chat`). |

### `marcar_mensagens_lidas`

Marca mensagens do chat interno como lidas.

| Parâmetro | Obrigatório | Descrição |
|---|---|---|
| `ids` | sim | Ids das mensagens (o `id` de `mensagens_recebidas_chat` ou de `historico_chat`). |
