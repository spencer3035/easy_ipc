use std::thread::spawn;

use serde::{Deserialize, Serialize};

use crate::error::ConnectionError;
use crate::prelude::*;

/// Maybe something like
/// ```no_compile
/// #[derive(ClientServerModel)]
/// #[server_message(ServerMessage)]
/// #[client_message(ClientMessage)]
/// ```
// #[allow(dead_code)]
// pub struct Model;

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
            ClientServerOptions::new(default_socket(socket_name)).create()
        }
    }

    let model = MyModel.model();

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
