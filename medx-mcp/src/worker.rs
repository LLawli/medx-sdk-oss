//! A conexão com a MedX: uma thread própria, uma chamada por vez.
//!
//! O SDK é blocking, e o cliente reqwest blocking não pode ser criado nem
//! destruído dentro do runtime async. O `MedxClient` vive numa thread do
//! sistema, criado na primeira chamada e destruído quando a thread acaba. As
//! chamadas chegam por um canal e rodam em fila: com duas em paralelo e o
//! token vencido, os dois re-logins derrubariam um ao outro (a MedX aceita uma
//! sessão por conta).

use std::sync::mpsc;

use medx::{MedxClient, MedxError, Session};
use tokio::sync::oneshot;

use crate::config::{Config, ConnectionPlan, plan_connection};
use crate::tools::ToolError;

/// Uma chamada enfileirada: roda na thread, com acesso à conexão.
type Job = Box<dyn FnOnce(&mut Connection) + Send>;

pub struct MedxWorker {
    // Quando o último remetente cai, a thread sai do laço e destrói o
    // cliente, ainda dentro dela.
    jobs: mpsc::Sender<Job>,
}

/// O que a thread guarda: como conectar e, depois da primeira chamada que
/// deu certo, o cliente.
struct Connection {
    config: Config,
    session: Option<Session>,
    client: Option<MedxClient>,
}

impl Connection {
    /// O cliente, criado na primeira chamada. Uma falha não fica em cache: a
    /// chamada seguinte tenta de novo.
    fn client(&mut self) -> Result<&MedxClient, ToolError> {
        if self.client.is_none() {
            self.client = Some(self.connect()?);
        }
        self.client.as_ref().ok_or(ToolError::WorkerGone)
    }

    fn connect(&self) -> Result<MedxClient, ToolError> {
        match plan_connection(&self.config, self.session.clone()) {
            ConnectionPlan::Session {
                session,
                base_url,
                credentials: None,
            } => Ok(MedxClient::from_session_at(session, base_url)),
            ConnectionPlan::Session {
                session,
                base_url,
                credentials: Some(credentials),
            } => Ok(MedxClient::from_session_with_credentials_at(
                session,
                base_url,
                credentials.email,
                credentials.password,
            )),
            ConnectionPlan::Login { host, credentials } => Ok(MedxClient::login_at(
                &host,
                &credentials.email,
                &credentials.password,
            )?),
            ConnectionPlan::Missing => Err(ToolError::NoSession),
        }
    }
}

impl MedxWorker {
    /// Sobe a thread da conexão. Não abre conexão nenhuma: o cliente nasce na
    /// primeira chamada, conforme `config::plan_connection(config, session)`.
    ///
    /// - `ConnectionPlan::Session`: `MedxClient::from_session_at`, ou
    ///   `from_session_with_credentials_at` quando há credenciais.
    /// - `ConnectionPlan::Login`: `MedxClient::login_at`. Se o login falhar,
    ///   a chamada devolve o erro e a próxima tenta de novo.
    /// - `ConnectionPlan::Missing`: toda chamada devolve `ToolError::NoSession`.
    pub fn spawn(config: Config, session: Option<Session>) -> Self {
        let (jobs, queue) = mpsc::channel::<Job>();
        let mut connection = Connection {
            config,
            session,
            client: None,
        };
        let spawned = std::thread::Builder::new()
            .name("medx-worker".to_owned())
            .spawn(move || {
                while let Ok(job) = queue.recv() {
                    job(&mut connection);
                }
            });
        if let Err(err) = spawned {
            // Sem thread, a fila fica sem leitor e toda chamada vira `WorkerGone`.
            tracing::error!("não foi possível iniciar a thread da conexão com a MedX: {err}");
        }
        Self { jobs }
    }

    /// Roda `f` com o cliente na thread da conexão, depois das chamadas que
    /// chegaram antes, e devolve o resultado.
    pub async fn call<T, F>(&self, f: F) -> Result<T, ToolError>
    where
        F: FnOnce(&MedxClient) -> Result<T, MedxError> + Send + 'static,
        T: Send + 'static,
    {
        let (reply, response) = oneshot::channel();
        let job: Job = Box::new(move |connection| {
            let result = connection
                .client()
                .and_then(|client| f(client).map_err(ToolError::from));
            // O chamador pode ter desistido (cancelamento); não é erro.
            let _ = reply.send(result);
        });
        self.jobs.send(job).map_err(|_| ToolError::WorkerGone)?;
        // `reply` cai sem resposta se a thread morrer no meio da chamada.
        response.await.map_err(|_| ToolError::WorkerGone)?
    }
}
