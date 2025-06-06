use {
    crate::{connection::Connection, error::ConnectionError, packet::MagicBytes},
    serde::{Deserialize, Serialize},
    std::marker::PhantomData,
};

/// Client that is able to connect to a server and send/receive messages
pub struct Client<T, R, M>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
    M: MagicBytes,
{
    connection: Connection<T, R, M>,
    _tx: PhantomData<T>,
    _rx: PhantomData<R>,
    _magic: PhantomData<M>,
}

impl<T, R, M> Client<T, R, M>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
    M: MagicBytes,
{
    /// Create a new client given a connection
    pub(crate) fn new(connection: Connection<T, R, M>) -> Self {
        Self {
            connection,
            _tx: PhantomData,
            _rx: PhantomData,
            _magic: PhantomData,
        }
    }
    /// Send a message to the server
    pub fn send(&mut self, msg: T) -> Result<(), ConnectionError> {
        self.connection.send(msg)
    }

    /// Receive a message from the server
    pub fn receive(&mut self) -> Result<R, ConnectionError> {
        self.connection.receive()
    }
}
