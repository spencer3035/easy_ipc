use app_common::{ClientMessage, MyModel, ServerMessage};
use easy_ipc::model::ClientServerModel;

fn main() {
    // Create our server
    let server = MyModel::server().unwrap();

    // Loop over all incoming connections
    for conn in server.connections() {
        // Ignore connection errors
        let mut conn = conn.unwrap();
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
        };

        // Send it back to the client
        println!("Sending: {:?}", resp);
        conn.send(resp).unwrap();
    }
}
