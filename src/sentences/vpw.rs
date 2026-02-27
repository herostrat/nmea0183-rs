use nom::{
    IResult, Parser as _,
    character::complete::char,
    combinator::opt,
    number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// VPW - Speed Measured Parallel to Wind
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_vpw_speed_measured_parallel_to_wind>
///
/// ```text
///        1   2 3   4 5
///        |   | |   | |
/// $--VPW,x.x,N,x.x,M*hh<CR><LF>
/// ```
/// 1. Speed, knots, negative means downwind
/// 2. N = Knots
/// 3. Speed, meters per second, negative means downwind
/// 4. M = Meters per second
/// 5. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VpwData {
    /// Speed parallel to wind in knots (negative = downwind)
    pub speed_knots: Option<f32>,
    /// Speed parallel to wind in meters per second (negative = downwind)
    pub speed_mps: Option<f32>,
}

impl From<VpwData> for ParseResult {
    fn from(value: VpwData) -> Self {
        ParseResult::VPW(value)
    }
}

/// # Parse VPW message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_vpw_speed_measured_parallel_to_wind>
pub fn parse_vpw(sentence: NmeaSentence<'_>) -> Result<VpwData, Error<'_>> {
    if sentence.message_id != SentenceType::VPW {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::VPW,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_vpw(sentence.data)?.1)
    }
}

fn do_parse_vpw(i: &str) -> IResult<&str, VpwData> {
    let (i, speed_knots) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('N')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, speed_mps) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('M')).parse(i)?;
    Ok((
        i,
        VpwData {
            speed_knots,
            speed_mps,
        },
    ))
}

impl crate::generate::GenerateNmeaBody for VpwData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::VPW
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        if let Some(v) = self.speed_knots {
            write!(f, "{}", v)?;
        }
        f.write_str(",N,")?;
        if let Some(v) = self.speed_mps {
            write!(f, "{}", v)?;
        }
        f.write_str(",M")
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_vpw() {
        let s = parse_nmea_sentence("$IIVPW,4.5,N,2.3,M*52").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vpw(s).unwrap();
        assert_relative_eq!(data.speed_knots.unwrap(), 4.5);
        assert_relative_eq!(data.speed_mps.unwrap(), 2.3);
    }

    #[test]
    fn test_parse_vpw_empty() {
        let s = parse_nmea_sentence("$IIVPW,,N,,M*52").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vpw(s).unwrap();
        assert_eq!(data.speed_knots, None);
        assert_eq!(data.speed_mps, None);
    }

    #[test]
    fn test_generate_vpw_roundtrip() {
        let original = VpwData {
            speed_knots: Some(4.5),
            speed_mps: Some(2.3),
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("II", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_vpw(s).unwrap();
        assert_relative_eq!(parsed.speed_knots.unwrap(), 4.5);
        assert_relative_eq!(parsed.speed_mps.unwrap(), 2.3);
    }
}
