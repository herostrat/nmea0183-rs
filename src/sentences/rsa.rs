use nom::{
    IResult, Parser as _,
    character::complete::char,
    combinator::opt,
    number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// RSA - Rudder Sensor Angle
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_rsa_rudder_sensor_angle>
///
/// ```text
///        1   2 3   4 5
///        |   | |   | |
/// $--RSA,x.x,A,x.x,A*hh<CR><LF>
/// ```
/// 1. Starboard (or single) rudder sensor, negative means Turn To Port
/// 2. Status: A = data is valid
/// 3. Port rudder sensor
/// 4. Status: A = data is valid
/// 5. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RsaData {
    /// Starboard (or single) rudder sensor angle in degrees
    pub starboard: Option<f32>,
    /// Starboard data valid
    pub starboard_valid: bool,
    /// Port rudder sensor angle in degrees
    pub port: Option<f32>,
    /// Port data valid
    pub port_valid: bool,
}

impl From<RsaData> for ParseResult {
    fn from(value: RsaData) -> Self {
        ParseResult::RSA(value)
    }
}

/// # Parse RSA message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_rsa_rudder_sensor_angle>
pub fn parse_rsa(sentence: NmeaSentence<'_>) -> Result<RsaData, Error<'_>> {
    if sentence.message_id != SentenceType::RSA {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::RSA,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_rsa(sentence.data)?.1)
    }
}

fn do_parse_rsa(i: &str) -> IResult<&str, RsaData> {
    let (i, starboard) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, stb_status) = opt(char('A')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, port) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, port_status) = opt(char('A')).parse(i)?;
    Ok((
        i,
        RsaData {
            starboard,
            starboard_valid: stb_status.is_some(),
            port,
            port_valid: port_status.is_some(),
        },
    ))
}

impl crate::generate::GenerateNmeaBody for RsaData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::RSA
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        if let Some(v) = self.starboard {
            write!(f, "{}", v)?;
        }
        f.write_char(',')?;
        f.write_char(if self.starboard_valid { 'A' } else { 'V' })?;
        f.write_char(',')?;
        if let Some(v) = self.port {
            write!(f, "{}", v)?;
        }
        f.write_char(',')?;
        f.write_char(if self.port_valid { 'A' } else { 'V' })
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_rsa() {
        let s = parse_nmea_sentence("$IIRSA,10.5,A,,V*4D").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_rsa(s).unwrap();
        assert_relative_eq!(data.starboard.unwrap(), 10.5);
        assert!(data.starboard_valid);
        assert_eq!(data.port, None);
        assert!(!data.port_valid);
    }

    #[test]
    fn test_parse_rsa_dual() {
        let s = parse_nmea_sentence("$IIRSA,5.2,A,-3.1,A*68").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_rsa(s).unwrap();
        assert_relative_eq!(data.starboard.unwrap(), 5.2);
        assert!(data.starboard_valid);
        assert_relative_eq!(data.port.unwrap(), -3.1);
        assert!(data.port_valid);
    }

    #[test]
    fn test_generate_rsa_roundtrip() {
        let original = RsaData {
            starboard: Some(10.5),
            starboard_valid: true,
            port: None,
            port_valid: false,
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("II", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_rsa(s).unwrap();
        assert_eq!(parsed, original);
    }
}
