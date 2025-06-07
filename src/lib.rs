//! Ever wanted interprocess communication, but get frustrated at all the boilerplate? This crate
//! is for you!
//!
//! Here we define a few simple traits that can be implemented to allow you to define how your
//! client and server processes will communicate. You need to define a server message, a client
//! message, and implement [`ClientServerModel`] on a struct and then you are good to go! Here is
//! an example.
//!
//!
//! ```
//! use easy_ipc::prelude::*;
//! use serde::{Deserialize, Serialize};
//!
//! enum ClientMessage {
//!     Start,
//!     Stop,
//! }
//!
//! enum ServerMessage {
//!     Ok,
//!     Fail,
//! }
//!
//! //struct MyModel;
//! //impl ClientServerModel<ClientMessage, ServerMessage> for MyModel {}
//! ```
//!
//! Then you are happy on your way to writing an application

/// Client process
pub mod client;
/// Connection between client and server
pub mod connection;
/// Error enumerations
pub mod error;
/// Definition of client server model
pub mod model;
/// Packets used to send data across the sockets
pub(crate) mod packet;
/// Server process
pub mod server;

/// Common required imports
pub mod prelude {
    pub use crate::client::Client;
    pub use crate::model::ClientServerModel;
    pub use crate::model::ClientServerOptions;
    pub use crate::model::Model;
    pub use crate::model::default_socket;
    pub use crate::server::Server;
}

#[cfg(test)]
mod test {
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
        let server = model.server().unwrap();
        assert!(matches!(model.options().socket_name.try_exists(), Ok(true)));

        let handle = spawn(move || {
            for conn in server.connections() {
                let mut conn = conn.unwrap();
                assert_eq!(ClientMessage::Ping, conn.receive().unwrap());
                conn.send(ServerMessage::Pong).unwrap();
                break;
            }
        });

        let mut client = model.client().unwrap();
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
}
