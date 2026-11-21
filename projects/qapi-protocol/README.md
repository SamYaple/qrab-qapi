# qrab-qapi-protocol

Async clients for QEMU's QMP monitor and the QEMU guest agent (QGA) over Unix
sockets, built on Tokio and the generated types of `qrab-qapi`, plus bounded
mock servers for tests. Each connection negotiates and owns its peer's QEMU
version, so requests and replies are checked against that release.

## License

Apache-2.0.
