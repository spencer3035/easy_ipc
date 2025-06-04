
use {
    crate::packet::{Header, Packet},
    interprocess::local_socket::Stream,
    serde::{Deserialize, Serialize},
    std::{
        io::{BufReader, prelude::*},
        marker::PhantomData,
    },
};

/// Errors that can result from using a connection
#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionError {
    SerilizationFailed,
    DeserilizationFailed,
    WriteFailed,
    ReadFailed,
    UnexepctedEof,
}

/// Represents a connection that can send and receive messages
// S[end] and R[eceive]
pub struct Connection<T, R> {
    connection: BufReader<Stream>,
    _send: PhantomData<T>,
    _receive: PhantomData<R>,
}

impl<S, R> Connection<S, R>
where
    S: Serialize,
    R: for<'de> Deserialize<'de>,
{
    /// Make a new connection given a stream.
    // NOTE: This method should not be exposed publicly
    pub fn new(connection: Stream) -> Self {
        let connection = BufReader::new(connection);
        Connection {
            connection,
            _send: PhantomData,
            _receive: PhantomData,
        }
    }

    /// Send a message to the other end of the connection.
    pub fn send(&mut self, message: S) -> Result<(), ConnectionError> {
        let bytes =
            bitcode::serialize(&message).map_err(|_| ConnectionError::SerilizationFailed)?;
        let packet = Packet::new(bytes);
        let packet_bytes = packet.to_bytes();
        self.connection
            .get_mut()
            .write_all(&packet_bytes)
            .map_err(|_| ConnectionError::WriteFailed)?;
        Ok(())
    }

    /// Receive a message from the other end of the connection
    pub fn receive(&mut self) -> Result<R, ConnectionError> {
        let mut header: [u8; Header::LENGTH] = [0; Header::LENGTH];
        println!("Reading header");
        let nread = self
            .connection
            .read(&mut header)
            .map_err(|_| ConnectionError::ReadFailed)?;

        if nread != Header::LENGTH {
            debug_assert_eq!(
                nread,
                Header::LENGTH,
                "Couldn't read the header: {:?}",
                header
            );
            return Err(ConnectionError::UnexepctedEof);
        }
        let header =
            Header::parse_header(&header).map_err(|_| ConnectionError::DeserilizationFailed)?;
        let len = header.length();
        let mut data = vec![0; len as usize];
        let nread = self
            .connection
            .read(&mut data)
            .map_err(|_| ConnectionError::ReadFailed)?;
        debug_assert_eq!(nread, len as usize, "Didn't read enough data");
        if nread != len as usize {
            return Err(ConnectionError::UnexepctedEof);
        }
        bitcode::deserialize(&data).map_err(|_| ConnectionError::DeserilizationFailed)
    }
}
