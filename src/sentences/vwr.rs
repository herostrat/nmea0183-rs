use nom::{
    IResult, Parser as _,
    character::complete::{char, one_of},
    combinator::opt,
    number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// VWR - Relative Wind Speed and Angle
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_vwr_relative_wind_speed_and_angle>
///
/// ```text
///        1   2 3   4 5   6 7   8 9
///        |   | |   | |   | |   | |
/// $--VWR,x.x,a,x.x,N,x.x,M,x.x,K*hh<CR><LF>
/// ```
/// 1. Wind direction magnitude in degrees (0 to 180)
/// 2. Wind direction L=Left/port, R=Right/starboard relative to vessel heading
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
pub struct VwrData {
    /// Wind angle relative to bow (0-180), negative = port, positive = starboard
    pub wind_angle: Option<f32>,
    /// Wind speed in knots
    pub speed_knots: Option<f32>,
    /// Wind speed in meters per second
    pub speed_mps: Option<f32>,
    /// Wind speed in km/hr
    pub speed_kmph: Option<f32>,
}

impl From<VwrData> for ParseResult {
    fn from(value: VwrData) -> Self {
        ParseResult::VWR(value)
    }
}

/// # Parse VWR message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_vwr_relative_wind_speed_and_angle>
pub fn parse_vwr(sentence: NmeaSentence<'_>) -> Result<VwrData, Error<'_>> {
    if sentence.message_id != SentenceType::VWR {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::VWR,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_vwr(sentence.data)?.1)
    }
}

fn do_parse_vwr(i: &str) -> IResult<&str, VwrData> {
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
        VwrData {
            wind_angle,
            speed_knots,
            speed_mps,
            speed_kmph,
        },
    ))
}

impl crate::generate::GenerateNmeaBody for VwrData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::VWR
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
    fn test_parse_vwr() {
        let s = parse_nmea_sentence("$IIVWR,75,R,1.0,N,0.51,M,1.85,K*6C").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vwr(s).unwrap();
        assert_relative_eq!(data.wind_angle.unwrap(), 75.0);
        assert_relative_eq!(data.speed_knots.unwrap(), 1.0);
        assert_relative_eq!(data.speed_mps.unwrap(), 0.51);
        assert_relative_eq!(data.speed_kmph.unwrap(), 1.85);
    }

    #[test]
    fn test_parse_vwr_port() {
        let s = parse_nmea_sentence("$IIVWR,024,L,018,N,,,,*5E").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vwr(s).unwrap();
        assert_relative_eq!(data.wind_angle.unwrap(), -24.0); // port = negative
        assert_relative_eq!(data.speed_knots.unwrap(), 18.0);
    }

    #[test]
    fn test_parse_vwr_empty() {
        let s = parse_nmea_sentence("$IIVWR,,,,,,,,*53").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vwr(s).unwrap();
        assert_eq!(data.wind_angle, None);
        assert_eq!(data.speed_knots, None);
        assert_eq!(data.speed_mps, None);
        assert_eq!(data.speed_kmph, None);
    }

    #[test]
    fn test_generate_vwr_roundtrip() {
        let original = VwrData {
            wind_angle: Some(75.0),
            speed_knots: Some(1.0),
            speed_mps: Some(0.51),
            speed_kmph: Some(1.85),
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("II", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_vwr(s).unwrap();
        assert_relative_eq!(parsed.wind_angle.unwrap(), 75.0);
        assert_relative_eq!(parsed.speed_knots.unwrap(), 1.0);
    }
}
