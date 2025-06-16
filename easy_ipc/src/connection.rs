use {
    crate::{error::ConnectionError, model::OptionsRaw},
    interprocess::local_socket::Stream,
    serde::{Deserialize, Serialize},
    std::{
        io::{BufReader, prelude::*},
        marker::PhantomData,
        sync::Arc,
    },
};

/// Represents a connection that can send and receive messages
// S[end] and R[eceive]
#[derive(Debug)]
pub struct Connection<T, R>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    connection: BufReader<Stream>,
    opts: Arc<OptionsRaw>,
    _tx: PhantomData<T>,
    _rx: PhantomData<R>,
}

impl<T, R> Connection<T, R>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    /// Make a new connection given a stream.
    // NOTE: This method should not be exposed publicly
    pub(crate) fn new(stream: Stream, opts: Arc<OptionsRaw>) -> Self {
        let connection = BufReader::new(stream);
        Connection {
            connection,
            opts,
            _tx: PhantomData,
            _rx: PhantomData,
        }
    }

    /// Send a message to the other end of the connection.
    pub fn send(&mut self, message: T) -> Result<(), ConnectionError> {
        let bytes =
            bitcode::serialize(&message).map_err(ConnectionError::SerilizationFailed)?;
        let packet_bytes = self.make_packet(bytes);
        self.connection
            .get_mut()
            .write_all(&packet_bytes)
            .map_err(ConnectionError::WriteFailed)?;
        Ok(())
    }

    /// Receive a message from the other end of the connection
    pub fn receive(&mut self) -> Result<R, ConnectionError> {
        let header_len = self.header_length();
        let mut header = vec![0; header_len];
        let nread = self
            .connection
            .read(&mut header)
            .map_err(ConnectionError::ReadFailed)?;

        if nread != self.header_length() {
            // TODO: This usually gets hit when the server closes and a client tries to read from it. Maybe check for 0 and report a different error?
            return Err(ConnectionError::UnexepctedEof);
        }
        let data_len = self.parse_header(&header).map_err(|e| match e {
            ParseHeaderError::NotEnoughBytes => ConnectionError::UnexepctedEof,
            ParseHeaderError::PacketTooLarge => ConnectionError::PacketTooLarge,
            ParseHeaderError::MagicBytesMissing => ConnectionError::HeaderMismatch,
        })?;

        let mut data = vec![0; data_len];
        let nread = self
            .connection
            .read(&mut data)
            .map_err(ConnectionError::ReadFailed)?;
        if nread != data_len {
            return Err(ConnectionError::UnexepctedEof);
        }
        bitcode::deserialize(&data).map_err(ConnectionError::DeserilizationFailed)
    }

    fn make_packet(&self, data: Vec<u8>) -> Vec<u8> {
        let mut header = self.gen_header(&data);
        let mut packet = data;
        header.append(&mut packet);
        header
    }

    fn gen_header(&self, data: &[u8]) -> Vec<u8> {
        let mut res = self.opts.magic_bytes.to_vec();
        // Assumes u128 targets don't exist
        let len: u64 = data.len() as u64;
        for val in len.to_le_bytes() {
            res.push(val);
        }
        res
    }

    fn header_length(&self) -> usize {
        self.opts.magic_bytes.len() + size_of::<u64>()
    }

    fn parse_header(&self, bytes: &[u8]) -> Result<usize, ParseHeaderError> {
        if bytes.len() < self.header_length() {
            return Err(ParseHeaderError::NotEnoughBytes);
        }
        for (x, y) in self.opts.magic_bytes.iter().zip(bytes.iter()) {
            if x != y {
                return Err(ParseHeaderError::MagicBytesMissing);
            }
        }
        let len = u64::from_le_bytes(
            bytes[self.opts.magic_bytes.len()..self.header_length()]
                .try_into()
                .unwrap(),
        );

        if (usize::MAX as u64) < len {
            return Err(ParseHeaderError::PacketTooLarge);
        }

        Ok(len as usize)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ParseHeaderError {
    NotEnoughBytes,
    MagicBytesMissing,
    PacketTooLarge,
}
