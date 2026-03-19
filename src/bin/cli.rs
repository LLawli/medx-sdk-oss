//! medx-cli — Interface de linha de comando para o MedX SDK.
//!
//! Uso básico:
//!   medx-cli help
//!   medx-cli <recurso> help
//!   medx-cli <recurso> <comando> [args...]

use std::env;

// ── Cores ANSI ────────────────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";

fn b(s: &str) -> String { format!("{BOLD}{s}{RESET}") }
fn c(s: &str) -> String { format!("{CYAN}{s}{RESET}") }
fn y(s: &str) -> String { format!("{YELLOW}{s}{RESET}") }
fn g(s: &str) -> String { format!("{GREEN}{s}{RESET}") }
fn dim(s: &str) -> String { format!("{DIM}{s}{RESET}") }

// ── Estruturas de ajuda ───────────────────────────────────────────────────────

struct Cmd {
    usage: &'static str,
    desc: &'static str,
    /// Linha extra de detalhes (opcional). Ex: lista de valores válidos.
    detail: Option<&'static str>,
}

struct Resource {
    name: &'static str,
    short: &'static str,   // descrição de uma linha
    about: &'static str,   // parágrafo de contexto
    commands: &'static [Cmd],
    examples: &'static [(&'static str, &'static str)],  // (comando, comentário)
}

// ── Catálogo de recursos ──────────────────────────────────────────────────────

const RESOURCES: &[Resource] = &[
    Resource {
        name: "auth",
        short: "Autenticação e sessão",
        about: "Gerencia o ciclo de autenticação com a plataforma MedX. O token \
                é salvo em ~/.config/medx-sdk/session.json e reutilizado \
                automaticamente pelos demais comandos.",
        commands: &[
            Cmd { usage: "login <email> <senha>",  desc: "Autentica e salva a sessão localmente", detail: None },
            Cmd { usage: "session",                 desc: "Exibe os dados da sessão ativa",         detail: None },
            Cmd { usage: "logout",                  desc: "Remove a sessão salva do disco",          detail: None },
        ],
        examples: &[
            ("auth login medico@clinica.com.br Senha@123", "faz login e salva token"),
            ("auth session",                               "mostra token e db_id ativos"),
            ("auth logout",                                "encerra a sessão local"),
        ],
    },

    Resource {
        name: "contacts",
        short: "Contatos e pacientes",
        about: "Busca, cria e atualiza contatos. Contatos são a entidade central do MedX — \
                todas as outras operações (agenda, prontuário, finanças) referenciam um contato.",
        commands: &[
            Cmd { usage: "search <termo> [limite]",  desc: "Busca contatos por nome ou documento",
                  detail: Some("limite padrão: 10") },
            Cmd { usage: "get <id>",                 desc: "Exibe detalhes de um contato pelo ID", detail: None },
            Cmd { usage: "homonym <nome> <sexo> <nascimento>",
                                                     desc: "Verifica duplicatas por nome, sexo e data de nascimento",
                  detail: Some("sexo: M ou F | nascimento: YYYY-MM-DD") },
            Cmd { usage: "insurance-plans",          desc: "Lista planos de saúde disponíveis", detail: None },
        ],
        examples: &[
            ("contacts search \"João Silva\"",               "busca contatos com nome João Silva"),
            ("contacts search joao 5",                       "retorna no máximo 5 resultados"),
            ("contacts get 42",                              "exibe dados completos do contato #42"),
            ("contacts homonym \"João Silva\" M 1990-05-20", "verifica duplicatas antes de cadastrar"),
            ("contacts insurance-plans",                     "lista convênios e planos disponíveis"),
        ],
    },

    Resource {
        name: "agenda",
        short: "Agenda e agendamentos",
        about: "Consulta a agenda diária, cria e atualiza agendamentos. \
                Use os parâmetros da agenda para obter as cores e rótulos de status.",
        commands: &[
            Cmd { usage: "daily <user_id> <data>",    desc: "Agenda do dia de um profissional",
                  detail: Some("data no formato YYYY-MM-DD") },
            Cmd { usage: "params",                     desc: "Parâmetros, setores, cores e rótulos de status", detail: None },
            Cmd { usage: "users",                      desc: "Profissionais com agenda ativa", detail: None },
            Cmd { usage: "create <patient_id> <user_id> <inicio> <fim>",
                                                       desc: "Cria um agendamento",
                  detail: Some("datas no formato YYYY-MM-DDTHH:MM:SS") },
            Cmd { usage: "status <appointment_id> <status_id>",
                                                       desc: "Atualiza o status de um agendamento", detail: None },
            Cmd { usage: "delete <appointment_id>",    desc: "Cancela/remove um agendamento", detail: None },
            Cmd { usage: "report <inicio> <fim> [user_id]",
                                                       desc: "Relatório de agenda por período",
                  detail: Some("datas YYYY-MM-DD; user_id 0 = todos os profissionais") },
            Cmd { usage: "no-show <inicio> <fim> [user_id]",
                                                       desc: "Relatório de no-show por período",
                  detail: Some("datas YYYY-MM-DD; user_id 0 = todos os profissionais") },
            Cmd { usage: "block-create <user_id> <inicio> <fim>",
                                                       desc: "Cria um bloqueio de agenda para um profissional",
                  detail: Some("datas no formato YYYY-MM-DDTHH:MM:SS") },
            Cmd { usage: "block-list <user_id> <data>",
                                                       desc: "Lista bloqueios de um profissional em uma data",
                  detail: Some("data no formato YYYY-MM-DD") },
            Cmd { usage: "block-remove <id>",
                                                       desc: "Remove um bloqueio (soft-delete: marca como desmarcado)",
                  detail: None },
        ],
        examples: &[
            ("agenda daily 3 2026-03-17",              "agenda do profissional #3 hoje"),
            ("agenda params",                           "lista setores, cores e rótulos de status"),
            ("agenda users",                            "lista profissionais com agenda"),
            ("agenda status 1001 2",                    "marca agendamento #1001 como status 2"),
            ("agenda report 2026-03-01 2026-03-31",     "relatório de agenda de março"),
            ("agenda no-show 2026-03-01 2026-03-31 5",  "no-show do profissional #5 em março"),
            ("agenda block-create 3 2026-04-01T12:00:00 2026-04-01T13:00:00", "bloqueia horário do profissional #3"),
            ("agenda block-list 3 2026-04-01",         "lista bloqueios do profissional #3 nessa data"),
            ("agenda block-remove 1001",               "remove o bloqueio #1001"),
        ],
    },

    Resource {
        name: "prontuario",
        short: "Prontuário médico",
        about: "Acessa e registra prontuários, sumários de histórico médico, \
                convênios, procedimentos e formulários clínicos.",
        commands: &[
            Cmd { usage: "summary <patient_id>",      desc: "Sumário do histórico médico do paciente", detail: None },
            Cmd { usage: "records <patient_id>",      desc: "Registros de prontuário do paciente",      detail: None },
            Cmd { usage: "files <patient_id>",        desc: "Arquivos anexados ao prontuário (PDFs, imagens) com URL",
                  detail: Some("mostra apenas registros que têm arquivo anexado") },
            Cmd { usage: "search <patient_id> <q>",   desc: "Busca nos prontuários do paciente",        detail: None },
            Cmd { usage: "convenios",                  desc: "Lista os convênios disponíveis",           detail: None },
            Cmd { usage: "procedures",                 desc: "Lista todos os procedimentos",             detail: None },
            Cmd { usage: "forms",                      desc: "Lista formulários da clínica",             detail: None },
            Cmd { usage: "keywords",                   desc: "Palavras-chave do prontuário",             detail: None },
            Cmd { usage: "units",                      desc: "Unidades de negócio (UNs)",                detail: None },
            Cmd { usage: "modules <patient_id> <modulo>",
                                                       desc: "Registros de um módulo customizado do paciente", detail: None },
        ],
        examples: &[
            ("prontuario summary 42",              "histórico médico do paciente #42"),
            ("prontuario records 42",              "lista registros de prontuário do paciente #42"),
            ("prontuario files 42",                "lista PDFs e imagens com URL do paciente #42"),
            ("prontuario search 42 diabetes",      "busca 'diabetes' no prontuário do paciente #42"),
            ("prontuario convenios",               "lista convênios cadastrados"),
            ("prontuario modules 42 anamnese",     "registros do módulo 'anamnese' do paciente #42"),
        ],
    },

    Resource {
        name: "financas",
        short: "Finanças e atendimentos",
        about: "Consulta atendimentos, valores de fatura e pagamentos. \
                Cada atendimento representa uma visita do paciente com dados financeiros.",
        commands: &[
            Cmd { usage: "by-patient <patient_id>",        desc: "Atendimentos de um paciente",                  detail: None },
            Cmd { usage: "all [filtro] [tipo]",            desc: "Todos os atendimentos com filtro opcional",
                  detail: Some("filtro: string de busca | tipo: período ou status") },
            Cmd { usage: "pre-payment <patient_id> <valor>", desc: "Gera link de pagamento Stone/Pagar.me",
                  detail: Some("valor em reais; ex: 150.00") },
        ],
        examples: &[
            ("financas by-patient 42",    "atendimentos do paciente #42"),
            ("financas all",              "todos os atendimentos sem filtro"),
            ("financas all \"março\" mes", "atendimentos filtrados por 'março'"),
            ("financas pre-payment 42 150.00", "gera link de pagamento para o paciente #42"),
        ],
    },

    Resource {
        name: "chat",
        short: "Chat interno entre profissionais",
        about: "Mensagens entre profissionais da clínica. \
                O chat usa persistência via API REST; a entrega em tempo real \
                depende do Agora RTM integrado na interface web.",
        commands: &[
            Cmd { usage: "users",                          desc: "Lista usuários disponíveis no chat (com contagem de não-lidas)", detail: None },
            Cmd { usage: "history <user_id>",              desc: "Histórico de mensagens com um usuário",                          detail: None },
            Cmd { usage: "unread",                         desc: "Total de mensagens não-lidas",                                   detail: None },
            Cmd { usage: "incoming",                       desc: "Mensagens recebidas ainda não-lidas",                            detail: None },
            Cmd { usage: "send <to_id> <mensagem>",        desc: "Envia uma mensagem a um usuário",                               detail: None },
        ],
        examples: &[
            ("chat users",              "lista profissionais e quantidade de mensagens não-lidas"),
            ("chat history 5",          "conversa com o usuário #5"),
            ("chat unread",             "quantidade total de mensagens não-lidas"),
            ("chat send 5 \"Boa tarde\"", "envia mensagem para o usuário #5"),
        ],
    },

    Resource {
        name: "notif",
        short: "Notificações e comunicação",
        about: "Configurações de mensagens da clínica e registro de e-mails enviados.",
        commands: &[
            Cmd { usage: "settings",                          desc: "Configurações de notificação da clínica (email, SMS, WhatsApp)", detail: None },
            Cmd { usage: "log-email <patient_id> <para> <assunto>",
                                                               desc: "Registra um e-mail enviado no log",
                  detail: Some("corpo do e-mail lido de stdin") },
        ],
        examples: &[
            ("notif settings",                              "exibe configurações de e-mail e WhatsApp"),
            ("notif log-email 42 paciente@email.com \"Consulta\"", "registra envio de e-mail"),
        ],
    },

    Resource {
        name: "marketing",
        short: "Marketing e eventos",
        about: "Eventos da clínica, questionários de satisfação e diagnósticos QP.",
        commands: &[
            Cmd { usage: "events",                desc: "Lista todos os eventos",                    detail: None },
            Cmd { usage: "quests",                desc: "Lista questionários disponíveis",           detail: None },
            Cmd { usage: "diagnostico-qp",        desc: "Lista diagnósticos QP",                    detail: None },
        ],
        examples: &[
            ("marketing events",          "lista eventos cadastrados"),
            ("marketing quests",          "lista questionários de satisfação"),
            ("marketing diagnostico-qp",  "lista diagnósticos QP"),
        ],
    },

    Resource {
        name: "hoje",
        short: "Dashboard do dia",
        about: "Visão geral do dia: notificações, últimos atendidos, notas internas \
                e informações da conta.",
        commands: &[
            Cmd { usage: "notificacoes",                       desc: "Notificações do dia",                   detail: None },
            Cmd { usage: "ultimos-atendidos",                  desc: "Últimos pacientes atendidos",           detail: None },
            Cmd { usage: "trial",                              desc: "Informações de trial da conta",         detail: None },
            Cmd { usage: "notas",                              desc: "Lista notas internas",                  detail: None },
            Cmd { usage: "nota-add <user_id> <texto>",         desc: "Cria uma nova nota interna",            detail: None },
            Cmd { usage: "nota-del <nota_id>",                 desc: "Remove uma nota pelo ID",               detail: None },
        ],
        examples: &[
            ("hoje notificacoes",           "notificações pendentes do dia"),
            ("hoje ultimos-atendidos",      "últimos pacientes atendidos"),
            ("hoje notas",                  "lista todas as notas internas"),
            ("hoje nota-add 2 \"Reunião\"", "cria nota para o usuário #2"),
            ("hoje nota-del 7",             "remove a nota #7"),
        ],
    },

    Resource {
        name: "ajustes",
        short: "Ajustes e administração",
        about: "Relatórios disponíveis, pastas de documentos, configuração ICS \
                e troca de senha.",
        commands: &[
            Cmd { usage: "reports",                        desc: "Lista relatórios disponíveis",         detail: None },
            Cmd { usage: "docs [filtro]",                  desc: "Pastas de documentos (autodocs)",      detail: None },
            Cmd { usage: "ics",                            desc: "Configuração do localizador ICS",      detail: None },
            Cmd { usage: "change-password <nova-senha>",   desc: "Altera a senha do usuário autenticado (sem confirmar senha atual)",
                  detail: Some("use change-password-full para confirmar a senha atual também") },
            Cmd { usage: "change-password-full <atual> <nova>",
                                                           desc: "Altera a senha confirmando a senha atual via RSA",
                  detail: None },
        ],
        examples: &[
            ("ajustes reports",                        "lista relatórios disponíveis"),
            ("ajustes docs",                           "todas as pastas de documentos"),
            ("ajustes docs paciente",                  "pastas filtradas por 'paciente'"),
            ("ajustes ics",                            "URL e token do calendário ICS"),
            ("ajustes change-password NovaSenha@2026", "troca a senha do usuário logado"),
        ],
    },
];

// ── Renderização de ajuda ─────────────────────────────────────────────────────

fn hr(width: usize) { println!("{}", dim(&"─".repeat(width))); }

fn print_global_help() {
    println!();
    println!("{} {}", b(&c("medx-cli")), dim("v0.1"));
    println!("{}", dim("CLI para a plataforma MedX — gerencia sessão, agenda, prontuário e mais."));
    println!();
    println!("{}", b("USO"));
    println!("  {} {} {}",
        c("medx-cli"),
        y("[RECURSO]"),
        dim("[COMANDO] [ARGS...]"));
    println!("  {} {} {}",
        c("medx-cli"),
        y("help"),
        dim("[RECURSO]"));
    println!();
    println!("{}", b("RECURSOS"));
    hr(56);
    for r in RESOURCES {
        println!("  {:<16} {}", g(r.name), r.short);
    }
    hr(56);
    println!();
    println!("{}", b("COMANDOS GLOBAIS"));
    println!("  {:<30} {}", g("help") + &dim(" [RECURSO]"),      "Ajuda geral ou por recurso");
    println!("  {:<30} {}", g("version"),                         "Exibe a versão");
    println!();
    println!("{}", b("EXEMPLOS"));
    println!("  {}  {}", c("medx-cli auth login medico@clinica.com Senha@1"), dim("# autentica"));
    println!("  {}    {}", c("medx-cli agenda daily 3 2026-03-17"),           dim("# agenda do dia"));
    println!("  {}             {}", c("medx-cli help agenda"),                dim("# ajuda da agenda"));
    println!();
    println!("{}", dim("Para ajuda de um recurso específico: medx-cli <recurso> help"));
    println!();
}

fn print_resource_help(res: &Resource) {
    println!();
    println!("{} — {}", b(&g(res.name)), res.short);
    hr(60);
    println!("{}", dim(res.about));
    println!();
    println!("{}", b("COMANDOS"));
    for cmd in res.commands {
        println!("  {} {}", c("medx-cli"), y(&format!("{} {}", res.name, cmd.usage)));
        println!("      {}", cmd.desc);
        if let Some(detail) = cmd.detail {
            println!("      {}", dim(detail));
        }
    }
    println!();
    println!("{}", b("EXEMPLOS"));
    let max_w = res.examples.iter().map(|(e, _)| e.len()).max().unwrap_or(0) + 2;
    for (example, comment) in res.examples {
        println!("  {:<width$}  {}", c(&format!("medx-cli {example}")), dim(&format!("# {comment}")), width = max_w + 9);
    }
    println!();
    println!("{}", dim(&format!("Ajuda geral: medx-cli help")));
    println!();
}

fn find_resource(name: &str) -> Option<&'static Resource> {
    RESOURCES.iter().find(|r| r.name == name)
}

fn print_unknown_resource(name: &str) {
    eprintln!();
    eprintln!("{}  recurso {} não reconhecido.", y("aviso:"), b(&format!("'{name}'")));
    eprintln!();
    eprintln!("Recursos disponíveis: {}",
        RESOURCES.iter().map(|r| g(r.name)).collect::<Vec<_>>().join(", "));
    eprintln!("Use {} para ver todos os recursos.", c("medx-cli help"));
    eprintln!();
}

fn show_version() {
    println!("medx-cli {}", env!("CARGO_PKG_VERSION"));
}

// ── Helpers de formatação de erro ─────────────────────────────────────────────

fn err_prefix() -> String { format!("{}{}{}", RED, "erro:", RESET) }

fn usage_err(usage: &str) -> String {
    format!("{} uso: {} {}",
        err_prefix(),
        c("medx-cli"),
        y(usage))
}

// ── Dispatch de comandos ──────────────────────────────────────────────────────

fn dispatch_auth(cmd: Option<&str>, args: &[String]) {
    match cmd {
        Some("login") => {
            let email = match args.get(0) {
                Some(e) => e,
                None => { eprintln!("{}", usage_err("auth login <email> <senha>")); std::process::exit(1); }
            };
            let password = match args.get(1) {
                Some(p) => p,
                None => { eprintln!("{}", usage_err("auth login <email> <senha>")); std::process::exit(1); }
            };
            match medx::auth::login(email, password) {
                Ok(session) => {
                    println!("\n{}", b("Sessão salva:"));
                    println!("  {} {}", dim("email :"), session.email);
                    println!("  {} {}", dim("db_id :"), session.db_id);
                    println!("  {} {}{}",
                        dim("token :"),
                        &session.token[..session.token.len().min(20)],
                        dim("..."));
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("session") => match medx::load_session() {
            Some(s) => {
                println!("\n{}", b("Sessão ativa:"));
                println!("  {} {}", dim("email :"), s.email);
                println!("  {} {}", dim("db_id :"), s.db_id);
                println!("  {} {}{}",
                    dim("token :"),
                    &s.token[..s.token.len().min(20)],
                    dim("..."));
                println!();
            }
            None => println!("Nenhuma sessão salva."),
        },
        Some("logout") => {
            medx::clear_session().expect("Falha ao remover sessão");
            println!("Sessão removida.");
        }
        _ => {
            if let Some(r) = find_resource("auth") { print_resource_help(r); }
        }
    }
}

fn require_client() -> medx::MedxClient {
    let session = match medx::load_session() {
        Some(s) => s,
        None => {
            eprintln!("{} nenhuma sessão ativa. Use: {}",
                err_prefix(),
                c("medx-cli auth login <email> <senha>"));
            std::process::exit(1);
        }
    };
    // Se credenciais disponíveis via env, habilita retry automático em 401
    if let (Ok(email), Ok(pass)) = (
        std::env::var("MEDX_LOGIN_CREDENTIAL"),
        std::env::var("MEDX_PASSWORD_CREDENTIAL"),
    ) {
        return medx::MedxClient::from_session_with_credentials(session, email, pass);
    }
    medx::MedxClient::from_session(session)
}

fn print_kv(label: &str, value: &str) {
    println!("  {:<20} {}", dim(&format!("{label}:")), value);
}

fn print_sep() {
    println!("{}", dim(&"─".repeat(52)));
}

fn ok(msg: &str) {
    println!("{} {msg}", g("✓"));
}

fn dispatch_contacts(cmd: Option<&str>, args: &[String]) {
    let c = require_client();
    match cmd {
        Some("search") => {
            let query = args.get(0).map(String::as_str).unwrap_or("A");
            let limit: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(10);
            match c.search_contacts(query, medx::ContactSearchGroup::All, limit) {
                Ok(contacts) if contacts.is_empty() => println!("{}", dim("(nenhum resultado)")),
                Ok(contacts) => {
                    for ct in &contacts {
                        print_sep();
                        print_kv("id",     &ct.id.to_string());
                        print_kv("nome",   &ct.name);
                        print_kv("fone",   &ct.mobile);
                        print_kv("email",  &ct.email);
                        print_kv("cpf",    &ct.cpf);
                    }
                    print_sep();
                    println!("{} {} contato(s)", dim("total:"), contacts.len());
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("get") => {
            let id: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("contacts get <id>")); std::process::exit(1); }
            };
            match c.contact(id) {
                Ok(ct) => {
                    print_kv("id",              &ct.id.to_string());
                    print_kv("nome",            &ct.name);
                    print_kv("nome social",     &ct.social_name);
                    print_kv("email",           &ct.email);
                    print_kv("celular",         &ct.mobile);
                    print_kv("telefone",        &ct.phone_home);
                    print_kv("cpf",             &ct.cpf);
                    print_kv("nascimento",      ct.birth_date.as_deref().unwrap_or(""));
                    print_kv("cidade",          &ct.city_home);
                    print_kv("estado",          &ct.state_home);
                    print_kv("convênio id",     &ct.insurance_id.to_string());
                    print_kv("observações",     &ct.notes);
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("homonym") => {
            let name = match args.get(0) {
                Some(n) => n.as_str(),
                None => { eprintln!("{}", usage_err("contacts homonym <nome> <sexo> <nascimento>")); std::process::exit(1); }
            };
            let gender = match args.get(1) {
                Some(g) => g.as_str(),
                None => { eprintln!("{}", usage_err("contacts homonym <nome> <sexo> <nascimento>")); std::process::exit(1); }
            };
            let birth = match args.get(2) {
                Some(b) => b.as_str(),
                None => { eprintln!("{}", usage_err("contacts homonym <nome> <sexo> <nascimento>")); std::process::exit(1); }
            };
            match c.homonym_contacts(name, gender, birth) {
                Ok(hs) if hs.is_empty() => println!("{}", dim("(nenhum homônimo encontrado)")),
                Ok(hs) => {
                    for h in &hs {
                        print_sep();
                        print_kv("id",          &h.id.to_string());
                        print_kv("nome",        &h.name);
                        print_kv("sexo",        &h.gender);
                        print_kv("nascimento",  h.birth_date.as_deref().unwrap_or(""));
                    }
                    print_sep();
                    println!("{} {} homônimo(s)", dim("total:"), hs.len());
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("insurance-plans") => {
            match c.insurance_plans() {
                Ok(plans) if plans.is_empty() => println!("{}", dim("(nenhum plano cadastrado)")),
                Ok(plans) => {
                    for p in &plans {
                        println!("  #{:<4} {}", p.id, p.name);
                    }
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        _ => { if let Some(r) = find_resource("contacts") { print_resource_help(r); } }
    }
}

fn dispatch_agenda(cmd: Option<&str>, args: &[String]) {
    let c = require_client();
    match cmd {
        Some("daily") => {
            let user_id: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("agenda daily <user_id> <data>")); std::process::exit(1); }
            };
            let date = match args.get(1) {
                Some(d) => d.as_str(),
                None => { eprintln!("{}", usage_err("agenda daily <user_id> <data>")); std::process::exit(1); }
            };
            match c.daily_agenda(user_id, date) {
                Ok(appts) if appts.is_empty() => println!("{}", dim("(sem agendamentos)")),
                Ok(appts) => {
                    for a in &appts {
                        print_sep();
                        print_kv("id",          &a.id.to_string());
                        print_kv("início",      &a.start);
                        print_kv("fim",         &a.end);
                        print_kv("descrição",   &a.description);
                        print_kv("status",      &a.status.to_string());
                        if let Some(cid) = a.contact_id { print_kv("paciente id", &cid.to_string()); }
                    }
                    print_sep();
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("params") => {
            match c.agenda_parameters() {
                Ok(p) => {
                    println!("{}", b("Parâmetros gerais:"));
                    print_kv("slot",       &p.general.slot_duration);
                    print_kv("início",     &p.general.start_time);
                    print_kv("fim",        &p.general.end_time);
                    println!("\n{}", b("Setores:"));
                    for s in &p.sectors { println!("  #{:<4} {}", s.id, s.name); }
                    println!("\n{}", b("Status (id → label → cor):"));
                    for i in 0..p.color_labels.labels.len() {
                        let label = p.color_labels.label_for(i as i64).unwrap_or("");
                        let color = p.color_labels.color_for(i as i64).unwrap_or("");
                        println!("  {:<3} {:<20} {}", i, label, dim(color));
                    }
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("users") => {
            match c.agenda_users() {
                Ok(users) if users.is_empty() => println!("{}", dim("(nenhum profissional)")),
                Ok(users) => {
                    for u in &users {
                        println!("  #{:<4} {:<30} {}", u.id, u.username, dim(&format!("setor {}", u.sector_id)));
                    }
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("status") => {
            let appt_id: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("agenda status <appointment_id> <status_id>")); std::process::exit(1); }
            };
            let status_id: i64 = match args.get(1).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("agenda status <appointment_id> <status_id>")); std::process::exit(1); }
            };
            match c.update_appointment_status(appt_id, status_id) {
                Ok(_) => ok(&format!("status do agendamento #{appt_id} atualizado para {status_id}")),
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("delete") => {
            let id: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("agenda delete <appointment_id>")); std::process::exit(1); }
            };
            match c.delete_appointment(id) {
                Ok(_) => ok(&format!("agendamento #{id} removido")),
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("report") => {
            let start = match args.get(0) {
                Some(d) => d.as_str(),
                None => { eprintln!("{}", usage_err("agenda report <inicio> <fim> [user_id]")); std::process::exit(1); }
            };
            let end = match args.get(1) {
                Some(d) => d.as_str(),
                None => { eprintln!("{}", usage_err("agenda report <inicio> <fim> [user_id]")); std::process::exit(1); }
            };
            let user_id: i64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            let dto = medx::AgendaReportDto::new(start, end, user_id);
            match c.agenda_report(&dto) {
                Ok(r) if r.file_url.is_empty() => println!("{}", dim("(sem dados no período)")),
                Ok(r) => println!("{CYAN}{}{RESET}", r.file_url),
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("no-show") => {
            let start = match args.get(0) {
                Some(d) => d.as_str(),
                None => { eprintln!("{}", usage_err("agenda no-show <inicio> <fim> [user_id]")); std::process::exit(1); }
            };
            let end = match args.get(1) {
                Some(d) => d.as_str(),
                None => { eprintln!("{}", usage_err("agenda no-show <inicio> <fim> [user_id]")); std::process::exit(1); }
            };
            let user_id: i64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            let dto = medx::NoShowReportDto::new(start, end, user_id);
            match c.no_show_report(&dto) {
                Ok(r) if r.file_url.is_empty() => println!("{}", dim("(sem dados no período)")),
                Ok(r) => println!("{CYAN}{}{RESET}", r.file_url),
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("block-create") => {
            let user_id: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("agenda block-create <user_id> <inicio> <fim>")); std::process::exit(1); }
            };
            let start = match args.get(1) {
                Some(d) => d.as_str(),
                None => { eprintln!("{}", usage_err("agenda block-create <user_id> <inicio> <fim>")); std::process::exit(1); }
            };
            let end = match args.get(2) {
                Some(d) => d.as_str(),
                None => { eprintln!("{}", usage_err("agenda block-create <user_id> <inicio> <fim>")); std::process::exit(1); }
            };
            match c.create_agenda_block(user_id, start, end) {
                Ok(true)  => ok(&format!("bloqueio criado para o profissional #{user_id}")),
                Ok(false) => eprintln!("{} bloqueio não foi confirmado pela API", err_prefix()),
                Err(e)    => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("block-list") => {
            let user_id: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("agenda block-list <user_id> <data>")); std::process::exit(1); }
            };
            let date = match args.get(1) {
                Some(d) => d.as_str(),
                None => { eprintln!("{}", usage_err("agenda block-list <user_id> <data>")); std::process::exit(1); }
            };
            match c.daily_blocks(user_id, date) {
                Ok(bs) if bs.is_empty() => println!("{}", dim("(nenhum bloqueio)")),
                Ok(bs) => {
                    for b in &bs {
                        print_sep();
                        print_kv("id",     &b.id.to_string());
                        print_kv("início", &b.start);
                        print_kv("fim",    &b.end);
                        if !b.description.is_empty() { print_kv("descrição", &b.description); }
                    }
                    print_sep();
                    println!("{} {} bloqueio(s)", dim("total:"), bs.len());
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("block-remove") => {
            let id: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("agenda block-remove <id>")); std::process::exit(1); }
            };
            match c.remove_agenda_block(id) {
                Ok(_) => ok(&format!("bloqueio #{id} removido")),
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        _ => { if let Some(r) = find_resource("agenda") { print_resource_help(r); } }
    }
}

fn dispatch_prontuario(cmd: Option<&str>, args: &[String]) {
    let c = require_client();
    match cmd {
        Some("summary") => {
            let pid: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("prontuario summary <patient_id>")); std::process::exit(1); }
            };
            match c.medical_history_summary(pid) {
                Ok(s) => {
                    print_kv("diagnóstico",  &s.diagnostic);
                    print_kv("hpp",          &s.hpp);
                    print_kv("medicamentos", &s.medications);
                    print_kv("alergias",     &s.allergies);
                    print_kv("texto livre",  &s.free_text);
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("records") => {
            let pid: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("prontuario records <patient_id>")); std::process::exit(1); }
            };
            match c.medical_records(pid) {
                Ok(rs) if rs.is_empty() => println!("{}", dim("(nenhum registro)")),
                Ok(rs) => {
                    for r in &rs {
                        print_sep();
                        print_kv("id",       &r.id.to_string());
                        print_kv("data",     &r.date);
                        print_kv("autor",    &r.usuario);
                        if !r.keywords.is_empty() { print_kv("keywords", &r.keywords); }
                        if r.has_file() {
                            print_kv("tipo",  &r.tipo_doc);
                            match c.resolve_file_url(&r.classe) {
                                Ok(url) => println!("  {CYAN}{url}{RESET}"),
                                Err(e)  => println!("  {DIM}(url indisponível: {e}){RESET}"),
                            }
                        } else {
                            // conteúdo pode ser HTML longo — truncar em 120 chars
                            let preview: String = r.content.chars()
                                .filter(|ch| ch.is_alphanumeric() || ch.is_whitespace() || "+-.,:".contains(*ch))
                                .take(120).collect();
                            print_kv("conteúdo", &format!("{}…", preview.trim()));
                        }
                    }
                    print_sep();
                    println!("{} {} registro(s)", dim("total:"), rs.len());
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("files") => {
            let pid: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("prontuario files <patient_id>")); std::process::exit(1); }
            };
            match c.medical_records(pid) {
                Ok(rs) => {
                    let files: Vec<_> = rs.iter().filter(|r| r.has_file()).collect();
                    if files.is_empty() {
                        println!("{}", dim("(nenhum arquivo anexado)"));
                    } else {
                        for r in &files {
                            print_sep();
                            print_kv("id",        &r.id.to_string());
                            print_kv("data",      &r.date);
                            if !r.last_edit_date.is_empty() { print_kv("editado", &r.last_edit_date); }
                            print_kv("autor",     &r.usuario);
                            print_kv("tipo",      &r.tipo_doc);
                            if !r.keywords.is_empty() { print_kv("keywords", &r.keywords); }
                            if !r.content.is_empty() {
                                let preview: String = r.content.chars()
                                    .filter(|ch| ch.is_alphanumeric() || ch.is_whitespace() || "+-.,:".contains(*ch))
                                    .take(80).collect();
                                print_kv("descrição", preview.trim());
                            }
                            print_kv("blob",      &r.classe);
                            match c.resolve_file_url(&r.classe) {
                                Ok(url) => println!("  {CYAN}{url}{RESET}"),
                                Err(e)  => println!("  {DIM}(url indisponível: {e}){RESET}"),
                            }
                        }
                        print_sep();
                        println!("{} {} arquivo(s)", dim("total:"), files.len());
                    }
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("search") => {
            let pid: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("prontuario search <patient_id> <query>")); std::process::exit(1); }
            };
            let query = args.get(1).map(String::as_str).unwrap_or("");
            match c.search_medical_records(pid, query) {
                Ok(rs) if rs.is_empty() => println!("{}", dim("(nenhum resultado)")),
                Ok(rs) => { for r in &rs { println!("  #{} {} — {}", r.id, dim(&r.date), r.keywords); } }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("convenios") => {
            match c.convenios() {
                Ok(vs) => { for v in &vs { println!("  #{:<4} {}", v.id, v.name); } }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("procedures") => {
            match c.procedures() {
                Ok(ps) => { for p in &ps { println!("  #{:<4} {:<40} R$ {:.2}", p.id, p.name, p.base_price); } }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("forms") => {
            match c.forms() {
                Ok(fs) => { for f in &fs { println!("  #{:<4} {}", f.id, f.name); } }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("keywords") => {
            match c.medical_keywords() {
                Ok(kw) => {
                    let list = kw.as_list();
                    if list.is_empty() { println!("{}", dim("(nenhuma palavra-chave)")); }
                    else { for k in list { println!("  • {}", k.trim()); } }
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("units") => {
            match c.business_units() {
                Ok(us) => { for u in &us { println!("  #{:<4} {:<30} {}/{}", u.id, u.name, u.city, u.state); } }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("modules") => {
            let pid: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("prontuario modules <patient_id> <modulo>")); std::process::exit(1); }
            };
            let module = match args.get(1) {
                Some(m) => m.as_str(),
                None => { eprintln!("{}", usage_err("prontuario modules <patient_id> <modulo>")); std::process::exit(1); }
            };
            match c.module_records(pid, module) {
                Ok(rs) if rs.is_empty() => println!("{}", dim("(nenhum registro)")),
                Ok(rs) => {
                    for r in &rs {
                        print_sep();
                        print_kv("id",      &r.id.to_string());
                        print_kv("data",    &r.date);
                        if !r.data.is_empty() {
                            let preview: String = r.data.chars()
                                .filter(|ch| ch.is_alphanumeric() || ch.is_whitespace() || "+-.,:".contains(*ch))
                                .take(120).collect();
                            print_kv("dados", preview.trim());
                        }
                    }
                    print_sep();
                    println!("{} {} registro(s)", dim("total:"), rs.len());
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        _ => { if let Some(r) = find_resource("prontuario") { print_resource_help(r); } }
    }
}

fn dispatch_financas(cmd: Option<&str>, args: &[String]) {
    let c = require_client();
    match cmd {
        Some("by-patient") => {
            let pid: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("financas by-patient <patient_id>")); std::process::exit(1); }
            };
            match c.attendances_by_patient(pid) {
                Ok(ats) if ats.is_empty() => println!("{}", dim("(nenhum atendimento)")),
                Ok(ats) => {
                    for a in &ats {
                        print_sep();
                        print_kv("id",       &a.id);
                        print_kv("data",     &a.date);
                        print_kv("paciente", &a.patient_name);
                        print_kv("fatura",   &format!("R$ {:.2}", a.invoice_value));
                        print_kv("pago",     &format!("R$ {:.2}", a.total_paid));
                        print_kv("saldo",    &format!("R$ {:.2}", a.balance_due()));
                        print_kv("fechado",  if a.is_closed() { "sim" } else { "não" });
                    }
                    print_sep();
                    println!("{} {} atendimento(s)", dim("total:"), ats.len());
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("all") => {
            let filter = args.get(0).map(String::as_str).unwrap_or("");
            let tipo   = args.get(1).map(String::as_str).unwrap_or("");
            match c.all_attendances(filter, tipo) {
                Ok(ats) if ats.is_empty() => println!("{}", dim("(nenhum atendimento)")),
                Ok(ats) => {
                    for a in &ats {
                        println!("  {} {} {:<30} R$ {:.2}", dim(&a.id), dim(&a.date), a.patient_name, a.invoice_value);
                    }
                    println!("{} {} atendimento(s)", dim("total:"), ats.len());
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("pre-payment") => {
            let patient_id: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("financas pre-payment <patient_id> <valor>")); std::process::exit(1); }
            };
            let value: f64 = match args.get(1).and_then(|s| s.parse().ok()) {
                Some(v) => v,
                None => { eprintln!("{}", usage_err("financas pre-payment <patient_id> <valor>")); std::process::exit(1); }
            };
            let dto = medx::PrePaymentDto::new(patient_id.to_string(), value, "", "", "");
            match c.create_pre_payment(&dto) {
                Ok(Some(url)) => println!("{CYAN}{url}{RESET}"),
                Ok(None) => println!("{}", dim("(sem link gerado)")),
                Err(e) => println!("{}", dim(&format!("(gateway de pagamento indisponível: {e})"))),
            }
        }
        _ => { if let Some(r) = find_resource("financas") { print_resource_help(r); } }
    }
}

fn dispatch_chat(cmd: Option<&str>, args: &[String]) {
    let client = require_client();
    match cmd {
        Some("users") => {
            match client.chat_users() {
                Ok(us) if us.is_empty() => println!("{}", dim("(nenhum usuário)")),
                Ok(us) => {
                    for u in &us {
                        let online  = if u.online { g("●") } else { dim("○").to_string() };
                        let unread  = if u.unread > 0 { y(&format!(" [{}]", u.unread)) } else { String::new() };
                        println!("  {} #{:<4} {:<30}{}", online, u.id, u.full_name, unread);
                    }
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("history") => {
            let uid: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("chat history <user_id>")); std::process::exit(1); }
            };
            match client.chat_history(uid) {
                Ok(msgs) if msgs.is_empty() => println!("{}", dim("(sem mensagens)")),
                Ok(msgs) => {
                    for m in &msgs {
                        let read = if m.is_read() { dim("✓") } else { y("●").to_string() };
                        println!("  {} {} {}: {}", read, dim(&m.date), c(&m.from_name), m.text);
                    }
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("unread") => {
            match client.chat_unread_count() {
                Ok(n) => println!("{} mensagem(ns) não-lida(s)", b(&n.to_string())),
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("incoming") => {
            match client.chat_incoming() {
                Ok(msgs) if msgs.is_empty() => println!("{}", dim("(sem mensagens não-lidas)")),
                Ok(msgs) => {
                    for m in &msgs {
                        println!("  {} {}: {}", dim(&m.date), c(&m.from_name), m.text);
                    }
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("send") => {
            let to_id: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("chat send <to_id> <mensagem>")); std::process::exit(1); }
            };
            if args.len() < 2 { eprintln!("{}", usage_err("chat send <to_id> <mensagem>")); std::process::exit(1); }
            let text = args[1..].join(" ");
            let me = match client.current_user() {
                Ok(u) => u,
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            };
            // data atual simples
            let now = {
                use std::time::{SystemTime, UNIX_EPOCH};
                let s = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                let (h, m, sec) = ((s / 3600) % 24, (s / 60) % 60, s % 60);
                let days = (s / 86400) as i64;
                let z = days + 719468; let era = if z >= 0 { z } else { z - 146096 } / 146097;
                let doe = z - era * 146097; let yoe = (doe - doe/1460 + doe/36524 - doe/146096) / 365;
                let y_val = yoe + era * 400; let doy = doe - (365*yoe + yoe/4 - yoe/100);
                let mp = (5*doy + 2) / 153; let day = doy - (153*mp+2)/5 + 1;
                let month = if mp < 10 { mp + 3 } else { mp - 9 };
                let year = if month <= 2 { y_val + 1 } else { y_val };
                format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", year, month, day, h, m, sec)
            };
            let dto = medx::SendMessageDto::new(me.user_id, &me.full_name, to_id, "", &text, now);
            match client.send_chat_message(&dto) {
                Ok(_) => ok("mensagem enviada"),
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        _ => { if let Some(r) = find_resource("chat") { print_resource_help(r); } }
    }
}

fn dispatch_notif(cmd: Option<&str>, args: &[String]) {
    let c = require_client();
    match cmd {
        Some("settings") => {
            match c.client_settings() {
                Ok(s) => {
                    print_kv("id",           &s.id.to_string());
                    print_kv("software_id",  &s.software_id.to_string());
                    print_kv("logo",         &s.logo);
                    print_kv("instagram",    &s.instagram);
                    print_kv("facebook",     &s.facebook);
                    print_kv("website",      &s.website);
                    print_kv("sms template", if s.has_sms_template() { "sim" } else { "não" });
                    print_kv("whatsapp tmpl",if s.has_whatsapp_template() { "sim" } else { "não" });
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("log-email") => {
            let pid: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("notif log-email <patient_id> <para> <assunto>")); std::process::exit(1); }
            };
            let to = match args.get(1) {
                Some(s) => s.as_str(),
                None => { eprintln!("{}", usage_err("notif log-email <patient_id> <para> <assunto>")); std::process::exit(1); }
            };
            let subject = if args.len() > 2 { args[2..].join(" ") } else {
                eprintln!("{}", usage_err("notif log-email <patient_id> <para> <assunto>")); std::process::exit(1);
            };
            let dto = medx::MailLogDto::new(pid, to, &subject, "");
            match c.log_email(&dto) {
                Ok(_) => ok("e-mail registrado no log"),
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        _ => { if let Some(r) = find_resource("notif") { print_resource_help(r); } }
    }
}

fn dispatch_marketing(cmd: Option<&str>, args: &[String]) {
    let c = require_client();
    match cmd {
        Some("events") => {
            match c.events() {
                Ok(es) if es.is_empty() => println!("{}", dim("(nenhum evento)")),
                Ok(es) => {
                    for e in &es {
                        println!("  #{:<4} {} {}", e.event_id, dim(&e.type_name), dim(&e.url));
                    }
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("quests") => {
            match c.quests() {
                Ok(qs) if qs.is_empty() => println!("{}", dim("(nenhum questionário)")),
                Ok(qs) => {
                    for q in &qs { println!("  #{:<4} {}", q.id, q.name); }
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("diagnostico-qp") => {
            match c.diagnostico_qp() {
                Ok(ds) if ds.is_empty() => println!("{}", dim("(nenhum diagnóstico)")),
                Ok(ds) => {
                    for d in &ds { println!("  #{:<12} {}", d.id, d.name); }
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        _ => { if let Some(r) = find_resource("marketing") { print_resource_help(r); } }
    }
}

fn dispatch_hoje(cmd: Option<&str>, args: &[String]) {
    let client = require_client();
    match cmd {
        Some("notificacoes") => {
            match client.hoje_notificacoes() {
                Ok(ns) if ns.is_empty() => println!("{}", dim("(sem notificações)")),
                Ok(ns) => {
                    for n in &ns {
                        println!("  {} — {}", c(&n.tipo), dim(&n.event_url));
                    }
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("ultimos-atendidos") => {
            match client.ultimos_atendidos() {
                Ok(us) if us.is_empty() => println!("{}", dim("(nenhum atendido)")),
                Ok(us) => {
                    for u in &us {
                        println!("  #{:<6} {:<30} {}", u.patient_id, u.patient_name, dim(&u.date));
                    }
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("trial") => {
            match client.trial_info() {
                Ok(t) => {
                    print_kv("plano",        &t.plan);
                    print_kv("trial",        if t.is_trial() { "sim" } else { "não" });
                    print_kv("expira em",    &t.trial_expires_at);
                    print_kv("dias restant", &t.days_remaining.to_string());
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("notas") => {
            match client.notas() {
                Ok(ns) if ns.is_empty() => println!("{}", dim("(nenhuma nota)")),
                Ok(ns) => {
                    for n in &ns {
                        print_sep();
                        print_kv("id",       &n.id.to_string());
                        print_kv("data",     &n.date);
                        print_kv("autor",    &n.user_id.to_string());
                        print_kv("texto",    &n.text);
                    }
                    print_sep();
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("nota-add") => {
            let uid: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("hoje nota-add <user_id> <texto>")); std::process::exit(1); }
            };
            if args.len() < 2 { eprintln!("{}", usage_err("hoje nota-add <user_id> <texto>")); std::process::exit(1); }
            let text = args[1..].join(" ");
            let dto = medx::InsertNotaDto::new(uid, &text);
            match client.insert_nota(&dto) {
                Ok(id) => ok(&format!("nota #{id} criada")),
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("nota-del") => {
            let id: i64 = match args.get(0).and_then(|s| s.parse().ok()) {
                Some(id) => id,
                None => { eprintln!("{}", usage_err("hoje nota-del <nota_id>")); std::process::exit(1); }
            };
            match client.delete_nota(id) {
                Ok(_) => ok(&format!("nota #{id} removida")),
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        _ => { if let Some(r) = find_resource("hoje") { print_resource_help(r); } }
    }
}

fn dispatch_ajustes(cmd: Option<&str>, args: &[String]) {
    let c = require_client();
    match cmd {
        Some("reports") => {
            match c.list_reports() {
                Ok(rs) if rs.is_empty() => println!("{}", dim("(nenhum relatório)")),
                Ok(rs) => { for r in &rs { println!("  #{:<4} {:<40} {}", r.id, r.name, dim(&r.tipo)); } }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("docs") => {
            let filter = args.get(0).map(String::as_str).unwrap_or("");
            match c.doc_folders(filter) {
                Ok(fs) if fs.is_empty() => println!("{}", dim("(nenhuma pasta)")),
                Ok(fs) => { for f in &fs { println!("  #{:<4} {:<30} {}", f.id, f.name, dim(&f.filter_key)); } }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("ics") => {
            match c.ics_config() {
                Ok(ics) => {
                    print_kv("ativo",  if ics.active() { "sim" } else { "não" });
                    print_kv("url",    &ics.url);
                    print_kv("token",  &ics.token);
                }
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("change-password") => {
            let new_pw = match args.get(0) {
                Some(p) => p.as_str(),
                None => { eprintln!("{}", usage_err("ajustes change-password <nova-senha>")); std::process::exit(1); }
            };
            match c.change_password(new_pw) {
                Ok(_) => ok("senha alterada com sucesso"),
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        Some("change-password-full") => {
            let old_pw = match args.get(0) {
                Some(p) => p.as_str(),
                None => { eprintln!("{}", usage_err("ajustes change-password-full <atual> <nova>")); std::process::exit(1); }
            };
            let new_pw = match args.get(1) {
                Some(p) => p.as_str(),
                None => { eprintln!("{}", usage_err("ajustes change-password-full <atual> <nova>")); std::process::exit(1); }
            };
            match c.change_password_with_old(old_pw, new_pw) {
                Ok(_) => ok("senha alterada com sucesso"),
                Err(e) => { eprintln!("{} {e}", err_prefix()); std::process::exit(1); }
            }
        }
        _ => { if let Some(r) = find_resource("ajustes") { print_resource_help(r); } }
    }
}

fn dispatch(resource: &str, cmd: Option<&str>, args: &[String]) {
    match resource {
        "auth"       => dispatch_auth(cmd, args),
        "contacts"   => dispatch_contacts(cmd, args),
        "agenda"     => dispatch_agenda(cmd, args),
        "prontuario" => dispatch_prontuario(cmd, args),
        "financas"   => dispatch_financas(cmd, args),
        "chat"       => dispatch_chat(cmd, args),
        "notif"      => dispatch_notif(cmd, args),
        "marketing"  => dispatch_marketing(cmd, args),
        "hoje"       => dispatch_hoje(cmd, args),
        "ajustes"    => dispatch_ajustes(cmd, args),
        other        => print_unknown_resource(other),
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        // Sem argumentos ou --help / -h
        None | Some("--help") | Some("-h") => print_global_help(),

        // `medx-cli help` ou `medx-cli help <recurso>`
        Some("help") => {
            match args.get(2).map(String::as_str) {
                None => print_global_help(),
                Some(resource) => match find_resource(resource) {
                    Some(r) => print_resource_help(r),
                    None    => print_unknown_resource(resource),
                },
            }
        }

        // `medx-cli version` / `--version`
        Some("version") | Some("--version") | Some("-v") => show_version(),

        // `medx-cli <recurso> help`  →  ajuda específica do recurso
        Some(resource) if args.get(2).map(String::as_str) == Some("help") => {
            match find_resource(resource) {
                Some(r) => print_resource_help(r),
                None    => print_unknown_resource(resource),
            }
        }

        // `medx-cli <recurso> [comando] [args...]`
        Some(resource) => {
            let resource = resource.to_string();
            let cmd = args.get(2).map(String::as_str);
            let rest = if args.len() > 3 { args[3..].to_vec() } else { vec![] };
            dispatch(&resource, cmd, &rest);
        }
    }
}
