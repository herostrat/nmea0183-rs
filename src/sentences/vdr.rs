use nom::{
    IResult, Parser as _,
    character::complete::char,
    combinator::opt,
    number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// VDR - Set and Drift
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_vdr_set_and_drift>
///
/// ```text
///        1   2 3   4 5   6 7
///        |   | |   | |   | |
/// $--VDR,x.x,T,x.x,M,x.x,N*hh<CR><LF>
/// ```
/// 1. Direction, degrees True
/// 2. T = True
/// 3. Direction, degrees Magnetic
/// 4. M = Magnetic
/// 5. Current Speed, knots
/// 6. N = Knots
/// 7. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VdrData {
    /// Set (current direction), degrees True
    pub direction_true: Option<f32>,
    /// Set (current direction), degrees Magnetic
    pub direction_magnetic: Option<f32>,
    /// Drift (current speed), knots
    pub speed: Option<f32>,
}

impl From<VdrData> for ParseResult {
    fn from(value: VdrData) -> Self {
        ParseResult::VDR(value)
    }
}

/// # Parse VDR message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_vdr_set_and_drift>
pub fn parse_vdr(sentence: NmeaSentence<'_>) -> Result<VdrData, Error<'_>> {
    if sentence.message_id != SentenceType::VDR {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::VDR,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_vdr(sentence.data)?.1)
    }
}

fn do_parse_vdr(i: &str) -> IResult<&str, VdrData> {
    let (i, direction_true) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('T')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, direction_magnetic) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('M')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, speed) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('N')).parse(i)?;
    Ok((
        i,
        VdrData {
            direction_true,
            direction_magnetic,
            speed,
        },
    ))
}

impl crate::generate::GenerateNmeaBody for VdrData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::VDR
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        if let Some(v) = self.direction_true {
            write!(f, "{}", v)?;
        }
        f.write_str(",T,")?;
        if let Some(v) = self.direction_magnetic {
            write!(f, "{}", v)?;
        }
        f.write_str(",M,")?;
        if let Some(v) = self.speed {
            write!(f, "{}", v)?;
        }
        f.write_str(",N")
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_vdr() {
        let s = parse_nmea_sentence("$IIVDR,180.0,T,175.5,M,1.5,N*32").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vdr(s).unwrap();
        assert_relative_eq!(data.direction_true.unwrap(), 180.0);
        assert_relative_eq!(data.direction_magnetic.unwrap(), 175.5);
        assert_relative_eq!(data.speed.unwrap(), 1.5);
    }

    #[test]
    fn test_parse_vdr_empty() {
        let s = parse_nmea_sentence("$IIVDR,,T,,M,,N*17").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vdr(s).unwrap();
        assert_eq!(data.direction_true, None);
        assert_eq!(data.direction_magnetic, None);
        assert_eq!(data.speed, None);
    }

    #[test]
    fn test_generate_vdr_roundtrip() {
        let original = VdrData {
            direction_true: Some(180.0),
            direction_magnetic: Some(175.5),
            speed: Some(1.5),
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("II", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_vdr(s).unwrap();
        assert_relative_eq!(parsed.direction_true.unwrap(), 180.0);
        assert_relative_eq!(parsed.direction_magnetic.unwrap(), 175.5);
        assert_relative_eq!(parsed.speed.unwrap(), 1.5);
    }
}
