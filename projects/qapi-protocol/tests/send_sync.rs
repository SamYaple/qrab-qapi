//! The client handles are `Send` and `Sync`, and the future of every client
//! and mock call is `Send`, so they can be shared and spawned on a
//! multi-threaded runtime. Each check is a type bound: if a type or a future
//! loses `Send` or `Sync`, this file no longer compiles. The futures are only
//! built inside closures that never run.

use qapi::commands::QueryStatus;
use qapi::qga::commands::{GuestPing, GuestShutdown};
use qapi_protocol::server::{QgaSession, QmpReader, QmpSession, QmpWriter};
use qapi_protocol::{ClientOptions, Error, Qmp, QmpEvents, qga::Qga};
use serde_json::Value;
use std::{os::fd::OwnedFd, path::Path, time::Duration};
use tokio::net::{UnixListener, UnixStream};

fn send_sync<T: Send + Sync>() {}

fn send<T: Send>(_: T) {}

/// A value of any type, for the closures that are only type-checked.
fn any<T>() -> T {
    unreachable!("the closures are never called")
}

#[test]
fn handles_are_send_and_sync() {
    send_sync::<Qmp>();
    send_sync::<QmpEvents>();
    send_sync::<Qga>();
    send_sync::<Error>();
    send_sync::<QmpSession>();
    send_sync::<QmpReader>();
    send_sync::<QmpWriter>();
    send_sync::<QgaSession>();
}

#[test]
fn qmp_futures_are_send() {
    let _: fn() = || {
        let path: &Path = any();
        send(Qmp::new(path));
        send(Qmp::connect_with_options(path, ClientOptions::default()));
        send(Qmp::from_stream(any::<UnixStream>()));
        send(Qmp::from_stream_with_options(
            any::<UnixStream>(),
            ClientOptions::default(),
        ));

        let qmp: &Qmp = any();
        let status = QueryStatus::default();
        let timeout = Duration::from_secs(1);
        send(qmp.closed());
        send(qmp.shutdown());
        send(qmp.execute(&status));
        send(qmp.execute_with_timeout(&status, timeout));
        send(qmp.execute_oob(&status));
        send(qmp.execute_oob_with_timeout(&status, timeout));
        send(qmp.execute_raw::<Value>("query-status", Value::Null));
        send(qmp.execute_raw_with_timeout::<Value>("query-status", Value::Null, timeout));
        send(qmp.execute_raw_oob::<Value>("query-status", Value::Null));
        send(qmp.execute_raw_oob_with_timeout::<Value>("query-status", Value::Null, timeout));
        send(qmp.execute_raw_with_fds::<Value>("getfd", Value::Null, any::<Vec<OwnedFd>>()));
        send(qmp.execute_raw_with_fds_and_timeout::<Value>(
            "getfd",
            Value::Null,
            any::<Vec<OwnedFd>>(),
            timeout,
        ));
        send(qmp.getfd("fd", any::<OwnedFd>()));

        send(any::<&mut QmpEvents>().recv());
    };
}

#[test]
fn qga_futures_are_send() {
    let _: fn() = || {
        let path: &Path = any();
        send(Qga::new(path));
        send(Qga::connect_with_options(path, ClientOptions::default()));
        send(Qga::from_stream(any::<UnixStream>()));
        send(Qga::from_stream_with_options(
            any::<UnixStream>(),
            ClientOptions::default(),
        ));

        let qga: &mut Qga = any();
        send(qga.execute(&GuestPing::default()));
        send(qga.execute_with_reply_limit(&GuestPing::default(), 1024));
        send(qga.execute_raw::<Value>("guest-ping", Value::Null));
        send(qga.send_no_response(&GuestShutdown::default()));
    };
}

#[test]
fn mock_futures_are_send() {
    let _: fn() = || {
        send(QmpSession::accept(any::<&UnixListener>()));
        let session: &mut QmpSession = any();
        send(session.send_greeting(11, 1, 0));
        send(session.negotiate());
        send(session.recv_command());
        send(session.recv_command_full());
        send(session.send_return(Value::Null));
        send(session.send_return_with_id(Value::Null, None));
        send(session.send_error("GenericError", "failed"));
        send(session.send_event("STOP", Value::Null));

        let reader: &mut QmpReader = any();
        send(reader.recv_command_full());
        let writer: &mut QmpWriter = any();
        send(writer.send_return_with_id(Value::Null, None));
        send(writer.send_event("STOP", Value::Null));

        send(QgaSession::accept(any::<&UnixListener>()));
        let session: &mut QgaSession = any();
        send(session.handle_sync());
        send(session.recv_command());
        send(session.send_return(Value::Null));
        send(session.send_error("GenericError", "failed"));
    };
}
