use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread::spawn,
};

use app_common::{ClientMessage, MyModel, ServerMessage};
use easy_ipc::{connection::Connection, model::Model};

static RUNNING: AtomicBool = AtomicBool::new(true);

/// Handles an incomming connection
fn handle_incomming_connection(mut conn: Connection<ServerMessage, ClientMessage>) {
    // Get a message from the client and print it out
    let msg = conn.receive().unwrap();
    println!("Got: {:?}", msg);

    // Craft a response based on the message
    let resp = match msg {
        ClientMessage::Add(a, b) => ServerMessage::Ok(a + b),
        ClientMessage::Sub(a, b) => ServerMessage::Ok(a - b),
        ClientMessage::Mul(a, b) => ServerMessage::Ok(a * b),
        ClientMessage::Div(a, b) => {
            if b == 0 {
                ServerMessage::DivByZero
            } else {
                ServerMessage::Ok(a / b)
            }
        }
        ClientMessage::Stop => {
            RUNNING.store(false, Ordering::Relaxed);
            spawn(move || {
                // This dummy client makes the server go into the next iteration and check the
                // running variable so that it exits immediately instead of waiting for the next
                // client to send a message. We ignore errors because the server might have stopped
                // already.
                if let Ok(mut kill_client) = MyModel.client() {
                    let _ = kill_client.send(ClientMessage::Stop);
                }
            });
            ServerMessage::Stopping
        }
    };

    // Send it back to the client
    println!("Sending: {:?}", resp);
    conn.send(resp).unwrap();
}

fn main() {
    // Create our server
    let server = MyModel.server().unwrap();
    let mut threads = Vec::new();

    // Loop over all incoming connections
    for conn in server.connections() {
        // First check if the last connection set the stop signal
        if !RUNNING.load(Ordering::Relaxed) {
            break;
        }

        // Handle connection
        match conn {
            Ok(c) => {
                // Spawn the handling of the client to a new thread so we can immediately handle
                // the next connection instead of waiting for each connection to finish. In a real
                // application you would want to manage how many threads were spawned.
                let handle = std::thread::spawn(move || handle_incomming_connection(c));
                threads.push(handle);
            }
            Err(e) => {
                println!("Couldn't connect: {e}");
            }
        }
    }

    // Join the threads, we aren't technically able to get here because the loop over the
    // connections will never terminate, but it's the thought that counts.
    for handle in threads {
        handle.join().unwrap();
    }
}
