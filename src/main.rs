use {
    interprocess::local_socket::{
        GenericFilePath, GenericNamespaced, ListenerOptions, Name, Stream, prelude::*,
    },
    serde::{Deserialize, Serialize},
    std::{
        io::{BufReader, prelude::*},
        marker::PhantomData,
    },
};

fn socket_name() -> Name<'static> {
    if GenericNamespaced::is_supported() {
        "example.sock".to_ns_name::<GenericNamespaced>().unwrap()
    } else {
        "/home/spencer/example.sock"
            .to_fs_name::<GenericFilePath>()
            .unwrap()
    }
}

struct Client {
    conn: Connection<ClientMessage, ServerMessage>,
}

impl Client {
    pub fn init() -> Self {
        let name = socket_name();
        let stream = Stream::connect(name).unwrap();
        let conn = Connection::new(stream);
        Client { conn }
    }

    pub fn send(&mut self, msg: ClientMessage) {
        self.conn.send(msg);
    }

    pub fn recieve(&mut self) -> ServerMessage {
        self.conn.receive()
    }
}

struct Server {
    listener: LocalSocketListener,
}

impl Server {
    pub fn init() -> Self {
        let name = socket_name();
        let opts = ListenerOptions::new().name(name);
        // Can fail for IO reasons
        let listener = opts.create_sync().unwrap();
        Server { listener }
    }

    /// Gets an infinate iterator over client connections
    pub fn connections(&self) -> impl Iterator<Item = Connection<ServerMessage, ClientMessage>> {
        self.listener
            .incoming()
            .map(|conn| Connection::new(conn.unwrap()))
    }
}

struct Connection<S, R> {
    connection: BufReader<Stream>,
    _send: PhantomData<S>,
    _receive: PhantomData<R>,
}

impl<S, R> Connection<S, R>
where
    S: Serialize,
    R: for<'de> Deserialize<'de>,
{
    fn new(connection: Stream) -> Self {
        let connection = BufReader::new(connection);
        Connection {
            connection,
            _send: PhantomData,
            _receive: PhantomData,
        }
    }

    pub fn send(&mut self, message: S) {
        let bytes = bitcode::serialize(&message).unwrap();
        let packet = Packet::new(bytes);
        let packet_bytes = packet.to_bytes();
        self.connection.get_mut().write_all(&packet_bytes).unwrap();
    }

    pub fn receive(&mut self) -> R {
        let mut header: [u8; 1] = [0];
        println!("Reading header");
        let nread = self.connection.read(&mut header).unwrap();
        // let nread = self.connection.read_exact(&mut header).unwrap();
        assert_eq!(nread, 1, "Couldn't read the header: {:?}", header);
        let len = header[0];
        let mut data = vec![0; len as usize];
        println!("Reading data");
        let nread = self.connection.read(&mut data).unwrap();
        assert_eq!(nread, len as usize, "Didn't read enough data");
        let val: R = bitcode::deserialize(&data).unwrap();
        val
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum ServerMessage {
    Ready,
    Ok,
    Fail,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum ClientMessage {
    Run,
    Jump,
    Hide,
}

struct Packet {
    len: u8,
    bytes: Vec<u8>,
}

impl Packet {
    fn new(data: Vec<u8>) -> Self {
        assert!(data.len() <= u8::max as usize);
        Packet {
            len: data.len() as u8,
            bytes: data,
        }
    }
    fn to_bytes(mut self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(self.bytes.len() + 1);
        vec.push(self.len);
        vec.append(&mut self.bytes);
        vec
    }
}

fn main() {
    let server = Server::init();
    let mut client = Client::init();

    std::thread::spawn(move || {
        for mut conn in server.connections() {
            println!("Got new connection!");
            let client_msg = conn.receive();
            dbg!(client_msg);
            conn.send(ServerMessage::Ok);
        }
    });

    assert_eq!(client.recieve(), ServerMessage::Ready);
    println!("Client sending message");
    client.send(ClientMessage::Jump);
    println!("Getting message from server");
    let server_message = client.recieve();
    dbg!(server_message);

    // dbg!(env!("CARGO_CRATE_NAME"));
    // dbg!(env!("CARGO_PKG_VERSION"));
    // dbg!(env!("CARGO_PKG_VERSION_MAJOR"));
    // dbg!(env!("CARGO_PKG_VERSION_MINOR"));
    // dbg!(env!("CARGO_PKG_VERSION_PATCH"));
    // dbg!(env!("CARGO_PKG_VERSION_PRE"));
}
