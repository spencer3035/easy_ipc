use std::thread::spawn;

use serde::{Deserialize, Serialize};

use crate::error::ConnectionError;
use crate::model::cleanup;
use crate::prelude::*;

/// Maybe something like
/// ```no_compile
/// #[derive(ClientServerModel)]
/// #[server_message(ServerMessage)]
/// #[client_message(ClientMessage)]
/// ```
// #[allow(dead_code)]
// pub struct Model;

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

        fn model(self) -> ClientServerModel<Self::ClientMsg, Self::ServerMsg> {
            let socket_name = $socket_name;
            println!("Starting {}", socket_name);
            ClientServerOptions::new(default_socket(socket_name))
                .handlers(|_model| {})
                .create()
        }
    }
    };
}

#[test]
fn basic_macro_test() {
    define_model!(
        BasicModel: "basic_macro_test.socket",
        ServerMessage {
            Pong,
        },
        ClientMessage {
            Ping,
        },
    );

    let model = BasicModel.model();
    cleanup(&model.options().socket_name);

    let server = BasicModel.server().unwrap();
    assert!(matches!(model.options().socket_name.try_exists(), Ok(true)));

    let handle = spawn(move || {
        for conn in server.connections() {
            let mut conn = conn.unwrap();
            assert_eq!(ClientMessage::Ping, conn.receive().unwrap());
            conn.send(ServerMessage::Pong).unwrap();
            break;
        }
    });

    let mut client = BasicModel.client().unwrap();
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

#[test]
fn basic_send_receive() {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    enum ServerMessage {
        Pong,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    enum ClientMessage {
        Ping,
    }

    struct MyModel;
    impl Model for MyModel {
        type ServerMsg = ServerMessage;
        type ClientMsg = ClientMessage;

        fn model(self) -> ClientServerModel<Self::ClientMsg, Self::ServerMsg> {
            let socket_name = "basic_send_receive.sock";
            ClientServerOptions::new(default_socket(socket_name))
                .handlers(|_model| {})
                .create()
        }
    }

    let model = MyModel.model();
    cleanup(&model.options().socket_name);

    let server = MyModel.server().unwrap();
    assert!(matches!(model.options().socket_name.try_exists(), Ok(true)));

    let handle = spawn(move || {
        for conn in server.connections() {
            let mut conn = conn.unwrap();
            assert_eq!(ClientMessage::Ping, conn.receive().unwrap());
            conn.send(ServerMessage::Pong).unwrap();
            break;
        }
    });

    let mut client = MyModel.client().unwrap();
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
