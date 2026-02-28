use heapless::Vec;
use nom::{
    IResult, Parser as _,
    bytes::complete::take,
    character::complete::char,
    combinator::opt,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// STALK - SeaTalk1 Datagram
///
/// Raymarine SeaTalk1 protocol encapsulated in NMEA 0183 sentences.
/// Talker ID is "ST", sentence type is "ALK" → `$STALK,...*hh`
///
/// ```text
///        1  2
///        |  |
/// $STALK,cc,p1,p2,...,pn*hh<CR><LF>
/// ```
///
/// All fields are hex-encoded bytes (2 characters, 00-FF):
///
/// 1. Command byte (datagram type)
/// 2. Payload bytes (variable length, depends on command)
///
/// The semantic interpretation of individual datagrams (0x00=Depth,
/// 0x10=Wind, 0x84=Compass, etc.) is left to the application layer.
///
/// Examples:
/// * `$STALK,52,A1,00,00*36` (Speed through water)
/// * `$STALK,84,36,02,00,40,00,03,00,08*69` (Compass heading)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct StalkData {
    /// Command byte (first hex byte = datagram type)
    pub command: u8,
    /// Payload bytes (remaining hex bytes)
    pub payload: Vec<u8, 18>,
}

impl From<StalkData> for ParseResult {
    fn from(value: StalkData) -> Self {
        ParseResult::STALK(value)
    }
}

/// Parse a 2-character hex string into a u8
fn parse_hex_byte(i: &str) -> IResult<&str, u8> {
    let (i, hex) = take(2usize).parse(i)?;
    match u8::from_str_radix(hex, 16) {
        Ok(v) => Ok((i, v)),
        Err(_) => Err(nom::Err::Failure(nom::error::Error::new(
            i,
            nom::error::ErrorKind::HexDigit,
        ))),
    }
}

fn do_parse_stalk(i: &str) -> IResult<&str, StalkData> {
    // Command byte (first hex field)
    let (i, command) = parse_hex_byte(i)?;

    // Payload bytes (comma-separated hex fields)
    let mut payload = Vec::<u8, 18>::new();
    let mut remaining = i;

    loop {
        // Try to consume comma before next byte
        let (r, comma) = opt(char(',')).parse(remaining)?;
        if comma.is_none() {
            break;
        }

        // Parse next hex byte
        let (r, byte) = parse_hex_byte(r)?;
        let _ = payload.push(byte);
        remaining = r;
    }

    Ok((remaining, StalkData { command, payload }))
}

/// # Parse STALK message
///
/// Parses Raymarine SeaTalk1 datagrams encapsulated in NMEA `$STALK` sentences.
/// Only parses the raw hex bytes — semantic interpretation is left to the application.
pub fn parse_stalk(sentence: NmeaSentence<'_>) -> Result<StalkData, Error<'_>> {
    if sentence.message_id != SentenceType::ALK {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::ALK,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_stalk(sentence.data)?.1)
    }
}

impl crate::generate::GenerateNmeaBody for StalkData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::ALK
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        write!(f, "{:02X}", self.command)?;
        for byte in &self.payload {
            write!(f, ",{:02X}", byte)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_stalk_speed() {
        let s = parse_nmea_sentence("$STALK,52,A1,00,00*36").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        assert_eq!(s.talker_id, "ST");
        assert_eq!(s.message_id, SentenceType::ALK);

        let data = parse_stalk(s).unwrap();
        assert_eq!(data.command, 0x52);
        assert_eq!(data.payload.len(), 3);
        assert_eq!(data.payload[0], 0xA1);
        assert_eq!(data.payload[1], 0x00);
        assert_eq!(data.payload[2], 0x00);
    }

    #[test]
    fn test_parse_stalk_compass() {
        let s = parse_nmea_sentence("$STALK,84,36,02,00,40,00,03,00,08*69").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_stalk(s).unwrap();

        assert_eq!(data.command, 0x84);
        assert_eq!(data.payload.len(), 8);
        assert_eq!(data.payload[0], 0x36);
        assert_eq!(data.payload[7], 0x08);
    }

    #[test]
    fn test_parse_stalk_short() {
        let s = parse_nmea_sentence("$STALK,86,21,02,FD*4C").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_stalk(s).unwrap();

        assert_eq!(data.command, 0x86);
        assert_eq!(data.payload.len(), 3);
        assert_eq!(data.payload[0], 0x21);
        assert_eq!(data.payload[1], 0x02);
        assert_eq!(data.payload[2], 0xFD);
    }

    #[test]
    fn test_parse_stalk_single_byte() {
        // Command-only datagram, no payload
        let s = parse_nmea_sentence("$STALK,00*6D").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_stalk(s).unwrap();

        assert_eq!(data.command, 0x00);
        assert_eq!(data.payload.len(), 0);
    }

    #[test]
    fn test_generate_stalk_roundtrip() {
        let original = StalkData {
            command: 0x52,
            payload: {
                let mut v = Vec::new();
                v.push(0xA1).unwrap();
                v.push(0x00).unwrap();
                v.push(0x00).unwrap();
                v
            },
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("ST", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_stalk(s).unwrap();
        assert_eq!(parsed.command, 0x52);
        assert_eq!(parsed.payload.len(), 3);
        assert_eq!(parsed.payload[0], 0xA1);
    }

    #[test]
    fn test_generate_stalk_compass_roundtrip() {
        let original = StalkData {
            command: 0x84,
            payload: {
                let mut v = Vec::new();
                for &b in &[0x36, 0x02, 0x00, 0x40, 0x00, 0x03, 0x00, 0x08] {
                    v.push(b).unwrap();
                }
                v
            },
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("ST", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_stalk(s).unwrap();
        assert_eq!(parsed.command, 0x84);
        assert_eq!(parsed.payload.len(), 8);
    }
}
