
use {
    crate::client::{Client, ClientError},
    crate::connection::Connection,
    crate::server::{Server, ServerError},
    interprocess::local_socket::{ListenerOptions, Name, Stream, prelude::*},
    serde::{Deserialize, Serialize},
};
pub trait ClientServerModel<C, S>
where
    C: Serialize + for<'de> Deserialize<'de>,
    S: Serialize + for<'de> Deserialize<'de>,
{
    fn socket_name() -> Name<'static>;

    /// Make a new client, errors if unable to connect to server
    fn client() -> Result<Client<C, S>, ClientError> {
        let name = Self::socket_name();
        let stream = Stream::connect(name).map_err(|_| ClientError::FailedConnectingToSocket)?;
        let conn = Connection::new(stream);
        Ok(Client::new(conn))
    }

    /// Try to create a new server instance. Needs to be created before clients.
    fn server() -> Result<Server<S, C>, ServerError> {
        let name = Self::socket_name();
        let opts = ListenerOptions::new().name(name);
        // Can fail for IO reasons
        let listener = opts
            .create_sync()
            .map_err(|_| ServerError::CouldntOpenSocket)?;
        Ok(Server::new(listener))
    }
}
