use nom::{
    IResult, Parser as _,
    character::complete::char,
    combinator::opt,
    number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// ROT - Rate Of Turn
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_rot_rate_of_turn>
///
/// ```text
///        1   2 3
///        |   | |
/// $--ROT,x.x,A*hh<CR><LF>
/// ```
/// 1. Rate of turn, degrees per minute, negative means bow turns to port
/// 2. Status: A = data is valid
/// 3. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotData {
    /// Rate of turn in degrees per minute, negative = bow turns to port
    pub rate: Option<f32>,
    /// Data validity status
    pub valid: Option<bool>,
}

impl From<RotData> for ParseResult {
    fn from(value: RotData) -> Self {
        ParseResult::ROT(value)
    }
}

/// # Parse ROT message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_rot_rate_of_turn>
pub fn parse_rot(sentence: NmeaSentence<'_>) -> Result<RotData, Error<'_>> {
    if sentence.message_id != SentenceType::ROT {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::ROT,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_rot(sentence.data)?.1)
    }
}

fn do_parse_rot(i: &str) -> IResult<&str, RotData> {
    let (i, rate) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, status) = opt(char('A')).parse(i)?;
    Ok((
        i,
        RotData {
            rate,
            valid: Some(status.is_some()),
        },
    ))
}

impl crate::generate::GenerateNmeaBody for RotData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::ROT
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        if let Some(r) = self.rate {
            write!(f, "{}", r)?;
        }
        f.write_char(',')?;
        match self.valid {
            Some(true) => f.write_char('A'),
            _ => f.write_char('V'),
        }
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_rot() {
        let s = parse_nmea_sentence("$TIROT,-0.3,A*15").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_rot(s).unwrap();
        assert_relative_eq!(data.rate.unwrap(), -0.3);
        assert_eq!(data.valid, Some(true));
    }

    #[test]
    fn test_parse_rot_empty() {
        let s = parse_nmea_sentence("$TIROT,,A*15").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_rot(s).unwrap();
        assert_eq!(data.rate, None);
        assert_eq!(data.valid, Some(true));
    }

    #[test]
    fn test_generate_rot_roundtrip() {
        let original = RotData {
            rate: Some(-0.3),
            valid: Some(true),
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("TI", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_rot(s).unwrap();
        assert_relative_eq!(parsed.rate.unwrap(), -0.3);
        assert_eq!(parsed.valid, Some(true));
    }
}
