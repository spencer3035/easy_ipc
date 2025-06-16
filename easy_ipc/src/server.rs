use {
    crate::{connection::Connection, error::ConnectionError, model::OptionsRaw},
    interprocess::local_socket::prelude::*,
    serde::{Deserialize, Serialize},
    std::{marker::PhantomData, sync::Arc},
};

/// A instance of a server
#[derive(Debug)]
pub struct Server<T, R>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    listener: LocalSocketListener,
    opts: Arc<OptionsRaw>,
    _tx: PhantomData<T>,
    _rx: PhantomData<R>,
}

impl<T, R> Server<T, R>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    /// Get a new Server listening on a socket
    pub(crate) fn new(listener: LocalSocketListener, opts: OptionsRaw) -> Self {
        let opts = Arc::new(opts);
        Self {
            listener,
            opts,
            _tx: PhantomData,
            _rx: PhantomData,
        }
    }
    /// Create an iterator over all connections
    pub fn connections(&self) -> impl Iterator<Item = Result<Connection<T, R>, ConnectionError>> {
        self.listener.incoming().map(|conn| {
            conn.map(|c| Connection::new(c, self.opts.clone()))
                .map_err(ConnectionError::InitError)
        })
    }
}
