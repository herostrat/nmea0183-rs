use nom::{
    IResult, Parser as _, character::complete::char, combinator::opt, number::complete::float,
};

use crate::{Error, SentenceType, parse::NmeaSentence};

/// MDA - Meterological Composite
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_mda_meteorological_composite>
///
/// ```text
///          1   2  3    4  5  6 7 8  9 10 11 12 13 14 15 16 17 18 19 20 21
///          |   |  |    |  |  | | |  |  |  |  |  |  |  |  |  |  |  |  |  |
///  $--MDA,n.nn,I,n.nnn,B,n.n,C,n.C,n.n,n,n.n,C,n.n,T,n.n,M,n.n,N,n.n,M*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq)]
pub struct MdaData {
    /// Pressure in inches of mercury
    pub pressure_in_hg: Option<f32>,
    /// Pressure in bars
    pub pressure_bar: Option<f32>,
    /// Air temp, deg celsius
    pub air_temp_deg: Option<f32>,
    /// Water temp, deg celsius
    pub water_temp_deg: Option<f32>,
    /// Relative humidity, percent
    pub rel_humidity: Option<f32>,
    /// Absolute humidity, percent
    pub abs_humidity: Option<f32>,
    /// Dew point, degrees celsius
    pub dew_point: Option<f32>,
    /// True Wind Direction, NED degrees
    pub wind_direction_true: Option<f32>,
    /// Magnetic Wind Direction, NED degrees
    pub wind_direction_magnetic: Option<f32>,
    /// Wind speed knots
    pub wind_speed_knots: Option<f32>,
    /// Wind speed meters/second
    pub wind_speed_ms: Option<f32>,
}

/// # Parse MDA message
///
/// Information from mda:
///
/// NMEA 0183 standard Wind Speed and Angle, in relation to the vessel’s bow/centerline.
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_mda_meteorological_composite>
///
///  Example: `$WIMDA,29.7544,I,1.0076,B,35.5,C,17.5,C,42.1,30.6,20.6,C,116.4,T,107.7,M,1.2,N,0.6,M*66`
///
///
/// 1. 29.7544     Pressure in inches of mercury
/// 2. I
/// 3. 1.0076      Pressure in bars
/// 4. B
/// 5. 35.5        Air temp, deg celsius
/// 6. C
/// 7. 17.5        Water temp, deg celsius
/// 8. C
/// 9. 42.1        Relative humidity, percent
/// 10. 30.6       Absolute humidity, percent
/// 11. 20.6       Dew point, degrees celsius
/// 12. C
/// 13. 116.4      True Wind Direction, NED degrees
/// 14. T
/// 15. 107.7      Magnetic Wind Direction, NED degrees
/// 16. M
/// 17. 1.2        Wind speed knots
/// 18. N
/// 19. 0.6        Wind speed meters/second
/// 20. M
/// 21. *16        Mandatory NMEA checksum
///
pub fn parse_mda(sentence: NmeaSentence<'_>) -> Result<MdaData, Error<'_>> {
    if sentence.message_id != SentenceType::MDA {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::MDA,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_mda(sentence.data)?.1)
    }
}

fn do_parse_mda(i: &str) -> IResult<&str, MdaData> {
    let (i, pressure_in_hg) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('I')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, pressure_bar) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('B')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, air_temp_deg) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('C')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, water_temp_deg) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('C')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, rel_humidity) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, abs_humidity) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, dew_point) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('C')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
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
    let (i, wind_speed_ms) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('M')).parse(i)?;

    Ok((
        i,
        MdaData {
            pressure_in_hg,
            pressure_bar,
            air_temp_deg,
            water_temp_deg,
            rel_humidity,
            abs_humidity,
            dew_point,
            wind_direction_true,
            wind_direction_magnetic,
            wind_speed_knots,
            wind_speed_ms,
        },
    ))
}

impl crate::generate::GenerateNmeaBody for MdaData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::MDA
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        // 1. Pressure in inches of mercury
        if let Some(v) = self.pressure_in_hg {
            write!(f, "{}", v)?;
        }
        f.write_str(",I,")?;
        // 3. Pressure in bars
        if let Some(v) = self.pressure_bar {
            write!(f, "{}", v)?;
        }
        f.write_str(",B,")?;
        // 5. Air temp
        if let Some(v) = self.air_temp_deg {
            write!(f, "{}", v)?;
        }
        f.write_str(",C,")?;
        // 7. Water temp
        if let Some(v) = self.water_temp_deg {
            write!(f, "{}", v)?;
        }
        f.write_str(",C,")?;
        // 9. Relative humidity
        if let Some(v) = self.rel_humidity {
            write!(f, "{}", v)?;
        }
        f.write_char(',')?;
        // 10. Absolute humidity
        if let Some(v) = self.abs_humidity {
            write!(f, "{}", v)?;
        }
        f.write_char(',')?;
        // 11. Dew point
        if let Some(v) = self.dew_point {
            write!(f, "{}", v)?;
        }
        f.write_str(",C,")?;
        // 13. True wind direction
        if let Some(v) = self.wind_direction_true {
            write!(f, "{}", v)?;
        }
        f.write_str(",T,")?;
        // 15. Magnetic wind direction
        if let Some(v) = self.wind_direction_magnetic {
            write!(f, "{}", v)?;
        }
        f.write_str(",M,")?;
        // 17. Wind speed knots
        if let Some(v) = self.wind_speed_knots {
            write!(f, "{}", v)?;
        }
        f.write_str(",N,")?;
        // 19. Wind speed m/s
        if let Some(v) = self.wind_speed_ms {
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
    fn test_parse_mda() {
        // Partial sentence from AirMax 150 model weather station
        let s = parse_nmea_sentence(
            "$WIMDA,29.7544,I,1.0076,B,35.5,C,,,42.1,,20.6,C,116.4,T,107.7,M,1.2,N,0.6,M*66",
        )
        .unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        assert_eq!(s.checksum, 0x66);
        let mda_data = parse_mda(s).unwrap();
        assert_relative_eq!(29.7544, mda_data.pressure_in_hg.unwrap());
        assert_relative_eq!(1.0076, mda_data.pressure_bar.unwrap());
        assert_relative_eq!(35.5, mda_data.air_temp_deg.unwrap());
        assert!(mda_data.water_temp_deg.is_none());
        assert_relative_eq!(42.1, mda_data.rel_humidity.unwrap());
        assert!(mda_data.abs_humidity.is_none());
        assert_relative_eq!(20.6, mda_data.dew_point.unwrap());
        assert_relative_eq!(116.4, mda_data.wind_direction_true.unwrap());
        assert_relative_eq!(107.7, mda_data.wind_direction_magnetic.unwrap());
        assert_relative_eq!(1.2, mda_data.wind_speed_knots.unwrap());
        assert_relative_eq!(0.6, mda_data.wind_speed_ms.unwrap());
    }

    #[test]
    fn test_generate_mda_roundtrip() {
        let original = MdaData {
            pressure_in_hg: Some(29.7544),
            pressure_bar: Some(1.0076),
            air_temp_deg: Some(35.5),
            water_temp_deg: None,
            rel_humidity: Some(42.1),
            abs_humidity: None,
            dew_point: Some(20.6),
            wind_direction_true: Some(116.4),
            wind_direction_magnetic: Some(107.7),
            wind_speed_knots: Some(1.2),
            wind_speed_ms: Some(0.6),
        };
        let mut buf = heapless::String::<256>::new();
        crate::generate::generate_sentence("WI", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_mda(s).unwrap();
        assert_relative_eq!(parsed.pressure_in_hg.unwrap(), 29.7544);
        assert_relative_eq!(parsed.pressure_bar.unwrap(), 1.0076);
        assert_relative_eq!(parsed.air_temp_deg.unwrap(), 35.5);
        assert!(parsed.water_temp_deg.is_none());
        assert_relative_eq!(parsed.rel_humidity.unwrap(), 42.1);
        assert!(parsed.abs_humidity.is_none());
        assert_relative_eq!(parsed.dew_point.unwrap(), 20.6);
        assert_relative_eq!(parsed.wind_direction_true.unwrap(), 116.4);
        assert_relative_eq!(parsed.wind_direction_magnetic.unwrap(), 107.7);
        assert_relative_eq!(parsed.wind_speed_knots.unwrap(), 1.2);
        assert_relative_eq!(parsed.wind_speed_ms.unwrap(), 0.6);
    }
}
