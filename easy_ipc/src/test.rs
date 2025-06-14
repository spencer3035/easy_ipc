use std::thread::{sleep, spawn};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{ConnectionError, InitError};
use crate::model::cleanup;
use crate::prelude::*;

/// TODO: Add a derive for something like
/// ```no_compile
/// #[derive(ClientServerModel)]
/// #[server_message(ServerMessage)]
/// #[client_message(ClientMessage)]
/// pub struct Model;
/// ```

macro_rules! define_model {
    (
    $model_name:ident : $socket_name:literal,
    $server_enum:ident {$($s_msg:ident),* $(,)+},
    $client_enum:ident {$($c_msg:ident),* $(,)+},
) => {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    enum $server_enum {
        $($s_msg),*
    }
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    enum $client_enum {
        $($c_msg),*
    }
    struct $model_name;
    impl Model for $model_name {
        type ServerMsg = $server_enum;
        type ClientMsg = $client_enum;

        fn model() -> ClientServerModel<Self::ClientMsg, Self::ServerMsg> {
            let socket_name = $socket_name;
            ClientServerOptions::new($crate::model::default_socket(socket_name))
                .disable_single_server_check()
                .handlers(|_model| {})
                .create()
        }
    }
    };
}

#[test]
fn basic_multi_client() {
    define_model!(
        BasicModel: "basic_multi_client.socket",
        ServerMessage {
            Pong,
        },
        ClientMessage {
            Ping,
        },
    );

    let model = BasicModel::model();
    cleanup(&model.options().socket_name);
    let server = BasicModel::server().unwrap();

    let mut handles = Vec::new();
    let num_conn = 10;

    let handle = spawn(move || {
        let mut count = 0;
        for mut conn in server.connections().map(|c| c.unwrap()) {
            count += 1;
            assert_eq!(conn.receive().unwrap(), ClientMessage::Ping);
            conn.send(ServerMessage::Pong).unwrap();
            // Break after reaching the correct number of connections
            if count >= num_conn {
                break;
            }
        }
    });
    handles.push(handle);

    for _ii in 0..num_conn {
        let mut client = BasicModel::client().unwrap();
        let handle = spawn(move || {
            client.send(ClientMessage::Ping).unwrap();
            assert_eq!(client.receive().unwrap(), ServerMessage::Pong);
            sleep(Duration::from_millis(50));
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn basic_multi_server() {
    define_model!(
        BasicModel: "basic_multi_server.socket",
        ServerMessage {
            Pong,
        },
        ClientMessage {
            Ping,
        },
    );

    let model = BasicModel::model();
    cleanup(&model.options().socket_name);
    let _server = BasicModel::server().unwrap();
    assert!(matches!(
        BasicModel::server().unwrap_err(),
        InitError::SocketAlreadyExists,
    ));
}

#[test]
fn basic_send_receive() {
    define_model!(
        BasicModel: "basic_send_receive.socket",
        ServerMessage {
            Pong,
        },
        ClientMessage {
            Ping,
        },
    );

    let model = BasicModel::model();
    cleanup(&model.options().socket_name);

    let server = BasicModel::server().unwrap();
    assert!(matches!(model.options().socket_name.try_exists(), Ok(true)));

    let handle = spawn(move || {
        for conn in server.connections() {
            let mut conn = conn.unwrap();
            assert_eq!(ClientMessage::Ping, conn.receive().unwrap());
            conn.send(ServerMessage::Pong).unwrap();
            break;
        }
    });

    let mut client = BasicModel::client().unwrap();
    client.send(ClientMessage::Ping).unwrap();
    assert_eq!(ServerMessage::Pong, client.receive().unwrap());

    // This makes sure the server is dropped
    handle.join().unwrap();

    // Now the socket shouldn't exist anymore
    assert!(matches!(
        model.options().socket_name.try_exists(),
        Ok(false)
    ));

    // Server is closed, should get errors when sending and receiving
    assert!(matches!(
        client.send(ClientMessage::Ping).unwrap_err(),
        ConnectionError::WriteFailed(_)
    ));
    assert!(matches!(
        client.receive().unwrap_err(),
        ConnectionError::UnexepctedEof
    ));
}
