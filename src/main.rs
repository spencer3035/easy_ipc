use {
    interprocess::local_socket::{GenericFilePath, GenericNamespaced, Name, prelude::*},
    model::ClientServerModel,
    serde::{Deserialize, Serialize},
};

pub mod client;
pub mod connection;
pub mod model;
mod packet;
pub mod server;

/// Gets the name/file of the socket
// TODO: Needs to be generated
fn socket_name() -> Name<'static> {
    if GenericNamespaced::is_supported() {
        "example.sock".to_ns_name::<GenericNamespaced>().unwrap()
    } else {
        "/home/spencer/example.sock"
            .to_fs_name::<GenericFilePath>()
            .unwrap()
    }
}

/// Example Model
struct MyModel;

impl ClientServerModel<ClientMessage, ServerMessage> for MyModel {
    fn socket_name() -> Name<'static> {
        socket_name()
    }
}

/// Example server messages
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum ServerMessage {
    Ready,
    Ok,
    Fail,
}

/// Example client messages
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum ClientMessage {
    Run,
    Jump,
    Hide,
}

fn main() {
    let server = MyModel::server().unwrap();
    let mut client = MyModel::client().unwrap();

    std::thread::spawn(move || {
        for conn in server.connections() {
            println!("Got new connection!");
            let mut conn = conn.unwrap();
            let client_msg = conn.receive().unwrap();
            // receive
            dbg!(client_msg);
            conn.send(ServerMessage::Ok).unwrap();
        }
    });

    // This would create a deadlock
    // assert_eq!(client.receive().unwrap(), ServerMessage::Ready);
    println!("Client sending message");
    client.send(ClientMessage::Jump).unwrap();
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
