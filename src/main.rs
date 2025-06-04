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

/// Errors that can result from using a connection
#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionError {
    SerilizationFailed,
    DeserilizationFailed,
    WriteFailed,
    ReadFailed,
    UnexepctedEof,
}

/// Server errors
#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    IncomingConnectionFailed,
    CouldntOpenSocket,
}

/// Client errors
#[derive(Debug, PartialEq, Eq)]
pub enum ClientError {
    FailedConnectingToSocket,
}

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

struct MyModel;

impl ClientServerModel<ClientMessage, ServerMessage> for MyModel {
    type ClientImpl = MyClient;
    type ServerImpl = MyServer;

    fn socket_name() -> Name<'static> {
        socket_name()
    }

    /// Make a new client, errors if unable to connect to server
    fn client() -> Result<Self::ClientImpl, ClientError> {
        let name = Self::socket_name();
        let stream = Stream::connect(name).map_err(|_| ClientError::FailedConnectingToSocket)?;
        let conn = Connection::new(stream);
        Ok(MyClient { connection: conn })
    }

    /// Try to create a new server instance. Needs to be created before clients.
    fn server() -> Result<Self::ServerImpl, ServerError> {
        let name = Self::socket_name();
        let opts = ListenerOptions::new().name(name);
        // Can fail for IO reasons
        let listener = opts
            .create_sync()
            .map_err(|_| ServerError::CouldntOpenSocket)?;
        Ok(MyServer { listener })
    }
}

pub trait ClientServerModel<C, S> {
    type ClientImpl: Client<C, S>;
    type ServerImpl: Server<S, C>;

    fn socket_name() -> Name<'static>;
    /// Gets a client
    fn client() -> Result<Self::ClientImpl, ClientError>;
    /// Gets a server
    fn server() -> Result<Self::ServerImpl, ServerError>;
}

pub trait Client<T, R> {
    fn send(&mut self, msg: T) -> Result<(), ConnectionError>;
    fn receive(&mut self) -> Result<R, ConnectionError>;
}

/// Client that is able to connect to a server and send/receive messages
struct MyClient {
    connection: Connection<ClientMessage, ServerMessage>,
}

impl Client<ClientMessage, ServerMessage> for MyClient {
    /// Send a message to the server
    fn send(&mut self, msg: ClientMessage) -> Result<(), ConnectionError> {
        self.connection.send(msg)
    }

    /// Receive a message from the server
    fn receive(&mut self) -> Result<ServerMessage, ConnectionError> {
        self.connection.receive()
    }
}

/// A instance of a server
struct MyServer {
    listener: LocalSocketListener,
}

impl Server<ServerMessage, ClientMessage> for MyServer {
    fn connections(
        &self,
    ) -> impl Iterator<Item = Result<Connection<ServerMessage, ClientMessage>, ServerError>> {
        self.listener.incoming().map(|conn| {
            conn.map(Connection::new)
                .map_err(|_| ServerError::IncomingConnectionFailed)
        })
    }
}

pub trait Server<T, R> {
    /// Gets an infinite iterator over client connections
    fn connections(&self) -> impl Iterator<Item = Result<Connection<T, R>, ServerError>>;
}

/// Represents a connection that can send and receive messages
// S[end] and R[eceive]
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
    /// Make a new connection given a stream.
    // NOTE: This method should not be exposed publicly
    fn new(connection: Stream) -> Self {
        let connection = BufReader::new(connection);
        Connection {
            connection,
            _send: PhantomData,
            _receive: PhantomData,
        }
    }

    /// Send a message to the other end of the connection.
    pub fn send(&mut self, message: S) -> Result<(), ConnectionError> {
        let bytes =
            bitcode::serialize(&message).map_err(|_| ConnectionError::SerilizationFailed)?;
        let packet = Packet::new(bytes);
        let packet_bytes = packet.to_bytes();
        self.connection
            .get_mut()
            .write_all(&packet_bytes)
            .map_err(|_| ConnectionError::WriteFailed)?;
        Ok(())
    }

    /// Receive a message from the other end of the connection
    pub fn receive(&mut self) -> Result<R, ConnectionError> {
        let mut header: [u8; Header::LENGTH] = [0; Header::LENGTH];
        println!("Reading header");
        let nread = self
            .connection
            .read(&mut header)
            .map_err(|_| ConnectionError::ReadFailed)?;

        if nread != Header::LENGTH {
            debug_assert_eq!(
                nread,
                Header::LENGTH,
                "Couldn't read the header: {:?}",
                header
            );
            return Err(ConnectionError::UnexepctedEof);
        }
        let header =
            Header::parse_header(&header).map_err(|_| ConnectionError::DeserilizationFailed)?;
        let len = header.len;
        let mut data = vec![0; len as usize];
        let nread = self
            .connection
            .read(&mut data)
            .map_err(|_| ConnectionError::ReadFailed)?;
        debug_assert_eq!(nread, len as usize, "Didn't read enough data");
        if nread != len as usize {
            return Err(ConnectionError::UnexepctedEof);
        }
        bitcode::deserialize(&data).map_err(|_| ConnectionError::DeserilizationFailed)
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

const HEADER_MAGIC: &[u8; 4] = b"heyo";

#[derive(Debug, PartialEq, Eq)]
enum ParseHeaderError {
    MagicBytesMissing,
}

struct Header {
    len: u32,
}

impl Header {
    const LENGTH: usize = HEADER_MAGIC.len() + size_of::<u32>();

    fn create_header(data: &[u8]) -> Self {
        let len = data.len();
        assert!(len <= u32::MAX as usize);
        let len = len as u32;
        Header { len }
    }

    fn parse_header(bytes: &[u8; Header::LENGTH]) -> Result<Self, ParseHeaderError> {
        for (x, y) in HEADER_MAGIC.iter().zip(bytes.iter()) {
            if x != y {
                return Err(ParseHeaderError::MagicBytesMissing);
            }
        }
        let len = u32::from_le_bytes(bytes[4..Header::LENGTH].try_into().unwrap());
        Ok(Header { len })
    }

    fn to_bytes(self) -> Vec<u8> {
        let mut res = HEADER_MAGIC.to_vec();
        for val in self.len.to_le_bytes() {
            res.push(val);
        }
        res
    }
}

/// A packet to be sent over a socket
struct Packet {
    header: Header,
    bytes: Vec<u8>,
}

impl Packet {
    /// Make a new packet from data
    fn new(data: Vec<u8>) -> Self {
        assert!(data.len() <= u8::max as usize);
        let header = Header::create_header(&data);
        Packet {
            header,
            bytes: data,
        }
    }
    /// Convert packet to bytes
    fn to_bytes(mut self) -> Vec<u8> {
        let mut vec = self.header.to_bytes();
        vec.append(&mut self.bytes);
        vec
    }
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
