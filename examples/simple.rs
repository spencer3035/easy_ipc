//! This is a toy example of how to use easy_ipc in a single file for learning purposes. If your
//! use case is simple enough to fit in a single file, consider using [`std::sync::mpsc`] instead.
//!
//! In general you would have this code spread across 3 different crates, a client binary crate, a
//! server binary crate, and a common library crate where you define the client and server messages
//! and your structure that implements [`ClientServerModel`]. You can see a more normal
//! implementation at [../examples/full/].

use easy_ipc::{prelude::*, socket_name};
use serde::{Deserialize, Serialize};

/// Example Model
struct MyModel;
impl Model<ClientMessage, ServerMessage> for MyModel {
    fn model(self) -> ClientServerModel<ClientMessage, ServerMessage> {
        ClientServerOptions::new(socket_name!()).create()
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
    let model = MyModel.model();
    // Make new server (needs to be before client)
    let server = model.server().unwrap();
    // Make a new client
    let mut client = model.client().unwrap();

    // Spawn server in new thread (Normally this would be another process)
    let handle = std::thread::spawn(move || {
        // Handle incoming client connections
        for conn in server.connections() {
            // Ignore errors in connecting to client
            let mut conn = conn.unwrap();
            // Receive and print single message from client
            println!("{:?}", conn.receive().unwrap());
            // Send an Ok signal back
            conn.send(ServerMessage::Ok).unwrap();
            // We only handle a single connection here! You do not want to break in your
            // application if you want to handle more than one client! We have this so that the
            // thread will terminate in our example.
            break;
        }
    });

    // This would create a deadlock! The server thread expects to receive the first message.
    // let msg = client.receive().unwrap();

    // Send a message to the server
    client.send(ClientMessage::Start).unwrap();
    // Get the response from the server
    println!("{:?}", client.receive().unwrap());

    // We make sure to join the handle here so that the `server` instance is dropped. This
    // guarantees that the socket file is removed and we don't end up with a dangling socket file.
    // There are internal safeguards for this.
    handle.join().unwrap();
}
