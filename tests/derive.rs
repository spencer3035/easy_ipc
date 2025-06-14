use easy_ipc_derive::Model;
use serde::{Deserialize, Serialize};

#[derive(Model)]
#[server_message(ServerMessage)]
#[client_message(ClientMessage)]
struct MyModel;

#[derive(Serialize, Deserialize)]
enum ServerMessage {
    Help,
    Me,
}

#[derive(Serialize, Deserialize)]
enum ClientMessage {
    No,
}
