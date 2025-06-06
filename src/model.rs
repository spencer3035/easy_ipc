use {
    crate::{client::Client, connection::Connection, error::InitError, server::Server},
    interprocess::local_socket::{GenericFilePath, ListenerOptions, Stream, prelude::*},
    serde::{Deserialize, Serialize},
    std::path::PathBuf,
};

pub type Name = interprocess::local_socket::Name<'static>;

/// Gets a sensible default name of socket according to your OS and a final path name.
///
/// On linux it would be something like `"/run/user/1000/<filename>"`.
///
/// Will return an error if the filename cannot be converted into an [`std::ffi::OsStr`].
/// ```
/// use easy_ipc::prelude::default_socket;
/// let my_socket = default_socket("myapp.socket").unwrap();
/// ```
// TODO: Make more generic than &str?
pub fn default_socket(filename: &str) -> Result<Name, std::io::Error> {
    let mut path = default_socket_path();
    path.push(filename);
    path.to_fs_name::<GenericFilePath>()
}

// TODO: Implement for "windows" "macos" "ios" "linux" "android" "freebsd" "dragonfly" "openbsd" "netbsd"
// "none" should always fail, we need an os to do what this crate does
#[cfg(not(target_os = "linux"))]
fn default_socket_path() -> PathBuf {
    panic!("platform not supported")
}

#[cfg(target_os = "linux")]
fn default_socket_path() -> PathBuf {
    let mut p = PathBuf::new();
    p.push("/run");
    p.push("user");
    p.push(users::get_current_uid().to_string());
    p
}

/// Generates a socket based on your crate name.
///
/// On linux it would be something like `"/run/user/1000/mycrate.socket"`.
/// ```
/// use easy_ipc::socket_name;
/// let name = socket_name!().unwrap();
/// ```
#[macro_export]
macro_rules! socket_name {
    () => {
        easy_ipc::prelude::default_socket(&(env!("CARGO_CRATE_NAME").to_string() + ".socket"))
    };
}

/// A model for a Client Server IPC interface. Client messages are denoted by the generic `C` and
/// server messages are denoted by the generic `S`.
pub trait ClientServerModel<C, S>
where
    C: Serialize + for<'de> Deserialize<'de>,
    S: Serialize + for<'de> Deserialize<'de>,
{
    /// The location that the socket will be stored. Can be overwriten if the default location is
    /// not desired. See [`default_socket_name`].
    fn socket_name() -> Name;

    /// Make a new client, errors if unable to connect to server. Should not be implemented
    /// manually.
    fn client() -> Result<Client<C, S>, InitError> {
        let name = Self::socket_name();
        let stream = Stream::connect(name).map_err(|e| InitError::FailedConnectingToSocket(e))?;
        let conn = Connection::new(stream);
        Ok(Client::new(conn))
    }

    /// Try to create a new server instance. Needs to be created before clients. Should not be
    /// implemented manually
    fn server() -> Result<Server<S, C>, InitError> {
        let name = Self::socket_name();
        let opts = ListenerOptions::new().name(name);
        // Can fail for IO reasons
        let listener = opts.create_sync().map_err(|e| match e {
            // Server is already running on the socket or the cleanup of the file failed
            e if e.kind() == std::io::ErrorKind::AddrInUse => InitError::SocketAlreadyExists,
            e => InitError::FailedConnectingToSocket(e),
        })?;
        Ok(Server::new(listener))
    }
}
