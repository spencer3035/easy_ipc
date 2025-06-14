use easy_ipc_derive::Model;
use serde::{Deserialize, Serialize};

// Enum model
#[derive(Model)]
#[easy_ipc(server_message = ServerEnumMessage, client_message = ClientEnumMessage)]
struct MyEnumModel;

#[derive(Serialize, Deserialize)]
enum ServerEnumMessage {
    Help,
    Me,
}

#[derive(Serialize, Deserialize)]
enum ClientEnumMessage {
    No,
}

// Struct model
#[derive(Model)]
#[easy_ipc(server_message = ServerStructMessage, client_message = ClientStructMessage)]
struct MyStructModel;

#[derive(Serialize, Deserialize)]
struct ServerStructMessage {
    data: u32,
}

#[derive(Serialize, Deserialize)]
struct ClientStructMessage {
    send: u32,
}
