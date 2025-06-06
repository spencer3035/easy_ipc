use app_common::{ClientMessage, MyModel, ServerMessage};
use easy_ipc::model::ClientServerModel;

fn main() {
    let server = MyModel::server().unwrap();
    for conn in server.connections() {
        let mut conn = conn.unwrap();
        let msg = conn.receive().unwrap();
        println!("Got: {:?}", msg);
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
        println!("Sending: {:?}", resp);
        conn.send(resp).unwrap();
    }
}
