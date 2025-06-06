const HEADER_MAGIC: &[u8; 4] = b"heyo";

#[derive(Debug, PartialEq, Eq)]
pub enum ParseHeaderError {
    MagicBytesMissing,
}

pub struct Header {
    len: u32,
}

// Some helpful env vars for the future
// dbg!(env!("CARGO_PKG_VERSION"));
// dbg!(env!("CARGO_PKG_VERSION_MAJOR"));
// dbg!(env!("CARGO_PKG_VERSION_MINOR"));
// dbg!(env!("CARGO_PKG_VERSION_PATCH"));
// dbg!(env!("CARGO_PKG_VERSION_PRE"));
impl Header {
    pub const LENGTH: usize = HEADER_MAGIC.len() + size_of::<u32>();

    pub fn length(&self) -> usize {
        self.len as usize
    }
    fn create_header(data: &[u8]) -> Self {
        let len = data.len();
        assert!(len <= u32::MAX as usize);
        let len = len as u32;
        Header { len }
    }

    pub fn parse_header(bytes: &[u8; Header::LENGTH]) -> Result<Self, ParseHeaderError> {
        for (x, y) in HEADER_MAGIC.iter().zip(bytes.iter()) {
            if x != y {
                return Err(ParseHeaderError::MagicBytesMissing);
            }
        }
        let len = u32::from_le_bytes(bytes[4..Header::LENGTH].try_into().unwrap());
        Ok(Header { len })
    }

    fn to_bytes(self) -> Vec<u8> {
        let mut res = HEADER_MAGIC.to_vec();
        for val in self.len.to_le_bytes() {
            res.push(val);
        }
        res
    }
}

/// A packet to be sent over a socket
pub struct Packet {
    header: Header,
    bytes: Vec<u8>,
}

impl Packet {
    /// Make a new packet from data
    pub fn new(data: Vec<u8>) -> Self {
        assert!(data.len() <= u8::max as usize);
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
