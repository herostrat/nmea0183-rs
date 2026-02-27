use nom::{
    IResult, Parser as _,
    bytes::complete::take_until,
    character::complete::char,
    combinator::{map_res, opt},
};

use super::utils::parse_float_num;
use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// HDM - Heading - Magnetic
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_hdm_heading_magnetic>
///
/// ```text
///        1   2 3
///        |   | |
/// $--HDM,x.x,M*hh<CR><LF>
/// ```
/// 1. Heading, degrees Magnetic
/// 2. M = Magnetic
/// 3. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq)]
pub struct HdmData {
    pub heading: Option<f32>,
}

impl From<HdmData> for ParseResult {
    fn from(value: HdmData) -> Self {
        ParseResult::HDM(value)
    }
}

/// # Parse HDM message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_hdm_heading_magnetic>
pub fn parse_hdm(sentence: NmeaSentence<'_>) -> Result<HdmData, Error<'_>> {
    if sentence.message_id != SentenceType::HDM {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::HDM,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_hdm(sentence.data)?.1)
    }
}

fn do_parse_hdm(i: &str) -> IResult<&str, HdmData> {
    let (i, heading) = opt(map_res(take_until(","), parse_float_num::<f32>)).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = char('M').parse(i)?;
    Ok((i, HdmData { heading }))
}

impl crate::generate::GenerateNmeaBody for HdmData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::HDM
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        if let Some(h) = self.heading {
            write!(f, "{}", h)?;
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
    fn test_parse_hdm() {
        let s = parse_nmea_sentence("$HCHDM,238.9,M*29").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_hdm(s).unwrap();
        assert_relative_eq!(data.heading.unwrap(), 238.9);
    }

    #[test]
    fn test_parse_hdm_empty() {
        let s = parse_nmea_sentence("$HCHDM,,M*07").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_hdm(s).unwrap();
        assert_eq!(data.heading, None);
    }

    #[test]
    fn test_generate_hdm_roundtrip() {
        let original = HdmData {
            heading: Some(238.9),
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("HC", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_hdm(s).unwrap();
        assert_relative_eq!(parsed.heading.unwrap(), 238.9);
    }
}
