use nom::{
    IResult, Parser as _,
    character::complete::{char, one_of},
    combinator::opt,
    number::complete::double,
    sequence::preceded,
};

use crate::{Error, ParseResult, SentenceType, parse::NmeaSentence};

/// DBT - Depth Below Transducer
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_dbt_depth_below_transducer>
///
/// ```text
///        1   2 3   4 5   6 7
///        |   | |   | |   | |
/// $--DBT,x.x,f,x.x,M,x.x,F*hh<CR><LF>
/// ```
///
/// 1. Depth, feet
/// 2. `f` = feet
/// 3. Depth, meters
/// 4. `M` = meters
/// 5. Depth, Fathoms
/// 6. `F` = Fathoms
/// 7. Mandatory NMEA checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct DbtData {
    pub depth_feet: Option<f64>,
    pub depth_meters: Option<f64>,
    pub depth_fathoms: Option<f64>,
}

impl From<DbtData> for ParseResult {
    fn from(value: DbtData) -> Self {
        ParseResult::DBT(value)
    }
}

/// # Parse DBT message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_dbt_depth_below_transducer>
pub fn parse_dbt(sentence: NmeaSentence<'_>) -> Result<DbtData, Error<'_>> {
    if sentence.message_id != SentenceType::DBT {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::DBT,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_dbt(sentence.data)?.1)
    }
}

fn do_parse_dbt(i: &str) -> IResult<&str, DbtData> {
    let (i, depth_feet) = opt(double).parse(i)?;
    let (i, _) = preceded(char(','), one_of("f")).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, depth_meters) = opt(double).parse(i)?;
    let (i, _) = preceded(char(','), one_of("M")).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, depth_fathoms) = opt(double).parse(i)?;
    let (i, _) = preceded(char(','), one_of("F")).parse(i)?;
    Ok((
        i,
        DbtData {
            depth_feet,
            depth_meters,
            depth_fathoms,
        },
    ))
}

impl crate::generate::GenerateNmeaBody for DbtData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::DBT
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        super::dbk::write_depth_triple(f, &self.depth_feet, &self.depth_meters, &self.depth_fathoms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_dbt() {
        let s = parse_nmea_sentence("$SDDBT,12.3,f,3.75,M,2.05,F*30").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_dbt(s).unwrap();
        assert_eq!(data.depth_feet, Some(12.3));
        assert_eq!(data.depth_meters, Some(3.75));
        assert_eq!(data.depth_fathoms, Some(2.05));
    }

    #[test]
    fn test_parse_dbt_empty() {
        let s = parse_nmea_sentence("$SDDBT,,f,,M,,F*28").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_dbt(s).unwrap();
        assert_eq!(data.depth_feet, None);
        assert_eq!(data.depth_meters, None);
        assert_eq!(data.depth_fathoms, None);
    }

    #[test]
    fn test_generate_dbt_roundtrip() {
        let original = DbtData {
            depth_feet: Some(12.3),
            depth_meters: Some(3.75),
            depth_fathoms: Some(2.05),
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("SD", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_dbt(s).unwrap();
        assert_eq!(parsed, original);
    }
}
