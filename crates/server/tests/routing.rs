use std::time::Duration;

use actix_web::{App, web, web::Bytes};
use awc::ws::{Frame, Message};
use cc_wired_server::ws::{Registry, configure};
use futures_util::{SinkExt, StreamExt};
use tokio::time::timeout;

fn spawn_app() -> actix_test::TestServer {
    actix_test::start(|| {
        App::new()
            .app_data(web::Data::new(Registry::default()))
            .configure(configure)
    })
}

const SHORT: Duration = Duration::from_millis(300);

/// Waits up to `dur` for the next item on `stream`, treating a timeout the
/// same as the stream ending: both mean "nothing arrived".
async fn recv_within<S>(stream: &mut S, dur: Duration) -> Option<S::Item>
where
    S: StreamExt + Unpin,
{
    timeout(dur, stream.next()).await.unwrap_or(None)
}

/// Ping/Pong round-trip used as a sync barrier: since a connection's frames
/// are dispatched in order, seeing the Pong proves everything sent before it
/// (including any `Registry::route` call) has already finished, no sleep needed.
async fn sync<S>(conn: &mut S)
where
    S: SinkExt<Message> + StreamExt<Item = Result<Frame, S::Error>> + Unpin,
    S::Error: std::fmt::Debug,
{
    conn.send(Message::Ping(Bytes::new())).await.unwrap();
    match recv_within(conn, SHORT).await {
        Some(Ok(Frame::Pong(_))) => {}
        other => panic!("expected Pong barrier response, got {other:?}"),
    }
}

#[actix_web::test]
async fn delivers_message_to_registered_consumer() {
    let mut srv = spawn_app();

    let mut consumer = srv.ws_at("/computer-1").await.unwrap();
    let mut producer = srv.ws_at("/").await.unwrap();

    let payload = r#"{"computer_id":"computer-1","payload":"hello"}"#;
    producer.send(Message::Text(payload.into())).await.unwrap();

    let frame = recv_within(&mut consumer, SHORT)
        .await
        .expect("expected a frame");
    match frame.unwrap() {
        Frame::Text(bytes) => assert_eq!(bytes, payload.as_bytes()),
        other => panic!("unexpected frame: {other:?}"),
    }
}

#[actix_web::test]
async fn does_not_deliver_to_wrong_consumer() {
    let mut srv = spawn_app();

    let mut other_consumer = srv.ws_at("/computer-2").await.unwrap();
    let mut producer = srv.ws_at("/").await.unwrap();

    let payload = r#"{"computer_id":"computer-1","payload":"hello"}"#;
    producer.send(Message::Text(payload.into())).await.unwrap();
    sync(&mut producer).await;

    let got = recv_within(&mut other_consumer, SHORT).await;
    assert!(got.is_none(), "expected no message, got {got:?}");
}

#[actix_web::test]
async fn drops_malformed_json() {
    let mut srv = spawn_app();

    let mut consumer = srv.ws_at("/computer-1").await.unwrap();
    let mut producer = srv.ws_at("/").await.unwrap();

    producer
        .send(Message::Text("not json".into()))
        .await
        .unwrap();
    sync(&mut producer).await;

    let got = recv_within(&mut consumer, SHORT).await;
    assert!(got.is_none(), "expected no message, got {got:?}");
}

#[actix_web::test]
async fn drops_message_missing_computer_id() {
    let mut srv = spawn_app();

    let mut consumer = srv.ws_at("/computer-1").await.unwrap();
    let mut producer = srv.ws_at("/").await.unwrap();

    producer
        .send(Message::Text(r#"{"payload":"hello"}"#.into()))
        .await
        .unwrap();
    sync(&mut producer).await;

    let got = recv_within(&mut consumer, SHORT).await;
    assert!(got.is_none(), "expected no message, got {got:?}");
}

#[actix_web::test]
async fn disconnect_removes_registration_so_new_connection_gets_nothing_stale()
{
    let mut srv = spawn_app();

    let consumer = srv.ws_at("/computer-1").await.unwrap();
    drop(consumer);

    let mut producer = srv.ws_at("/").await.unwrap();
    producer
        .send(Message::Text(
            r#"{"computer_id":"computer-1","payload":"stale?"}"#.into(),
        ))
        .await
        .unwrap();
    sync(&mut producer).await;

    let mut fresh_consumer = srv.ws_at("/computer-1").await.unwrap();
    let got = recv_within(&mut fresh_consumer, SHORT).await;
    assert!(
        got.is_none(),
        "expected no buffered/stale message, got {got:?}"
    );
}

#[actix_web::test]
async fn consumer_responds_to_ping() {
    let mut srv = spawn_app();
    let mut consumer = srv.ws_at("/computer-1").await.unwrap();

    sync(&mut consumer).await;
}
