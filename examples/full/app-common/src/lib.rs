use easy_ipc::prelude::*;
use serde::{Deserialize, Serialize};

/// Define you model and implement [`ClientServerModel`] for it.
#[derive(Debug, Copy, Clone)]
pub struct MyModel;

impl Model for MyModel {
    type ServerMsg = ServerMessage;
    type ClientMsg = ClientMessage;

    fn model() -> ClientServerModel<Self::ClientMsg, Self::ServerMsg> {
        model!()
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
