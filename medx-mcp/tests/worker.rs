//! A conexão com a MedX: uma chamada por vez, e o cliente criado sob demanda.

mod common;

use std::time::Duration;

use common::fake_medx::FakeMedx;
use common::{call, connect, connect_fake, fixtures};
use serde_json::json;

#[tokio::test]
async fn chamadas_em_paralelo_chegam_a_medx_uma_por_vez() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/security/getcurrentuser",
        fixtures::current_user(),
    )
    .set_delay(Duration::from_millis(150));
    let client = connect_fake(&fake).await;

    let (a, b, c) = tokio::join!(
        call(&client, "usuario_atual", json!({})),
        call(&client, "usuario_atual", json!({})),
        call(&client, "usuario_atual", json!({})),
    );

    for result in [a, b, c] {
        assert_ne!(result.is_error, Some(true), "{result:?}");
    }
    assert_eq!(fake.requests().len(), 3);
    assert_eq!(
        fake.max_in_flight(),
        1,
        "a MedX recebeu chamadas sobrepostas"
    );
}

#[tokio::test]
async fn handshake_nao_chama_a_medx() {
    let fake = FakeMedx::start();
    let client = connect(fake.config(), Some(fake.session())).await;

    client.list_all_tools().await.expect("tools/list");

    assert!(fake.requests().is_empty(), "{:?}", fake.requests());
}
