use app_common::{ClientMessage, MyModel};
use clap::{Parser, Subcommand};
use easy_ipc::prelude::ClientServerModel;

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
    let args = Cli::parse();
    let msg: ClientMessage = args.op.into();
    let mut client = MyModel::client().unwrap();
    client.send(msg.clone()).unwrap();
    let resp = client.receive().unwrap();
    println!("{:?} => {:?}", msg, resp);
}
