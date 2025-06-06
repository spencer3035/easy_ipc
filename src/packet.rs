use std::marker::PhantomData;

const HEADER_MAGIC: &[u8; 4] = b"heyo";

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ParseHeaderError {
    MagicBytesMissing,
}

pub(crate) struct Header<M>
where
    M: MagicBytes,
{
    len: u32,
    _marker: PhantomData<M>,
}

pub struct IpcMagicBytes;
impl MagicBytes for IpcMagicBytes {
    fn magic_bytes() -> &'static [u8] {
        b"v010"
    }
}

// TODO: Implement this once we have better testing in place
pub trait MagicBytes {
    /// Magic bytes at the beginning of the header to check for correctness.
    ///
    /// This should not change, if const trait functions existed, this would be annotated with it.
    fn magic_bytes() -> &'static [u8];
}

// Some helpful env vars for the future
// dbg!(env!("CARGO_PKG_VERSION"));
// dbg!(env!("CARGO_PKG_VERSION_MAJOR"));
// dbg!(env!("CARGO_PKG_VERSION_MINOR"));
// dbg!(env!("CARGO_PKG_VERSION_PATCH"));
// dbg!(env!("CARGO_PKG_VERSION_PRE"));
impl<M> Header<M>
where
    M: MagicBytes,
{
    /// The length of the header in bytes
    pub(crate) fn header_bytes() -> usize {
        M::magic_bytes().len() + size_of::<u32>()
    }

    /// The length of the data portion of the packet.
    pub(crate) fn length(&self) -> usize {
        self.len as usize
    }

    /// Create a header given some data
    fn create_header(data: &[u8]) -> Self {
        let len = data.len();
        assert!(len <= u32::MAX as usize);
        let len = len as u32;
        Header {
            len,
            _marker: PhantomData,
        }
    }

    /// Parse header and check that magic bytes are correct
    pub fn parse_header(bytes: &[u8]) -> Result<Self, ParseHeaderError> {
        if bytes.len() != Self::header_bytes() {
            return Err(ParseHeaderError::MagicBytesMissing);
        }
        for (x, y) in HEADER_MAGIC.iter().zip(bytes.iter()) {
            if x != y {
                return Err(ParseHeaderError::MagicBytesMissing);
            }
        }
        let len = u32::from_le_bytes(
            bytes[M::magic_bytes().len()..Self::header_bytes()]
                .try_into()
                .unwrap(),
        );
        Ok(Header {
            len,
            _marker: PhantomData,
        })
    }

    /// Convert the header to bytes
    fn to_bytes(self) -> Vec<u8> {
        let mut res = HEADER_MAGIC.to_vec();
        for val in self.len.to_le_bytes() {
            res.push(val);
        }
        res
    }
}

/// A packet to be sent over a socket
pub struct Packet<M>
where
    M: MagicBytes,
{
    header: Header<M>,
    bytes: Vec<u8>,
}

impl<M> Packet<M>
where
    M: MagicBytes,
{
    /// Make a new packet from data
    pub fn new(data: Vec<u8>) -> Self {
        let header = Header::create_header(&data);
        Packet {
            header,
            bytes: data,
        }
    }
    /// Convert packet to bytes
    pub fn to_bytes(mut self) -> Vec<u8> {
        let mut vec = self.header.to_bytes();
        vec.append(&mut self.bytes);
        vec
    }
}
