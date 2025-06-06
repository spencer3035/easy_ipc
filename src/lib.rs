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
mod packet;
/// Server process
pub mod server;

/// Common required imports
pub mod prelude {
    pub use crate::client::Client;
    pub use crate::model::ClientServerModel;
    pub use crate::server::Server;
}
