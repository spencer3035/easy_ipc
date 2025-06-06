use {
    crate::{connection::Connection, error::ConnectionError, packet::MagicBytes},
    interprocess::local_socket::prelude::*,
    serde::{Deserialize, Serialize},
    std::marker::PhantomData,
};

/// A instance of a server
pub struct Server<T, R, M>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
    M: MagicBytes,
{
    listener: LocalSocketListener,
    _tx: PhantomData<T>,
    _rx: PhantomData<R>,
    _magic: PhantomData<M>,
}

impl<T, R, M> Server<T, R, M>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
    M: MagicBytes,
{
    /// Get a new Server listening on a socket
    pub(crate) fn new(listener: LocalSocketListener) -> Self {
        Self {
            listener,
            _tx: PhantomData,
            _rx: PhantomData,
            _magic: PhantomData,
        }
    }
    /// Create an iterator over all connections
    pub fn connections(
        &self,
    ) -> impl Iterator<Item = Result<Connection<T, R, M>, ConnectionError>> {
        self.listener.incoming().map(|conn| {
            conn.map(Connection::new)
                .map_err(|e| ConnectionError::IoError(e))
        })
    }
}
