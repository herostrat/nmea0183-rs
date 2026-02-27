use nom::{
    IResult, Parser as _,
    character::complete::{char, one_of},
    combinator::opt,
    number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// VWT - True Wind Speed and Angle
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_vwt_true_wind_speed_and_angle>
///
/// ```text
///        1   2 3   4 5   6 7   8 9
///        |   | |   | |   | |   | |
/// $--VWT,x.x,a,x.x,N,x.x,M,x.x,K*hh<CR><LF>
/// ```
/// 1. Wind angle, 0 to 180 degrees
/// 2. L=Left/port, R=Right/starboard relative to vessel heading
/// 3. Wind speed, knots
/// 4. N = Knots
/// 5. Wind speed, meters per second
/// 6. M = Meters per second
/// 7. Wind speed, km/hr
/// 8. K = Km/hr
/// 9. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VwtData {
    /// True wind angle relative to bow (0-180), negative = port, positive = starboard
    pub wind_angle: Option<f32>,
    /// Wind speed in knots
    pub speed_knots: Option<f32>,
    /// Wind speed in meters per second
    pub speed_mps: Option<f32>,
    /// Wind speed in km/hr
    pub speed_kmph: Option<f32>,
}

impl From<VwtData> for ParseResult {
    fn from(value: VwtData) -> Self {
        ParseResult::VWT(value)
    }
}

/// # Parse VWT message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_vwt_true_wind_speed_and_angle>
pub fn parse_vwt(sentence: NmeaSentence<'_>) -> Result<VwtData, Error<'_>> {
    if sentence.message_id != SentenceType::VWT {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::VWT,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_vwt(sentence.data)?.1)
    }
}

fn do_parse_vwt(i: &str) -> IResult<&str, VwtData> {
    let (i, angle) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, direction) = opt(one_of("LR")).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, speed_knots) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('N')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, speed_mps) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('M')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, speed_kmph) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('K')).parse(i)?;

    let wind_angle = match (angle, direction) {
        (Some(a), Some('L')) => Some(-a),
        (Some(a), _) => Some(a),
        _ => None,
    };

    Ok((
        i,
        VwtData {
            wind_angle,
            speed_knots,
            speed_mps,
            speed_kmph,
        },
    ))
}

impl crate::generate::GenerateNmeaBody for VwtData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::VWT
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        if let Some(angle) = self.wind_angle {
            write!(f, "{}", angle.abs())?;
            f.write_str(",")?;
            if angle < 0.0 {
                f.write_str("L")?;
            } else {
                f.write_str("R")?;
            }
        } else {
            f.write_str(",")?;
        }
        f.write_str(",")?;
        if let Some(v) = self.speed_knots {
            write!(f, "{}", v)?;
        }
        f.write_str(",N,")?;
        if let Some(v) = self.speed_mps {
            write!(f, "{}", v)?;
        }
        f.write_str(",M,")?;
        if let Some(v) = self.speed_kmph {
            write!(f, "{}", v)?;
        }
        f.write_str(",K")
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_vwt() {
        let s = parse_nmea_sentence("$IIVWT,030.,R,10.1,N,05.2,M,018.7,K*75").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vwt(s).unwrap();
        assert_relative_eq!(data.wind_angle.unwrap(), 30.0);
        assert_relative_eq!(data.speed_knots.unwrap(), 10.1);
        assert_relative_eq!(data.speed_mps.unwrap(), 5.2);
        assert_relative_eq!(data.speed_kmph.unwrap(), 18.7);
    }

    #[test]
    fn test_parse_vwt_port() {
        let s = parse_nmea_sentence("$IIVWT,170.,L,10.1,N,05.2,M,018.7,K*6E").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vwt(s).unwrap();
        assert_relative_eq!(data.wind_angle.unwrap(), -170.0);
    }

    #[test]
    fn test_parse_vwt_empty() {
        let s = parse_nmea_sentence("$IIVWT,,,,,,,,*55").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vwt(s).unwrap();
        assert_eq!(data.wind_angle, None);
    }

    #[test]
    fn test_generate_vwt_roundtrip() {
        let original = VwtData {
            wind_angle: Some(30.0),
            speed_knots: Some(10.1),
            speed_mps: Some(5.2),
            speed_kmph: Some(18.7),
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("II", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_vwt(s).unwrap();
        assert_relative_eq!(parsed.wind_angle.unwrap(), 30.0);
        assert_relative_eq!(parsed.speed_mps.unwrap(), 5.2);
    }
}
