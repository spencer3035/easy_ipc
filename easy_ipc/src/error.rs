use std::fmt::Display;

/// Errors that can result from using a connection
#[derive(Debug)]
#[non_exhaustive]
pub enum ConnectionError {
    /// Header magic bytes did not match or there were not enough bytes to for
    HeaderMismatch,
    /// Packet was larger that `usize` bytes, this may be due to a malformed header.
    PacketTooLarge,
    /// Not enough bytes to read the packet
    UnexepctedEof,
    /// Failed serializing a struct
    SerilizationFailed(bitcode::Error),
    /// Failed de-serializing a struct
    DeserilizationFailed(bitcode::Error),
    /// Failed writing to the connection
    WriteFailed(std::io::Error),
    /// Failed reading from the connection
    ReadFailed(std::io::Error),
    /// Failing initializing the connection
    InitError(std::io::Error),
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::SerilizationFailed(e) => write!(f, "serilization fail {e}"),
            ConnectionError::DeserilizationFailed(e) => write!(f, "deserilization fail {e}"),
            ConnectionError::HeaderMismatch => {
                write!(f, "header didn't match, likely version incompatability")
            }
            ConnectionError::WriteFailed(e) => writeln!(f, "write failed, {e}"),
            _ => todo!(),
        }
    }
}

/// And error that occurred during initialization of a server or client.
#[derive(Debug)]
#[non_exhaustive]
pub enum InitError {
    /// Wasn't able to construct a namespace
    FailedGettingNamespace,
    /// Path was not formatted correctly either for the OS or the socket type.
    BadPath,
    /// Server is already connected an running, can't create a new one.
    ServerAlreadyRunning,
    /// Generic IO error like permission denied
    FailedConnectingToSocket(std::io::Error),
    /// Specific to servers trying to connect to already existing sockets. This can happen if a
    /// server is already running or it exited in a non-graceful way.
    SocketAlreadyExists,
}
