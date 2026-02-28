use nom::{Parser as _, character::complete::char, combinator::opt, number::complete::float};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// DBS - Depth Below Surface
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_dbs_depth_below_surface>
/// ```text
///         1   2 3   4 5   6 7
///         |   | |   | |   | |
///  $--DBS,x.x,f,x.x,M,x.x,F*hh<CR><LF>
/// ```
///
/// Field Number:
/// 1. Water depth, feet
/// 2. `f` = feet
/// 3. Water depth, meters
/// 4. `M` = meters
/// 5. Water depth, Fathoms
/// 6. `F` = Fathoms
/// 7. Checksum
///
/// In real-world sensors, sometimes not all three conversions are reported.
/// So you might see something like `$SDDBS,,f,22.5,M,,F*cs`
///
/// Examples:
/// * `$DBS,x.x,f,x.x,M,x.x,F*hh<CR><LF>`
///
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct DbsData {
    pub water_depth_feet: Option<f32>,
    pub water_depth_meters: Option<f32>,
    pub water_depth_fathoms: Option<f32>,
}

impl From<DbsData> for ParseResult {
    fn from(value: DbsData) -> Self {
        ParseResult::DBS(value)
    }
}

pub fn parse_dbs(sentence: NmeaSentence<'_>) -> Result<DbsData, Error<'_>> {
    if sentence.message_id != SentenceType::DBS {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::DBS,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_dbs(sentence.data)?)
    }
}

fn do_parse_dbs(i: &str) -> Result<DbsData, Error<'_>> {
    let (i, water_depth_feet) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = char('f').parse(i)?;
    let (i, _) = char(',').parse(i)?;

    let (i, water_depth_meters) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = char('M').parse(i)?;
    let (i, _) = char(',').parse(i)?;

    let (i, water_depth_fathoms) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (_, _) = char('F').parse(i)?;

    if water_depth_feet.is_none() && water_depth_meters.is_none() && water_depth_fathoms.is_none() {
        return Err(Error::Unknown(
            "No water depth data available any conversion",
        ));
    }

    Ok(DbsData {
        water_depth_feet,
        water_depth_meters,
        water_depth_fathoms,
    })
}

impl crate::generate::GenerateNmeaBody for DbsData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::DBS
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        if let Some(v) = self.water_depth_feet {
            write!(f, "{}", v)?;
        }
        f.write_str(",f,")?;
        if let Some(v) = self.water_depth_meters {
            write!(f, "{}", v)?;
        }
        f.write_str(",M,")?;
        if let Some(v) = self.water_depth_fathoms {
            write!(f, "{}", v)?;
        }
        f.write_str(",F")
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::parse_nmea_sentence;

    use super::*;
    #[test]
    fn parse_dbs_with_full_sentence() {
        let sentence = parse_nmea_sentence("$SDDBS,45.0,f,13.7,M,7.5,F*68").unwrap();
        let data = parse_dbs(sentence).unwrap();
        assert_relative_eq!(data.water_depth_feet.unwrap(), 45.0);
        assert_relative_eq!(data.water_depth_meters.unwrap(), 13.7);
        assert_relative_eq!(data.water_depth_fathoms.unwrap(), 7.5);
    }

    #[test]
    fn test_invalid_sentence() {
        let sentence = parse_nmea_sentence("$SDDBS,,,M,F*68").unwrap();
        let result = parse_dbs(sentence);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_dbs_roundtrip() {
        let original = DbsData {
            water_depth_feet: Some(45.0),
            water_depth_meters: Some(13.7),
            water_depth_fathoms: Some(7.5),
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("SD", &original, &mut buf).unwrap();

        let sentence = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(sentence.checksum, sentence.calc_checksum());
        let parsed = parse_dbs(sentence).unwrap();
        assert_relative_eq!(parsed.water_depth_feet.unwrap(), 45.0);
        assert_relative_eq!(parsed.water_depth_meters.unwrap(), 13.7);
        assert_relative_eq!(parsed.water_depth_fathoms.unwrap(), 7.5);
    }
}
