# qrab-qapi-protocol

Async clients for QEMU's QMP monitor and the QEMU guest agent (QGA) over Unix
sockets, built on Tokio and the generated types of `qrab-qapi`, plus bounded
mock servers for tests. Each connection negotiates and owns its peer's QEMU
version, so requests and replies are checked against that release, and asks
the peer once for the commands it offers, so that a command this QEMU or
guest agent cannot run fails before anything is sent.

The package is `qrab-qapi-protocol`; the library is `qapi_protocol`:

```toml
[dependencies]
qapi = { package = "qrab-qapi", version = "0.1" }
qapi-protocol = { package = "qrab-qapi-protocol", version = "0.1" }
```

```rust,no_run
use std::path::Path;

async fn print_status() -> qapi_protocol::Result<()> {
    let (qmp, _events) = qapi_protocol::Qmp::new(Path::new("/run/vm/qmp.sock")).await?;
    let status = qmp.execute(&qapi::commands::QueryStatus::default()).await?;
    println!("running: {}", status.running);
    qmp.shutdown().await;
    Ok(())
}
```

The clients run on Tokio and need the `tokio` feature, which is on by
default. Without it, the crate has only its types that need no runtime, such
as `Error` and `ClientOptions`.

The mock servers are the `server` module, behind the `mock` feature:

```toml
[dev-dependencies]
qapi-protocol = { package = "qrab-qapi-protocol", version = "0.1", features = ["mock"] }
```

## License

Apache-2.0.
