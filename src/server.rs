use {
    crate::{connection::Connection, error::ConnectionError},
    interprocess::local_socket::prelude::*,
    serde::{Deserialize, Serialize},
    std::marker::PhantomData,
};

/// A instance of a server
pub struct Server<T, R>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    listener: LocalSocketListener,
    _tx: PhantomData<T>,
    _rx: PhantomData<R>,
}

impl<T, R> Server<T, R>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    /// Get a new Server listening on a socket
    pub(crate) fn new(listener: LocalSocketListener) -> Self {
        Self {
            listener,
            _tx: PhantomData,
            _rx: PhantomData,
        }
    }
    /// Create an iterator over all connections
    pub fn connections(&self) -> impl Iterator<Item = Result<Connection<T, R>, ConnectionError>> {
        self.listener.incoming().map(|conn| {
            conn.map(Connection::new)
                .map_err(|e| ConnectionError::IoError(e))
        })
    }
}
