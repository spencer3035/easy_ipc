use std::sync::atomic::AtomicBool;

use crate::packet::{IpcMagicBytes, MagicBytes};
use signal_hook::{consts::*, iterator::Signals};

use {
    crate::{client::Client, connection::Connection, error::InitError, server::Server},
    interprocess::local_socket::{GenericFilePath, ListenerOptions, Name, Stream, prelude::*},
    serde::{Deserialize, Serialize},
    std::path::{Path, PathBuf},
};

/// Gets a sensible default name of socket according to your OS and a final path name.
///
/// On linux it would be something like `"/run/user/1000/<filename>"`.
/// ```
/// use easy_ipc::prelude::default_socket;
/// let my_socket = default_socket("myapp.socket");
/// ```
// TODO: Make more generic than &str?
pub fn default_socket<P>(filename: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let mut path = default_socket_path();
    path.push(filename);
    path
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
/// On linux it would be something like `"/run/user/1000/mycrate.socket"` if your crate name was
/// `mycrate`.
/// ```
/// use easy_ipc::socket_name;
/// let name = socket_name!();
/// ```
#[macro_export]
macro_rules! socket_name {
    () => {{
        let name = env!("CARGO_CRATE_NAME").to_string() + ".socket";
        ::easy_ipc::prelude::default_socket(&name)
    }};
}

static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);
static HANDLERS_SET: AtomicBool = AtomicBool::new(false);

/// A model for a Client Server IPC interface. Client messages are denoted by the generic `C` and
/// server messages are denoted by the generic `S`.
pub trait ClientServerModel<C, S, M = IpcMagicBytes>
where
    C: Serialize + for<'de> Deserialize<'de>,
    S: Serialize + for<'de> Deserialize<'de>,
    M: MagicBytes,
{
    /// The location that the socket will be stored. Can be overwriten if the default location is
    /// not desired. See [`socket_name`] and [`default_socket`] for easy method of creating a
    /// socket name.
    fn socket_path() -> PathBuf;

    /// Do not manually call this method.
    ///
    /// Sets up panic and signal handlers to automatically delete the socket file generated at
    /// [`ClientServerModel::socket_path`] when the program exists. This gets run when creating the
    /// first server via the [`ClientServerModel::server`] associated method.
    ///
    /// If you want to set up these handles yourself, you can overwrite this method to be empty.
    /// It is then your responsibilty to delete the socket file (or ensure the Server struct is
    /// dropped before the program exits).
    fn register_handlers() {
        // Protect against users calling this method
        let handle_lock = HANDLERS_SET.fetch_or(true, std::sync::atomic::Ordering::Relaxed);
        if handle_lock {
            return;
        }
        setup_handlers(Self::socket_path());
    }

    /// Make a new client, errors if unable to connect to server. Multiple clients can exist across
    /// threads and processes.
    fn client() -> Result<Client<C, S, M>, InitError> {
        let name = pathbuf_to_interprocess_name(Self::socket_path())?;
        let stream = Stream::connect(name).map_err(|e| InitError::FailedConnectingToSocket(e))?;
        let conn = Connection::new(stream);
        Ok(Client::new(conn))
    }

    /// Try to create a new server instance.
    ///
    /// Needs to be created before clients. Only one server can exist at a time.
    ///
    /// Returns various kinds of errors that could happend when trying to init a new server.
    fn server() -> Result<Server<S, C, M>, InitError> {
        let name = pathbuf_to_interprocess_name(Self::socket_path())?;
        let opts = ListenerOptions::new().name(name);
        Self::server_with_opts(opts)
    }

    /// Create a server given options.
    ///
    /// See: [`ClientServerModel::server`]
    fn server_with_opts(opts: ListenerOptions) -> Result<Server<S, C, M>, InitError> {
        let name = pathbuf_to_interprocess_name(Self::socket_path())?;
        let opts = opts.name(name);
        // Can fail for IO reasons
        let listener = opts.create_sync().map_err(|e| match e {
            // Server is already running on the socket or the cleanup of the file failed
            e if e.kind() == std::io::ErrorKind::AddrInUse => InitError::SocketAlreadyExists,
            e => InitError::FailedConnectingToSocket(e),
        })?;
        // Gaurentee that there is only one server running in the current process.
        let server_lock = SERVER_RUNNING.fetch_or(true, std::sync::atomic::Ordering::Relaxed);
        if server_lock {
            return Err(InitError::ServerAlreadyRunning);
        }
        // We need to setup handlers after creating the listener to avoid killing a running
        // server's socket. We also need to setup handlers after we have gaurenteed that a server
        // hasn't already been spawned to ensure we don't try to setup two instances of handlers.
        Self::register_handlers();
        Ok(Server::new(listener))
    }
}

/// Converts [`PathBuf`] to [`Name`] using consistent method
fn pathbuf_to_interprocess_name(path: PathBuf) -> Result<Name<'static>, InitError> {
    path.to_fs_name::<GenericFilePath>()
        .map_err(|e| InitError::FailedConnectingToSocket(e))
}

/// Trys to cleanup the socket file if it exists.
///
/// `Ok(true)` if the socket was sucessfully removed
/// `Ok(false)` if there wasn't a socket file to remove
/// `Err(_)` if the cleanup failed for an io reason.
fn remove_socket_file<P>(socket: P) -> Result<bool, std::io::Error>
where
    P: AsRef<Path>,
{
    if socket.as_ref().try_exists()? {
        std::fs::remove_file(socket)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Tries to clean up socket file and prints errors if it fails.
pub fn cleanup<P>(path: P)
where
    P: AsRef<Path>,
{
    match remove_socket_file(&path) {
        // Should be the usual case, program was exited while server was running, so we need to
        // clean up the socket
        Ok(true) => (),
        // Less common, the socket didn't exist so something bad might have happened before the
        // server was created.
        Ok(false) => (),
        // Bad, we failed deleting the socket file, this might lead to a zombie socket file or it
        // could be because of bad permissions
        Err(e) => eprintln!(
            "Couldn't clean up socket file {}: {e}",
            &path.as_ref().display()
        ),
    }
}

/// Handles os signals by calling the cleanup function
fn handle_os_signals<P>(path: P) -> Result<(), std::io::Error>
where
    P: AsRef<Path>,
{
    // What signals do we want to handle?
    let mut signals = Signals::new(TERM_SIGNALS)?;
    for sig in signals.forever() {
        cleanup(path);
        unsafe {
            libc::signal(sig, libc::SIG_DFL);
            libc::raise(sig);
        }
        // Failsafe exit in case the re-raise of the signals doesn't properly exit
        std::process::exit(1);
    }
    Ok(())
}

/// Sets up handlers to try and delete a given path upon panic and signals that ask to terminate
/// the process.
fn setup_handlers<P>(path: P)
where
    P: AsRef<Path>,
{
    // Handle panics, we do this first because the handling of OS errors thread might panic
    let default_panic_hook = std::panic::take_hook();
    let path_clone = path.as_ref().to_path_buf();
    std::panic::set_hook(Box::new(move |info| {
        cleanup(&path_clone);
        default_panic_hook(info)
    }));

    // Handle signals from the OS
    let path_clone = path.as_ref().to_path_buf();
    std::thread::spawn(move || {
        if let Err(e) = handle_os_signals(&path_clone) {
            panic!("Failed setting up signal handlers: {e}");
        }
    });
}
