use std::str::{self, Utf8Error};

#[derive(Debug, PartialEq)]
enum PacketParseError {
    IncompletePacket,
    FilenameParseError
}

enum Packet {
    Header(Header),
    Data(Data)
}

impl Packet {
    fn is_header(bytes: &[u8]) -> Result<bool, PacketParseError> {
        if bytes.is_empty() {
            return Err(PacketParseError::IncompletePacket)
        }
        Ok(bytes[0] % 2 == 0)
    }
}

impl TryFrom<&[u8]> for Packet {
    type Error = PacketParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, PacketParseError> {
        if Packet::is_header(bytes)? {
            Ok(Packet::Header(bytes.try_into()?))
        } else {
            Ok(Packet::Data(bytes.try_into()?))
        }
    }
}

#[derive(Debug, PartialEq)]
struct Header {
    file_id: u8,
    file_name: String
}

impl From<Utf8Error> for PacketParseError {
    fn from(_: Utf8Error) -> PacketParseError {
        PacketParseError::FilenameParseError
    }
}

// TODO: Look at how we could use the zerocopy crate to automagic
//   some of this conversion.
impl TryFrom<&[u8]> for Header {
    type Error = PacketParseError;

    /// Convert the given byte array slice to a header packet.
    /// This assumes
    ///   * All the bytes in the given slice are
    ///     used (i.e., there are no "empty" or unused bytes at the
    ///     end)
    ///   * There are at least 3 bytes (the minimal size for a header packet)
    ///   * This is actually a header packet (i.e., the first byte is even)
    ///   * Bytes 2.. can be parsed as a String  
    fn try_from(bytes: &[u8]) -> Result<Self, PacketParseError> {
        if bytes.len() < 3 {
            return Err(PacketParseError::IncompletePacket)
        }
        assert!(Packet::is_header(bytes)?, "expected a header packet but first byte was not even");
        let file_id = bytes[1];
        let file_name = str::from_utf8(&bytes[2..])?.to_string();

        Ok(Header { file_id, file_name })
    }
}

#[derive(Debug, PartialEq)]
struct Data {
    file_id: u8,
    packet_number: u16,
    data: Vec<u8>
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
        todo!()
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
    use super::{Header, PacketParseError};

    #[test]
    fn error_on_empty_array() {
        let bytes: Vec<u8> = vec![];
        let result = Header::try_from(bytes.as_slice());
        assert_eq!(result, Err(PacketParseError::IncompletePacket));
    }

    #[test]
    fn error_on_short_array() {
        let bytes: Vec<u8> = vec![0, 1];
        let result = Header::try_from(bytes.as_slice());
        assert_eq!(result, Err(PacketParseError::IncompletePacket));
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
        assert_eq!(result, Ok(Header{ file_id: 0, file_name: "💖".to_string() }));
    }

    #[test]
    fn illegal_file_name() {
        // The following is legal bytes for a sparkle heart emoji
        // let sparkle_heart = vec![240, 159, 146, 150];
        // These last four bytes are not legal utf8 because we replaced
        // the first byte in the emoji sequence with a 0.
        let sparkle_heart: Vec<u8> = vec![0, 0, 0, 159, 146, 150];
        let result = Header::try_from(sparkle_heart.as_slice());
        assert_eq!(result, Err(PacketParseError::FilenameParseError));
    }
}