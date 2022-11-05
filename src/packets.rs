use std::{str::{self, Utf8Error}, ops::Not};

use anyhow::Context;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PacketParseError {
    #[error("Packet didn't have enough bytes")]
    IncompletePacket,
    #[error("Packet didn't contain data that could be parsed to a String")]
    FilenameParseError
}

#[derive(Debug)]
pub enum Packet {
    Header(Header),
    Data(Data)
}

impl Packet {
    // An alternative from Wgaffa@Twitch that is more Haskell-like:
    //  bytes.is_empty().not().then(|| bytes[0] % 2 == 0).ok_or(PacketParseError::IncompletePacket)
    const fn is_header(bytes: &[u8]) -> Result<bool, PacketParseError> {
        if bytes.is_empty() {
            return Err(PacketParseError::IncompletePacket)
        }
        Ok(bytes[0] % 2 == 0)
    }

    #[must_use]
    pub const fn file_id(&self) -> u8 {
        match self {
            Self::Header(header) => header.file_id,
            Self::Data(data) => data.file_id
        }
    }
}

impl TryFrom<&[u8]> for Packet {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        if Self::is_header(bytes)? {
            Ok(Self::Header(bytes.try_into()?))
        } else {
            Ok(Self::Data(bytes.try_into()?))
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Header {
    pub(crate) file_id: u8,
    pub(crate) file_name: String
}

impl From<Utf8Error> for PacketParseError {
    fn from(_: Utf8Error) -> Self {
        Self::FilenameParseError
    }
}

// TODO: Look at how we could use the zerocopy crate to automagic
//   some of this conversion.
impl TryFrom<&[u8]> for Header {
    type Error = anyhow::Error; // PacketParseError;

    /// Convert the given byte array slice to a header packet.
    /// This assumes
    ///   * All the bytes in the given slice are
    ///     used (i.e., there are no "empty" or unused bytes at the
    ///     end)
    ///   * There are at least 3 bytes (the minimal size for a header packet)
    ///   * This is actually a header packet (i.e., the first byte is even)
    ///   * Bytes 2.. can be parsed as a String  
    fn try_from(bytes: &[u8]) -> Result<Self, anyhow::Error> { // PacketParseError> {
        if bytes.len() < 3 {
            return Err(anyhow::Error::from(PacketParseError::IncompletePacket))
        }
        assert!(Packet::is_header(bytes)?, "expected a header packet but first byte was not even");
        let file_id = bytes[1];
        let file_name 
            = str::from_utf8(&bytes[2..])
                .context(format!(""))?
                .to_string();

        Ok(Self { file_id, file_name })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Data {
    pub(crate) file_id: u8,
    pub(crate) packet_number: u16,
    pub(crate) is_last_packet: bool,
    pub(crate) data: Vec<u8>
}

impl TryFrom<&[u8]> for Data {
    type Error = PacketParseError;

    /// Convert the given byte array slice to a header packet.
    /// This assumes
    ///   * All the bytes in the given slice are
    ///     used (i.e., there are no "empty" or unused bytes at the
    ///     end)
    ///   * There are at least 5 bytes (the minimal size for a data packet)
    ///   * This is actually a data packet (i.e., the first byte is odd)
    fn try_from(bytes: &[u8]) -> Result<Self, PacketParseError> {
        if bytes.len() < 5 {
            return Err(PacketParseError::IncompletePacket)
        }
        assert!(Packet::is_header(bytes)?.not(), "expected a data packet but first byte was not odd");
        let file_id = bytes[1];
        let packet_number_bytes: [u8; 2] = [bytes[2], bytes[3]];
        let packet_number = u16::from_be_bytes(packet_number_bytes);
        let is_last_packet = bytes[0] % 4 == 3;
        let data = bytes[4..].to_vec();

        Ok(Self { file_id, packet_number, is_last_packet, data })
    }
}

#[cfg(test)]
mod is_header_tests {
    use crate::packets::{Packet, PacketParseError};
    
    #[test]
    fn is_header_with_0() {
        let bytes: Vec<u8> = vec![0, 5, 8, 9, 6, 3, 2, 0];
        let result = Packet::is_header(&bytes);
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn is_header_with_12() {
        let bytes: Vec<u8> = vec![12, 5, 8, 9, 6, 3, 2, 0];
        let result = Packet::is_header(&bytes);
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn is_not_header_with_1() {
        let bytes: Vec<u8> = vec![1, 5, 8, 9, 6, 3, 2, 0];
        let result = Packet::is_header(&bytes);
        assert_eq!(result, Ok(false));
    }

    #[test]
    fn is_not_header_with_3() {
        let bytes: Vec<u8> = vec![3, 5, 8, 9, 6, 3, 2, 0];
        let result = Packet::is_header(&bytes);
        assert_eq!(result, Ok(false));
    }

    #[test]
    fn error_on_empty_array() {
        let bytes: Vec<u8> = vec![];
        let result = Packet::is_header(&bytes);
        assert_eq!(result, Err(PacketParseError::IncompletePacket));
    }
}

#[cfg(test)]
mod parse_header_tests {
    use super::{Header}; // , PacketParseError};

    #[test]
    fn error_on_empty_array() {
        let bytes: Vec<u8> = vec![];
        let result = Header::try_from(bytes.as_slice());
        assert!(result.is_err());
        // TODO: Figure out how to check the source of the error.
        // assert_eq!(result, Err(PacketParseError::IncompletePacket));
    }

    #[test]
    fn error_on_short_array() {
        let bytes: Vec<u8> = vec![0, 1];
        let result = Header::try_from(bytes.as_slice());
        assert!(result.is_err());
        // TODO: Figure out how to check the source of the error.
        // assert_eq!(result, Err(PacketParseError::IncompletePacket));
    }

    #[test]
    #[should_panic(expected = "expected a header packet but first byte was not even")]
    fn non_header_panics() {
        let bytes: Vec<u8> = vec![1, 5, 8, 9, 6, 3, 2, 0];
        let _ = Header::try_from(bytes.as_slice());
    }

    #[test]
    fn emoji_in_file_name() {
        // The last four bytes in the following are legal bytes
        // for a sparkle heart emoji
        let sparkle_heart = vec![0, 0, 240, 159, 146, 150];
        let result = Header::try_from(sparkle_heart.as_slice());
        assert!(result.is_ok());
        if let Ok(Header { file_id, file_name }) = result {
            assert_eq!(0, file_id);
            assert_eq!("💖", file_name);
        }
    }

    #[test]
    fn illegal_file_name() {
        // The following is legal bytes for a sparkle heart emoji
        // let sparkle_heart = vec![240, 159, 146, 150];
        // These last four bytes are not legal utf8 because we replaced
        // the first byte in the emoji sequence with a 0.
        let sparkle_heart: Vec<u8> = vec![0, 0, 0, 159, 146, 150];
        let result = Header::try_from(sparkle_heart.as_slice());
        assert!(result.is_err());
        // TODO: Figure out how to check the source of the error.
        // assert_eq!(result, Err(PacketParseError::FilenameParseError));
    }
}

#[cfg(test)]
mod parse_data_tests {
    use super::{Data, PacketParseError};

    #[test]
    fn error_on_empty_array() {
        let bytes: Vec<u8> = vec![];
        let result = Data::try_from(bytes.as_slice());
        assert_eq!(result, Err(PacketParseError::IncompletePacket));
    }

    #[test]
    fn error_on_short_array() {
        let bytes: Vec<u8> = vec![0, 1, 2, 3];
        let result = Data::try_from(bytes.as_slice());
        assert_eq!(result, Err(PacketParseError::IncompletePacket));
    }

    #[test]
    #[should_panic(expected = "expected a data packet but first byte was not odd")]
    fn non_data_panics() {
        let bytes: Vec<u8> = vec![0, 5, 8, 9, 6, 3, 2, 0];
        let _ = Data::try_from(bytes.as_slice());
    }

    #[test]
    fn not_last_data_packet() {
        let bytes: Vec<u8> = vec![1, 5, 8, 9, 6, 3, 2, 0];
        let result = Data::try_from(bytes.as_slice()).unwrap();
        assert!(!result.is_last_packet);
    }

    #[test]
    fn is_last_data_packet() {
        let bytes: Vec<u8> = vec![3, 5, 8, 9, 6, 3, 2, 0];
        let result = Data::try_from(bytes.as_slice()).unwrap();
        assert!(result.is_last_packet);
    }

    #[test]
    fn parse_packet_number() {
        let bytes: Vec<u8> = vec![3, 5, 8, 9, 3, 2, 0];
        let result = Data::try_from(bytes.as_slice()).unwrap();
        assert_eq!(result.packet_number, 8*256+9);
    }

    #[test]
    fn extract_data() {
        let bytes: Vec<u8> = vec![3, 5, 8, 9, 3, 2, 0];
        let result = Data::try_from(bytes.as_slice()).unwrap();
        assert_eq!(result.data, vec![3, 2, 0]);
    }
}

use quickcheck::{Arbitrary, Gen};

impl Arbitrary for Header {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            file_id: u8::arbitrary(g),
            file_name: String::arbitrary(g),
        }
    }
}

impl Arbitrary for Data {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            file_id: u8::arbitrary(g),
            packet_number: u16::arbitrary(g),
            is_last_packet: bool::arbitrary(g),
            data: Vec::arbitrary(g),
        }
    }
}
