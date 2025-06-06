use {
    crate::{connection::Connection, error::ConnectionError},
    serde::{Deserialize, Serialize},
    std::marker::PhantomData,
};

/// Client that is able to connect to a server and send/receive messages
pub struct Client<T, R>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    connection: Connection<T, R>,
    _tx: PhantomData<T>,
    _rx: PhantomData<R>,
}

impl<T, R> Client<T, R>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    /// Create a new client given a connection
    pub(crate) fn new(connection: Connection<T, R>) -> Self {
        Self {
            connection,
            _tx: PhantomData,
            _rx: PhantomData,
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
