use std::{marker::PhantomData, sync::atomic::AtomicBool};

use interprocess::local_socket::{GenericNamespaced, ToNsName};

use crate::handlers::setup_handlers;

use {
    crate::{client::Client, error::InitError, server::Server},
    interprocess::local_socket::{GenericFilePath, ListenerOptions, Name, Stream, prelude::*},
    serde::{Deserialize, Serialize},
    std::path::{Path, PathBuf},
};

static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Internal options that a server/client model can have
#[derive(Debug)]
pub(crate) struct OptionsRaw {
    pub(crate) socket_name: PathBuf,
    pub(crate) magic_bytes: Vec<u8>,
    pub(crate) disable_single_server_check: bool,
}

impl OptionsRaw {
    fn new<P>(namespace: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            socket_name: namespace.as_ref().to_path_buf(),
            magic_bytes: b"4242".to_vec(),
            disable_single_server_check: false,
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
    ///  See [`crate::namespace::namespace`] for easy methods of creating a namespace.
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
    pub fn magic_bytes<T>(mut self, magic_bytes: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        self.options_inner.magic_bytes = magic_bytes.into();
        self
    }

    /// NOT RECOMMENDED: Overwrite the default handling of panics and OS termination signals.
    ///
    /// The default implementation ensures there are no hiccups with starting new servers due to
    /// files getting left lying around. If you choose to overwrite this method you accept you will
    /// either need to do this yourself or deal with the issues associated with not doing it. In
    /// general, we assume in this library that you use the default implementation.
    ///
    /// The function passed into this gets called a maximum of one time if [`IpcModel::server`] is
    /// called.
    ///
    /// By default, we set up panic and signal handlers to automatically delete the socket file
    /// generated at the namespace set in [`ClientServerOptions`] so that when the program exists
    /// with Ctrl-c or a termination signal there are no issues with creating another server.
    ///
    /// It is recommended you look at the internal implementation of these handlers for a reference.
    /// and also understand how this library works under the hood before calling this method.
    pub fn handlers(mut self, hook: fn(&ClientServerModel<C, S>)) -> Self {
        // If you are here to look for the internal implementation, look at the definition for the
        // [`ClientServerOptions::new`] method to see what hook it sets by default.
        self.handler = hook;
        self
    }

    /// Allow multiple servers to run in a single process.
    ///
    /// By default, we use an atomic check to ensure only one [`Server`] is created in a process to
    /// protect against user errors, you can disable that check here if, for instance, you want to
    /// have multiple servers connected to different sockets running at the same time.
    pub fn disable_single_server_check(mut self) -> Self {
        self.options_inner.disable_single_server_check = true;
        self
    }

    /// Create a new client-server model with the given options
    pub fn create(self) -> ClientServerModel<C, S> {
        ClientServerModel::new(self)
    }
}

pub trait IpcModel {
    type ClientMsg: Serialize + for<'de> Deserialize<'de>;
    type ServerMsg: Serialize + for<'de> Deserialize<'de>;

    /// Generate a client/server model. See [`ClientServerOptions`] on how to create a new model.
    ///
    /// This function needs to be pure (have no side effects) to be sound. The client and the
    /// server need to agree on what the model looks like in order to communicate. This is
    /// partially enforced by the function taking no arguments. For simple use cases, use
    /// [`crate::ipc_model!`].
    fn model() -> Result<ClientServerModel<Self::ClientMsg, Self::ServerMsg>, InitError>;

    /// Make a new client, errors if unable to connect to server.
    ///
    /// Multiple clients can exist at the same time.
    ///
    /// This should not be implemented (in fact it is not possible).
    fn client() -> Result<Client<Self::ClientMsg, Self::ServerMsg>, InitError>
    where
        Self: Sized,
    {
        Self::model()?.client()
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
        Self::model()?.server()
    }
}

/// A model for a Client Server IPC interface. Client messages are denoted by the generic `C` and
/// server messages are denoted by the generic `S`.
///
/// The primary way to generate this is to use [`ClientServerOptions`] or a struct that implements
/// [`IpcModel`].
pub struct ClientServerModel<C, S>
where
    C: Serialize + for<'de> Deserialize<'de>,
    S: Serialize + for<'de> Deserialize<'de>,
{
    pub(crate) options: ClientServerOptions<C, S>,
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
        if server_lock && !self.options.options_inner.disable_single_server_check {
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
    let len = path.as_ref().iter().count();
    if len == 1 && GenericNamespaced::is_supported() {
        path.as_ref()
            .file_name()
            .ok_or(InitError::BadPath)?
            .to_owned()
            .to_ns_name::<GenericNamespaced>()
            .map_err(|e| InitError::FailedConnectingToSocket(e))
    } else {
        path.as_ref()
            .to_owned()
            .to_fs_name::<GenericFilePath>()
            .map_err(|e| InitError::FailedConnectingToSocket(e))
    }
}
