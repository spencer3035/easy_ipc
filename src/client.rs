use {
    crate::{connection::Connection, error::ConnectionError, model::OptionsRaw},
    interprocess::local_socket::Stream,
    serde::{Deserialize, Serialize},
    std::{marker::PhantomData, sync::Arc},
};

/// Client that is able to connect to a server and send/receive messages
pub struct Client<T, R>
where
    T: Serialize + for<'de> Deserialize<'de>,
    R: Serialize + for<'de> Deserialize<'de>,
{
    connection: Connection<T, R>,
    _tx: PhantomData<T>,
    _rx: PhantomData<R>,
}

impl<T, R> Client<T, R>
where
    T: Serialize + for<'de> Deserialize<'de>,
    R: Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new client given a connection
    pub(crate) fn new(opts: OptionsRaw, stream: Stream) -> Self {
        //conn.map(|c| Connection::new(c, &self.opts))
        let opts = Arc::new(opts);
        let connection = Connection::new(stream, opts);
        Self {
            connection,
            _tx: PhantomData,
            _rx: PhantomData,
        }
    }
    /// Send a message to the server
    pub fn send(&mut self, msg: T) -> Result<(), ConnectionError> {
        // self.connection.send(msg, &self.model)
        self.connection.send(msg)
    }

    /// Receive a message from the server
    pub fn receive(&mut self) -> Result<R, ConnectionError> {
        self.connection.receive()
    }
}
