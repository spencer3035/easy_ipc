//! # IPC Made easy!
//!
//! Ever wanted interprocess communication, but get frustrated at all the boilerplate? This crate
//! is for you!
//!
//! Here we define a few simple traits that can be implemented to allow you to define how your
//! client and server processes will communicate. You need to do 3 things
//!
//! 1. Define a server message that implements [`serde::Serialize`] and [`serde::Deserialize`].
//! 1. Define a client message that implements [`serde::Serialize`] and [`serde::Deserialize`].
//! 1. Define a struct that implements [`model::IpcModel`], referencing your server and client messages.
//!
//! Then you are happy on your way to writing your application.
//!
//! # Example
//!
//! ```
//! use easy_ipc::prelude::*;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! enum ClientMessage {
//!     Start,
//!     Stop,
//! }
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! enum ServerMessage {
//!     Success,
//!     Fail,
//! }
//!
//! #[derive(IpcModel)]
//! #[easy_ipc(client_message = ClientMessage, server_message = ServerMessage)]
//! struct MyModel;
//!
//! fn main() {
//!     // Make new server (needs to be before client)
//!     let server = MyModel::server().unwrap();
//!     // Make a new client
//!     let mut client = MyModel::client().unwrap();
//!
//!     // Spawn server in new thread (normally this would be another process)
//!     let handle = std::thread::spawn(move || {
//!         // Handle incoming client connections
//!         for conn in server.connections() {
//!             // Ignore errors in connecting to client
//!             let mut conn = conn.unwrap();
//!             // Receive and print single message from client
//!             println!("{:?}", conn.receive().unwrap());
//!             // Send an Success signal back
//!             conn.send(ServerMessage::Success).unwrap();
//!             // Usually you would not want to break after one connection, we do it here so that
//!             // the function terminates.
//!             break;
//!         }
//!     });
//!
//!     // This would create a deadlock! The server thread expects to receive the first message.
//!     // let msg = client.receive().unwrap();
//!
//!     // Send a message to the server
//!     client.send(ClientMessage::Start).unwrap();
//!     // Get the response from the server
//!     println!("{:?}", client.receive().unwrap());
//!
//!     // We make sure to join the handle here so that the `server` instance is dropped. This
//!     // guarantees that the socket file is removed and we don't end up with a dangling socket file.
//!     // There are internal safeguards for this when used across processes.
//!     handle.join().unwrap();
//! }
//! ```
//!
//! # Limitations
//!
//! This crate cannot handle non-blocking send/receive messages at the moment.

/// Common required imports
pub mod prelude {
    pub use crate::ipc_model;
    pub use easy_ipc_derive::IpcModel;

    pub use crate::client::Client;
    pub use crate::model::ClientServerModel;
    pub use crate::model::ClientServerOptions;
    pub use crate::model::IpcModel;
    pub use crate::server::Server;
}

/// Client process
pub mod client;
/// Connection between client and server
pub mod connection;
/// Error enumerations
pub mod error;
/// Definition of client server model
pub mod model;
/// Handle getting default namespace information
pub mod namespace;
/// Server process
pub mod server;

/// Handle OS signals
mod handlers;
/// Helper macros
mod macros;
/// Tests
#[cfg(test)]
mod test;
