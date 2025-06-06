use std::fmt::Display;

/// Errors that can result from using a connection
#[derive(Debug)]
#[non_exhaustive]
pub enum ConnectionError {
    HeaderMismatch,
    UnexepctedEof,
    SerilizationFailed(bitcode::Error),
    DeserilizationFailed(bitcode::Error),
    WriteFailed(std::io::Error),
    ReadFailed(std::io::Error),
    IoError(std::io::Error),
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

#[derive(Debug)]
#[non_exhaustive]
pub enum InitError {
    /// Server is already connected an running, can't create a new one.
    ServerAlreadyRunning,
    /// Generic IO error like permission denied
    FailedConnectingToSocket(std::io::Error),
    /// Specific to servers trying to connect to already existing sockets. This can happen if
    /// the a server is already running or it exited in a non-graceful way.
    SocketAlreadyExists,
}
