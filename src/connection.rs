use {
    crate::{
        error::ConnectionError,
        packet::{Header, IpcMagicBytes, MagicBytes, Packet},
    },
    interprocess::local_socket::Stream,
    serde::{Deserialize, Serialize},
    std::{
        io::{BufReader, prelude::*},
        marker::PhantomData,
    },
};

/// Represents a connection that can send and receive messages
// S[end] and R[eceive]
pub struct Connection<T, R, M = IpcMagicBytes>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
    M: MagicBytes,
{
    connection: BufReader<Stream>,
    _tx: PhantomData<T>,
    _rx: PhantomData<R>,
    _magic: PhantomData<M>,
}

impl<T, R, M> Connection<T, R, M>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
    M: MagicBytes,
{
    /// Make a new connection given a stream.
    // NOTE: This method should not be exposed publicly
    pub fn new(connection: Stream) -> Self {
        let connection = BufReader::new(connection);
        Connection {
            connection,
            _tx: PhantomData,
            _rx: PhantomData,
            _magic: PhantomData,
        }
    }

    /// Send a message to the other end of the connection.
    pub fn send(&mut self, message: T) -> Result<(), ConnectionError> {
        let bytes =
            bitcode::serialize(&message).map_err(|e| ConnectionError::SerilizationFailed(e))?;
        let packet: Packet<M> = Packet::new(bytes);
        let packet_bytes = packet.to_bytes();
        self.connection
            .get_mut()
            .write_all(&packet_bytes)
            .map_err(|e| ConnectionError::WriteFailed(e))?;
        Ok(())
    }

    /// Receive a message from the other end of the connection
    pub fn receive(&mut self) -> Result<R, ConnectionError> {
        let mut header = vec![0; Header::<M>::header_bytes()];
        let nread = self
            .connection
            .read(&mut header)
            .map_err(|e| ConnectionError::ReadFailed(e))?;

        if nread != Header::<M>::header_bytes() {
            return Err(ConnectionError::UnexepctedEof);
        }
        let header =
            Header::<M>::parse_header(&header).map_err(|_| ConnectionError::HeaderMismatch)?;
        let len = header.length();
        let mut data = vec![0; len as usize];
        let nread = self
            .connection
            .read(&mut data)
            .map_err(|e| ConnectionError::ReadFailed(e))?;
        debug_assert_eq!(nread, len as usize, "Didn't read enough data");
        if nread != len as usize {
            return Err(ConnectionError::UnexepctedEof);
        }
        bitcode::deserialize(&data).map_err(|e| ConnectionError::DeserilizationFailed(e))
    }
}
