use nom::{
    IResult, Parser as _, character::complete::char, combinator::opt, number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// HSC - Heading Steering Command
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_hsc_heading_steering_command>
///
/// ```text
///        1   2 3   4 5
///        |   | |   | |
/// $--HSC,x.x,T,x.x,M*hh<CR><LF>
/// ```
/// 1. Heading To Steer, degrees True
/// 2. T = True
/// 3. Heading To Steer, degrees Magnetic
/// 4. M = Magnetic
/// 5. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HscData {
    /// Heading to steer, degrees True
    pub heading_true: Option<f32>,
    /// Heading to steer, degrees Magnetic
    pub heading_magnetic: Option<f32>,
}

impl From<HscData> for ParseResult {
    fn from(value: HscData) -> Self {
        ParseResult::HSC(value)
    }
}

/// # Parse HSC message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_hsc_heading_steering_command>
pub fn parse_hsc(sentence: NmeaSentence<'_>) -> Result<HscData, Error<'_>> {
    if sentence.message_id != SentenceType::HSC {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::HSC,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_hsc(sentence.data)?.1)
    }
}

fn do_parse_hsc(i: &str) -> IResult<&str, HscData> {
    let (i, heading_true) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('T')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, heading_magnetic) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, _) = opt(char('M')).parse(i)?;
    Ok((
        i,
        HscData {
            heading_true,
            heading_magnetic,
        },
    ))
}

impl crate::generate::GenerateNmeaBody for HscData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::HSC
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        if let Some(v) = self.heading_true {
            write!(f, "{}", v)?;
        }
        f.write_str(",T,")?;
        if let Some(v) = self.heading_magnetic {
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
    fn test_parse_hsc() {
        let s = parse_nmea_sentence("$GPHSC,128.5,T,135.2,M*5D").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_hsc(s).unwrap();
        assert_relative_eq!(data.heading_true.unwrap(), 128.5);
        assert_relative_eq!(data.heading_magnetic.unwrap(), 135.2);
    }

    #[test]
    fn test_parse_hsc_empty() {
        let s = parse_nmea_sentence("$GPHSC,,T,,M*56").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_hsc(s).unwrap();
        assert_eq!(data.heading_true, None);
        assert_eq!(data.heading_magnetic, None);
    }

    #[test]
    fn test_generate_hsc_roundtrip() {
        let original = HscData {
            heading_true: Some(128.5),
            heading_magnetic: Some(135.2),
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("GP", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_hsc(s).unwrap();
        assert_relative_eq!(parsed.heading_true.unwrap(), 128.5);
        assert_relative_eq!(parsed.heading_magnetic.unwrap(), 135.2);
    }
}
