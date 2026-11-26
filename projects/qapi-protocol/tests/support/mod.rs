#![allow(dead_code)]
use qapi_protocol::qga::Qga;
use qapi_protocol::server::QgaSession;
use qapi_protocol::{ClientOptions, Qmp, QmpEvents, server::QmpSession};
use tokio::net::UnixStream;

pub async fn qmp(options: ClientOptions) -> (Qmp, QmpEvents, QmpSession) {
    qmp_version(options, (11, 1, 3)).await
}

pub async fn qmp_version(
    options: ClientOptions,
    version: (u32, u32, u32),
) -> (Qmp, QmpEvents, QmpSession) {
    let (client, server) = UnixStream::pair().unwrap();
    let mut peer = QmpSession::from_stream(server);
    let (client, ()) = tokio::join!(Qmp::from_stream_with_options(client, options), async {
        peer.send_greeting(version.0, version.1, version.2)
            .await
            .unwrap();
        peer.negotiate().await.unwrap();
    });
    let (client, events) = client.unwrap();
    (client, events, peer)
}

pub async fn qga(options: ClientOptions) -> (Qga, QgaSession) {
    qga_version(options, "11.1.3").await
}

pub async fn qga_version(options: ClientOptions, version: &str) -> (Qga, QgaSession) {
    let (client, server) = UnixStream::pair().unwrap();
    let mut peer = QgaSession::from_stream(server);
    let (client, ()) = tokio::join!(Qga::from_stream_with_options(client, options), async {
        peer.handle_sync().await.unwrap();
        assert_eq!(peer.recv_command().await.unwrap().0, "guest-info");
        peer.send_return(serde_json::json!({"version": version, "supported_commands": []}))
            .await
            .unwrap();
    });
    (client.unwrap(), peer)
}

pub type RawReader = tokio::io::BufReader<tokio::net::unix::OwnedReadHalf>;
pub type RawWriter = tokio::net::unix::OwnedWriteHalf;

pub async fn write(writer: &mut RawWriter, value: serde_json::Value) {
    use tokio::io::AsyncWriteExt;
    let mut bytes = serde_json::to_vec(&value).unwrap();
    bytes.extend_from_slice(b"\r\n");
    writer.write_all(&bytes).await.unwrap();
}
pub async fn read(reader: &mut RawReader) -> serde_json::Value {
    use tokio::io::AsyncBufReadExt;
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    serde_json::from_str(&line).unwrap()
}
pub fn greeting() -> serde_json::Value {
    serde_json::json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 1, "micro": 3}, "package": "test"}, "capabilities": ["oob"]}})
}
/// The error reply of a peer without `query-commands`, after which the
/// client checks calls against the release alone.
pub fn probe_refusal(id: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({"error": {"class": "CommandNotFound", "desc": "The command query-commands has not been found"}, "id": id})
}
pub async fn raw_qmp(options: ClientOptions) -> (Qmp, QmpEvents, RawReader, RawWriter) {
    let (client, peer) = UnixStream::pair().unwrap();
    let (read_half, mut writer) = peer.into_split();
    let mut reader = tokio::io::BufReader::new(read_half);
    let probe = options.probe_commands;
    let (client, ()) = tokio::join!(Qmp::from_stream_with_options(client, options), async {
        write(&mut writer, greeting()).await;
        let caps = read(&mut reader).await;
        write(
            &mut writer,
            serde_json::json!({"return": {}, "id": caps["id"]}),
        )
        .await;
        if probe {
            let query = read(&mut reader).await;
            assert_eq!(query["execute"], "query-commands");
            write(&mut writer, probe_refusal(&query["id"])).await;
        }
    });
    let (client, events) = client.unwrap();
    (client, events, reader, writer)
}
