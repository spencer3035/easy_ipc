use serde::{Deserialize, Serialize};

use easy_ipc::{prelude::ClientServerModel, socket_name};

/// Example Model
struct MyModel;
impl ClientServerModel<ClientMessage, ServerMessage> for MyModel {
    fn socket_name() -> easy_ipc::model::Name {
        socket_name!().unwrap()
    }
}

/// Example server messages
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum ServerMessage {
    Ok,
    Fail,
}

/// Example client messages
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum ClientMessage {
    Start,
    Stop,
}

/// Run your server
fn run_server() {
    // Make a new server
    let server = MyModel::server().unwrap();

    // Handle incomming client connections
    for conn in server.connections() {
        let mut conn = conn.unwrap();
        let client_msg = conn.receive().unwrap();
        // receive
        dbg!(client_msg);
        conn.send(ServerMessage::Ok).unwrap();
    }
}

fn main() {
    // Make a new client
    let mut client = MyModel::client().unwrap();

    // Spawn server in new thread (You would do this
    std::thread::spawn(move || {
        run_server();
    });

    // This would create a deadlock
    // assert_eq!(client.receive().unwrap(), ServerMessage::Ready);
    println!("Client sending message");
    client.send(ClientMessage::Start).unwrap();
    println!("Getting message from server");
    let server_message = client.receive().unwrap();
    dbg!(server_message);

    // dbg!(env!("CARGO_CRATE_NAME"));
    // dbg!(env!("CARGO_PKG_VERSION"));
    // dbg!(env!("CARGO_PKG_VERSION_MAJOR"));
    // dbg!(env!("CARGO_PKG_VERSION_MINOR"));
    // dbg!(env!("CARGO_PKG_VERSION_PATCH"));
    // dbg!(env!("CARGO_PKG_VERSION_PRE"));
}

mod test {

    /// Maybe something like
    /// ```
    /// #[derive(ClientServerModel)]
    /// #[server_message(ServerMessage)]
    /// #[client_message(ClientMessage)]
    /// ```
    #[allow(dead_code)]
    pub struct Model;

    #[allow(dead_code)]
    fn tmp() {
        // let client = Model::client();
        // let server = Model::server();
    }
}
