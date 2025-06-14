use std::{marker::PhantomData, sync::atomic::AtomicBool};

use signal_hook::{consts::*, iterator::Signals};

use {
    crate::{client::Client, error::InitError, server::Server},
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
        $crate::prelude::default_socket(&name)
    }};
}

static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub(crate) struct OptionsRaw {
    pub(crate) socket_name: PathBuf,
    pub(crate) magic_bytes: &'static [u8],
}

impl OptionsRaw {
    fn new<P>(namespace: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            socket_name: namespace.as_ref().to_path_buf(),
            magic_bytes: b"4242",
        }
    }
}

pub struct ClientServerOptions<C, S>
where
    C: Serialize + for<'de> Deserialize<'de>,
    S: Serialize + for<'de> Deserialize<'de>,
{
    pub(crate) options_inner: OptionsRaw,
    pub(crate) handler: fn(&ClientServerModel<C, S>),
    _client: PhantomData<C>,
    _server: PhantomData<S>,
}

/// A model for a Client Server IPC interface. Client messages are denoted by the generic `C` and
/// server messages are denoted by the generic `S`.
impl<C, S> ClientServerOptions<C, S>
where
    C: Serialize + for<'de> Deserialize<'de>,
    S: Serialize + for<'de> Deserialize<'de>,
{
    /// Creates a model in the given namespace with the default options.
    ///
    /// Currently the namespace is equivalent to the name of the socket. See [`socket_name`] and
    /// [`default_socket`] for easy methods of creating a socket name.
    pub fn new<P>(namespace: P) -> Self
    where
        P: AsRef<Path>,
    {
        let options_inner = OptionsRaw::new(namespace);
        Self {
            options_inner,
            handler: |model| {
                setup_handlers(model);
            },
            _client: PhantomData,
            _server: PhantomData,
        }
    }

    /// NOT RECOMMENDED: Set the magic bytes used in the packets to validate client and server
    /// methods.
    ///
    /// Changing this value will break compatibility for previous versions of your program. Both
    /// the client and the server need to agree on this value for messages to be passed back and
    /// forth.
    pub fn magic_bytes(mut self, magic_bytes: &'static [u8]) -> Self {
        self.options_inner.magic_bytes = magic_bytes;
        self
    }

    /// NOT RECOMMENDED: Overwrite the default handling of panics and OS termination signals.
    ///
    /// The default implementation ensures there are no hiccups with starting new servers due to
    /// files getting left lying around. If you choose to overwrite this method you accept you will
    /// either need to do this yourself or deal with the issues associated with not doing it. In
    /// general, we assume in this library that you use the default implementation.
    ///
    /// The function passed into this gets called a maximum of one time if
    /// [`ClientServerModel::server`] is called.
    ///
    /// By default, we set up panic and signal handlers to automatically delete the socket file
    /// generated at the namespace set in [`ClientServerOptions`] so that when the program exists with Ctrl-c or a
    /// termination signal there are no issues with creating another server.
    ///
    /// It is recommended you look at or use [`cleanup`] and also understand how this library works
    /// under the hood before calling this method.
    pub fn handlers(mut self, hook: fn(&ClientServerModel<C, S>)) -> Self {
        self.handler = hook;
        self
    }

    // Create a new client-server model with the given options
    pub fn create(self) -> ClientServerModel<C, S> {
        ClientServerModel::new(self)
    }
}

pub trait Model {
    type ClientMsg: Serialize + for<'de> Deserialize<'de>;
    type ServerMsg: Serialize + for<'de> Deserialize<'de>;

    /// Generate a client/server model. See [`ClientServerOptions`] on how to create a new model.
    ///
    /// This function needs to be pure (have no side effects) to be sound. The client and the
    /// server need to agree on what the model looks like in order to communicate. This is
    /// partially enforced by the function taking no arguments.
    fn model() -> ClientServerModel<Self::ClientMsg, Self::ServerMsg>;

    /// Make a new client, errors if unable to connect to server.
    ///
    /// Multiple clients can exist at the same time.
    ///
    /// This should not be implemented (in fact it is not possible).
    fn client() -> Result<Client<Self::ClientMsg, Self::ServerMsg>, InitError>
    where
        Self: Sized,
    {
        Self::model().client()
    }

    /// Try to create a new server instance.
    ///
    /// Needs to be created before clients. Only one server can exist at a time on a given host.
    ///
    /// This should not be implemented (in fact it is not possible).
    fn server() -> Result<Server<Self::ServerMsg, Self::ClientMsg>, InitError>
    where
        Self: Sized,
    {
        Self::model().server()
    }
}

/// A model for a Client Server IPC interface. Client messages are denoted by the generic `C` and
/// server messages are denoted by the generic `S`.
///
/// The primary way to generate this is to use [`ClientServerOptions`] or a struct that implements
/// [`Model`].
pub struct ClientServerModel<C, S>
where
    C: Serialize + for<'de> Deserialize<'de>,
    S: Serialize + for<'de> Deserialize<'de>,
{
    options: ClientServerOptions<C, S>,
    _client: PhantomData<C>,
    _server: PhantomData<S>,
}

/// A model for a Client Server IPC interface. Client messages are denoted by the generic `C` and
/// server messages are denoted by the generic `S`.
///
/// This should not be interacted with directly, you should instead use [`Model::client`] and
/// [`Model::server`] to create model and server instances.
impl<C, S> ClientServerModel<C, S>
where
    C: Serialize + for<'de> Deserialize<'de>,
    S: Serialize + for<'de> Deserialize<'de>,
{
    fn new(options: ClientServerOptions<C, S>) -> Self {
        ClientServerModel {
            options,
            _client: PhantomData,
            _server: PhantomData,
        }
    }
    /// Make a new client, errors if unable to connect to server. Multiple clients can exist across
    /// threads and processes.
    fn client(self) -> Result<Client<C, S>, InitError> {
        let name = pathbuf_to_interprocess_name(&self.options.options_inner.socket_name)?;
        let stream = Stream::connect(name).map_err(|e| InitError::FailedConnectingToSocket(e))?;
        Ok(Client::new(self.options.options_inner, stream))
    }

    /// Try to create a new server instance.
    ///
    /// Needs to be created before clients. Only one server can exist at a time.
    ///
    /// Returns various kinds of errors that could happend when trying to init a new server.
    fn server(self) -> Result<Server<S, C>, InitError> {
        let opts = ListenerOptions::new();
        self.server_with_opts(opts)
    }

    /// Get a reference to the internal options
    #[cfg(test)]
    pub(crate) fn options(&self) -> &OptionsRaw {
        &self.options.options_inner
    }

    /// Create a server given options.
    ///
    /// See: [`ClientServerModel::server`]
    fn server_with_opts(self, opts: ListenerOptions) -> Result<Server<S, C>, InitError> {
        let name = pathbuf_to_interprocess_name(&self.options.options_inner.socket_name)?;
        let opts = opts.name(name);
        // Can fail for IO reasons
        let listener = opts.create_sync().map_err(|e| match e {
            // Server is already running on the socket or the cleanup of the file failed
            e if e.kind() == std::io::ErrorKind::AddrInUse => InitError::SocketAlreadyExists,
            e => InitError::FailedConnectingToSocket(e),
        })?;
        // Gaurentee that there is only one server running in the current process.
        let server_lock = SERVER_RUNNING.fetch_or(true, std::sync::atomic::Ordering::Relaxed);
        // TODO: This prevents two servers from running, even if they are on different sockets,
        // this is probably not desired, maybe we could have an option to disable this check?
        if server_lock {
            // Skip for testing so we can test lots of servers
            #[cfg(not(test))]
            return Err(InitError::ServerAlreadyRunning);
        }
        // We need to setup handlers after creating the listener to avoid killing a running
        // server's socket. We also need to setup handlers after we have gaurenteed that a server
        // hasn't already been spawned to ensure we don't try to setup two instances of handlers.
        (self.options.handler)(&self);
        Ok(Server::new(listener, self.options.options_inner))
    }
}

/// Converts [`PathBuf`] to [`Name`] using consistent method
fn pathbuf_to_interprocess_name<'a, P>(path: P) -> Result<Name<'a>, InitError>
where
    P: AsRef<Path> + 'a,
{
    path.as_ref()
        .to_owned()
        .to_fs_name::<GenericFilePath>()
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

/// Tries to clean up a given file and prints errors if it fails.
///
/// Meant to be used in handling panics and signals sent to kill the program.
pub fn cleanup<P>(path: P)
where
    P: AsRef<Path>,
{
    match remove_socket_file(&path) {
        // Should be the usual case, program was exited while server was running, so we need to
        // clean up the socket
        Ok(true) => (),
        // Less common, the either the server was dropped or something bad might have happened
        // before the server was created.
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
    // Handle all term signals
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
fn setup_handlers<C, S>(model: &ClientServerModel<C, S>)
where
    C: Serialize + for<'de> Deserialize<'de>,
    S: Serialize + for<'de> Deserialize<'de>,
{
    let path = &model.options.options_inner.socket_name;

    // Handle panics, we do this first because the handling of OS errors thread might panic
    let default_panic_hook = std::panic::take_hook();
    let path_clone = path.clone();
    std::panic::set_hook(Box::new(move |info| {
        cleanup(&path_clone);
        default_panic_hook(info)
    }));

    // Handle signals from the OS
    let path_clone = path.clone();
    std::thread::spawn(move || {
        if let Err(e) = handle_os_signals(&path_clone) {
            panic!("Failed setting up signal handlers: {e}");
        }
        panic!("Stopped handling signals.");
    });
}
