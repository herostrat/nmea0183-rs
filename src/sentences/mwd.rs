use nom::{
    IResult, Parser as _, character::complete::char, combinator::opt, number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// MWD - Wind Direction & Speed
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_mwd_wind_direction_speed>
///
/// ```text
///        1   2 3   4 5   6 7   8 9
///        |   | |   | |   | |   | |
/// $--MWD,x.x,T,x.x,M,x.x,N,x.x,M*hh<CR><LF>
/// ```
/// 1. Wind direction, degrees True
/// 2. T = True
/// 3. Wind direction, degrees Magnetic
/// 4. M = Magnetic
/// 5. Wind speed, knots
/// 6. N = Knots
/// 7. Wind speed, meters/second
/// 8. M = Meters per second
/// 9. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MwdData {
    /// Wind direction, degrees True
    pub wind_direction_true: Option<f32>,
    /// Wind direction, degrees Magnetic
    pub wind_direction_magnetic: Option<f32>,
    /// Wind speed, knots
    pub wind_speed_knots: Option<f32>,
    /// Wind speed, meters per second
    pub wind_speed_mps: Option<f32>,
}

impl From<MwdData> for ParseResult {
    fn from(value: MwdData) -> Self {
        ParseResult::MWD(value)
    }
}

/// # Parse MWD message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_mwd_wind_direction_speed>
pub fn parse_mwd(sentence: NmeaSentence<'_>) -> Result<MwdData, Error<'_>> {
    if sentence.message_id != SentenceType::MWD {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::MWD,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_mwd(sentence.data)?.1)
    }
}

fn do_parse_mwd(i: &str) -> IResult<&str, MwdData> {
    let (i, wind_direction_true) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('T')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, wind_direction_magnetic) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('M')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, wind_speed_knots) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('N')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, wind_speed_mps) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('M')).parse(i)?;
    Ok((
        i,
        MwdData {
            wind_direction_true,
            wind_direction_magnetic,
            wind_speed_knots,
            wind_speed_mps,
        },
    ))
}

impl crate::generate::GenerateNmeaBody for MwdData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::MWD
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        if let Some(v) = self.wind_direction_true {
            write!(f, "{}", v)?;
        }
        f.write_str(",T,")?;
        if let Some(v) = self.wind_direction_magnetic {
            write!(f, "{}", v)?;
        }
        f.write_str(",M,")?;
        if let Some(v) = self.wind_speed_knots {
            write!(f, "{}", v)?;
        }
        f.write_str(",N,")?;
        if let Some(v) = self.wind_speed_mps {
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
    fn test_parse_mwd() {
        let s = parse_nmea_sentence("$WIMWD,270.0,T,265.5,M,10.2,N,5.3,M*6E").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_mwd(s).unwrap();
        assert_relative_eq!(data.wind_direction_true.unwrap(), 270.0);
        assert_relative_eq!(data.wind_direction_magnetic.unwrap(), 265.5);
        assert_relative_eq!(data.wind_speed_knots.unwrap(), 10.2);
        assert_relative_eq!(data.wind_speed_mps.unwrap(), 5.3);
    }

    #[test]
    fn test_parse_mwd_empty() {
        let s = parse_nmea_sentence("$WIMWD,,T,,M,,N,,M*5A").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_mwd(s).unwrap();
        assert_eq!(data.wind_direction_true, None);
        assert_eq!(data.wind_direction_magnetic, None);
        assert_eq!(data.wind_speed_knots, None);
        assert_eq!(data.wind_speed_mps, None);
    }

    #[test]
    fn test_generate_mwd_roundtrip() {
        let original = MwdData {
            wind_direction_true: Some(270.0),
            wind_direction_magnetic: Some(265.5),
            wind_speed_knots: Some(10.2),
            wind_speed_mps: Some(5.3),
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("WI", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_mwd(s).unwrap();
        assert_relative_eq!(parsed.wind_direction_true.unwrap(), 270.0);
        assert_relative_eq!(parsed.wind_direction_magnetic.unwrap(), 265.5);
        assert_relative_eq!(parsed.wind_speed_knots.unwrap(), 10.2);
        assert_relative_eq!(parsed.wind_speed_mps.unwrap(), 5.3);
    }
}
