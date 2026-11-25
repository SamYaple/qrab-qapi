# qrab-qapi-protocol

Async clients for QEMU's QMP monitor and the QEMU guest agent (QGA) over Unix
sockets, built on Tokio and the generated types of `qrab-qapi`, plus bounded
mock servers for tests. Each connection negotiates and owns its peer's QEMU
version, so requests and replies are checked against that release.

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
    let status = qmp.execute(&qapi::commands::QueryStatus {}).await?;
    println!("running: {}", status.running);
    qmp.shutdown().await;
    Ok(())
}
```

The mock servers are the `server` module, behind the `mock` feature:

```toml
[dev-dependencies]
qapi-protocol = { package = "qrab-qapi-protocol", version = "0.1", features = ["mock"] }
```

## License

Apache-2.0.
