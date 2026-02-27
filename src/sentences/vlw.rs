use nom::{
    IResult, Parser as _,
    character::complete::char,
    combinator::opt,
    number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// VLW - Distance Traveled through Water
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_vlw_distance_traveled_through_water>
///
/// ```text
///        1   2 3   4 5   6 7   8 9
///        |   | |   | |   | |   | |
/// $--VLW,x.x,N,x.x,N,x.x,N,x.x,N*hh<CR><LF>
/// ```
/// 1. Total cumulative water distance, nautical miles
/// 2. N = Nautical Miles
/// 3. Water distance since Reset, nautical miles
/// 4. N = Nautical Miles
/// 5. Total cumulative ground distance, nautical miles (NMEA 3.0+)
/// 6. N = Nautical Miles
/// 7. Ground distance since Reset, nautical miles (NMEA 3.0+)
/// 8. N = Nautical Miles
/// 9. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VlwData {
    /// Total cumulative water distance in nautical miles
    pub total_water_distance: Option<f32>,
    /// Water distance since reset in nautical miles
    pub trip_water_distance: Option<f32>,
    /// Total cumulative ground distance in nautical miles (NMEA 3.0+)
    pub total_ground_distance: Option<f32>,
    /// Ground distance since reset in nautical miles (NMEA 3.0+)
    pub trip_ground_distance: Option<f32>,
}

impl From<VlwData> for ParseResult {
    fn from(value: VlwData) -> Self {
        ParseResult::VLW(value)
    }
}

/// # Parse VLW message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_vlw_distance_traveled_through_water>
pub fn parse_vlw(sentence: NmeaSentence<'_>) -> Result<VlwData, Error<'_>> {
    if sentence.message_id != SentenceType::VLW {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::VLW,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_vlw(sentence.data)?.1)
    }
}

fn parse_nm_field(i: &str) -> IResult<&str, Option<f32>> {
    let (i, val) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('N')).parse(i)?;
    Ok((i, val))
}

fn do_parse_vlw(i: &str) -> IResult<&str, VlwData> {
    let (i, total_water_distance) = parse_nm_field(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, trip_water_distance) = parse_nm_field(i)?;
    // Fields 5-8 are optional (NMEA 3.0+)
    let (i, total_ground_distance) = opt(|i| {
        let (i, _) = char(',').parse(i)?;
        parse_nm_field(i)
    })
    .parse(i)?;
    let (i, trip_ground_distance) = opt(|i| {
        let (i, _) = char(',').parse(i)?;
        parse_nm_field(i)
    })
    .parse(i)?;
    Ok((
        i,
        VlwData {
            total_water_distance,
            trip_water_distance,
            total_ground_distance: total_ground_distance.flatten(),
            trip_ground_distance: trip_ground_distance.flatten(),
        },
    ))
}

impl crate::generate::GenerateNmeaBody for VlwData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::VLW
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        if let Some(v) = self.total_water_distance {
            write!(f, "{}", v)?;
        }
        f.write_str(",N,")?;
        if let Some(v) = self.trip_water_distance {
            write!(f, "{}", v)?;
        }
        f.write_str(",N")?;
        if self.total_ground_distance.is_some() || self.trip_ground_distance.is_some() {
            f.write_char(',')?;
            if let Some(v) = self.total_ground_distance {
                write!(f, "{}", v)?;
            }
            f.write_str(",N,")?;
            if let Some(v) = self.trip_ground_distance {
                write!(f, "{}", v)?;
            }
            f.write_str(",N")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_vlw() {
        let s = parse_nmea_sentence("$VWVLW,7803.2,N,0.00,N*42").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vlw(s).unwrap();
        assert_relative_eq!(data.total_water_distance.unwrap(), 7803.2);
        assert_relative_eq!(data.trip_water_distance.unwrap(), 0.0);
        assert_eq!(data.total_ground_distance, None);
        assert_eq!(data.trip_ground_distance, None);
    }

    #[test]
    fn test_parse_vlw_with_ground() {
        let s = parse_nmea_sentence("$VWVLW,7803.2,N,0.00,N,8000.1,N,10.5,N*4F").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vlw(s).unwrap();
        assert_relative_eq!(data.total_water_distance.unwrap(), 7803.2);
        assert_relative_eq!(data.trip_water_distance.unwrap(), 0.0);
        assert_relative_eq!(data.total_ground_distance.unwrap(), 8000.1);
        assert_relative_eq!(data.trip_ground_distance.unwrap(), 10.5);
    }

    #[test]
    fn test_generate_vlw_roundtrip() {
        let original = VlwData {
            total_water_distance: Some(7803.2),
            trip_water_distance: Some(0.0),
            total_ground_distance: None,
            trip_ground_distance: None,
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("VW", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_vlw(s).unwrap();
        assert_relative_eq!(parsed.total_water_distance.unwrap(), 7803.2);
        assert_relative_eq!(parsed.trip_water_distance.unwrap(), 0.0);
    }
}
