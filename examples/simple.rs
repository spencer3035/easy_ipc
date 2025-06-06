use easy_ipc::{prelude::ClientServerModel, socket_name};
use serde::{Deserialize, Serialize};

/// Example Model
struct MyModel;
impl ClientServerModel<ClientMessage, ServerMessage> for MyModel {
    fn socket_path() -> std::path::PathBuf {
        socket_name!()
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

fn main() {
    // Make new server (needs to be before client)
    let server = MyModel::server().unwrap();
    // Make a new client
    let mut client = MyModel::client().unwrap();

    // Spawn server in new thread (Normally this would be another process, just use
    // [`std::sync::mpsc`] if your application looks like this.
    let handle = std::thread::spawn(move || {
        // Handle incomming client connections
        for conn in server.connections() {
            // Ignore errors in connecting to client
            let mut conn = conn.unwrap();
            // Print single message from client
            println!("{:?}", conn.receive().unwrap());
            // Send an Ok signal back
            conn.send(ServerMessage::Ok).unwrap();
            // We only handle a single connection here! You do not want to break in your
            // application if you want to handle more than one client! We have this so that this
            // thread will terminate in our example.
            break;
        }
    });

    // This would create a deadlock! The server thread expects to recieve the first message.
    // let msg = client.receive().unwrap();

    // Send a message to the server
    client.send(ClientMessage::Start).unwrap();
    // Get the responce from the server
    println!("{:?}", client.receive().unwrap());

    // We make sure to join the handle here so that the `server` instance is dropped. This
    // gaurentees that the socket file is removed and we don't end up with a dangling socket file.
    handle.join().unwrap();
}
