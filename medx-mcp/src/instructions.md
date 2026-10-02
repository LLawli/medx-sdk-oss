Ferramentas da MedX, o sistema de gestão da clínica: agenda, pacientes,
prontuário, financeiro, notas, chat interno e configurações.

- Pacientes são "contatos" na MedX. Para achar um paciente pelo nome, use a
  busca de pacientes e depois as outras ferramentas com o id dele.
- As respostas são JSON com os nomes de campo do medx-sdk, em snake_case.
- Ferramentas que devolvem lista aceitam `limite` (padrão 50, até 500).
  Quando a lista passa do limite, a resposta avisa quantos itens existiam:
  refine a busca ou peça um limite maior.
- A MedX aceita uma sessão por conta. Quando este servidor faz login, a
  sessão aberta no navegador com a mesma conta cai, e vice-versa.
- Os dados são de pacientes reais. Mostre ao usuário só o que ele pediu.
- As ferramentas de escrita só existem quando o servidor foi iniciado com
  `MEDX_MCP_ALLOW_WRITE=1`. Exclusões e troca de senha não existem aqui.
