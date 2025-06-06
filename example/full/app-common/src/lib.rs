use easy_ipc::{prelude::*, socket_name};
use serde::{Deserialize, Serialize};

pub struct MyModel;

impl ClientServerModel<ClientMessage, ServerMessage> for MyModel {
    fn socket_name() -> easy_ipc::model::Name {
        socket_name!().unwrap()
    }
}

/// Operations to send to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Add(i32, i32),
    Sub(i32, i32),
    Mul(i32, i32),
    Div(i32, i32),
}

/// Possible responces to give to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Ok(i32),
    DivByZero,
}
