use easy_ipc::{error::InitError, prelude::*};
use serde::{Deserialize, Serialize};

/// Define you model and implement [`ClientServerModel`] for it.
#[derive(Debug, Copy, Clone)]
pub struct MyModel;

impl IpcModel for MyModel {
    type ServerMsg = ServerMessage;
    type ClientMsg = ClientMessage;

    fn model() -> Result<ClientServerModel<Self::ClientMsg, Self::ServerMsg>, InitError> {
        Ok(ClientServerOptions::new("calculator").create())
    }
}

/// Operations to send to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Add(i32, i32),
    Sub(i32, i32),
    Mul(i32, i32),
    Div(i32, i32),
    Stop,
}

/// Possible responces to give to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Ok(i32),
    Stopping,
    DivByZero,
}
