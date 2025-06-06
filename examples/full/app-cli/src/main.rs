use clap::{Parser, Subcommand};
use easy_ipc::prelude::ClientServerModel;

// Bring in our model and messages into scope.
use app_common::{ClientMessage, MyModel};

/// Simple command line interface that lets you call it like `app-cli add 1 2`
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    op: Op,
}

#[derive(Subcommand, Debug)]
enum Op {
    Add { a: i32, b: i32 },
    Sub { a: i32, b: i32 },
    Div { a: i32, b: i32 },
    Mul { a: i32, b: i32 },
}

// Define a transformation from our CLI internal struct to our client messages. Hypothetically you
// could have them be the same struct, but it could get kind of messy.
impl From<Op> for ClientMessage {
    fn from(value: Op) -> Self {
        match value {
            Op::Add { a, b } => ClientMessage::Add(a, b),
            Op::Sub { a, b } => ClientMessage::Sub(a, b),
            Op::Mul { a, b } => ClientMessage::Mul(a, b),
            Op::Div { a, b } => ClientMessage::Div(a, b),
        }
    }
}

fn main() {
    // Parse cli
    let args = Cli::parse();
    // Convert args to a message
    let msg: ClientMessage = args.op.into();
    // Make our client
    let mut client = MyModel::client().unwrap();
    // Send the message to the server
    client.send(msg.clone()).unwrap();
    // Get the response and print it out
    let resp = client.receive().unwrap();
    println!("{:?} => {:?}", msg, resp);
}
