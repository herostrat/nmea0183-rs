use nom::{
    IResult, Parser as _,
    character::complete::char,
    combinator::opt,
    number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// HDG - Heading - Deviation & Variation
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_hdg_heading_deviation_variation>
///
/// ```text
///        1   2   3 4   5 6
///        |   |   | |   | |
/// $--HDG,x.x,x.x,a,x.x,a*hh<CR><LF>
/// ```
/// 1. Magnetic Sensor heading in degrees
/// 2. Magnetic Deviation, degrees
/// 3. Magnetic Deviation direction, E = Easterly, W = Westerly
/// 4. Magnetic Variation degrees
/// 5. Magnetic Variation direction, E = Easterly, W = Westerly
/// 6. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HdgData {
    /// Magnetic sensor heading in degrees
    pub heading: Option<f32>,
    /// Magnetic deviation in degrees (positive = East, negative = West)
    pub deviation: Option<f32>,
    /// Magnetic variation in degrees (positive = East, negative = West)
    pub variation: Option<f32>,
}

impl From<HdgData> for ParseResult {
    fn from(value: HdgData) -> Self {
        ParseResult::HDG(value)
    }
}

/// # Parse HDG message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_hdg_heading_deviation_variation>
pub fn parse_hdg(sentence: NmeaSentence<'_>) -> Result<HdgData, Error<'_>> {
    if sentence.message_id != SentenceType::HDG {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::HDG,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_hdg(sentence.data)?.1)
    }
}

/// Parse a float value with E/W direction indicator. W negates the value.
fn parse_ew_float(i: &str) -> IResult<&str, Option<f32>> {
    let (i, val) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, dir) = opt(nom::character::complete::one_of("EW")).parse(i)?;
    match (val, dir) {
        (Some(v), Some('W')) => Ok((i, Some(-v))),
        (Some(v), _) => Ok((i, Some(v))),
        _ => Ok((i, None)),
    }
}

fn do_parse_hdg(i: &str) -> IResult<&str, HdgData> {
    let (i, heading) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, deviation) = parse_ew_float(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, variation) = parse_ew_float(i)?;
    Ok((
        i,
        HdgData {
            heading,
            deviation,
            variation,
        },
    ))
}

/// Write a float value with E/W direction.
fn write_ew(f: &mut dyn core::fmt::Write, val: &Option<f32>) -> core::fmt::Result {
    match val {
        Some(v) if *v < 0.0 => {
            write!(f, "{}", -v)?;
            f.write_str(",W")
        }
        Some(v) => {
            write!(f, "{}", v)?;
            f.write_str(",E")
        }
        None => f.write_char(','),
    }
}

impl crate::generate::GenerateNmeaBody for HdgData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::HDG
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        if let Some(h) = self.heading {
            write!(f, "{}", h)?;
        }
        f.write_char(',')?;
        write_ew(f, &self.deviation)?;
        f.write_char(',')?;
        write_ew(f, &self.variation)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_hdg() {
        let s = parse_nmea_sentence("$HCHDG,98.3,0.0,E,12.6,W*57").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_hdg(s).unwrap();
        assert_relative_eq!(data.heading.unwrap(), 98.3);
        assert_relative_eq!(data.deviation.unwrap(), 0.0);
        assert_relative_eq!(data.variation.unwrap(), -12.6);
    }

    #[test]
    fn test_parse_hdg_minimal() {
        let s = parse_nmea_sentence("$HCHDG,98.3,,,,*70").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_hdg(s).unwrap();
        assert_relative_eq!(data.heading.unwrap(), 98.3);
        assert_eq!(data.deviation, None);
        assert_eq!(data.variation, None);
    }

    #[test]
    fn test_generate_hdg_roundtrip() {
        let original = HdgData {
            heading: Some(98.3),
            deviation: Some(0.0),
            variation: Some(-12.6),
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("HC", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_hdg(s).unwrap();
        assert_relative_eq!(parsed.heading.unwrap(), 98.3);
        // deviation 0.0 gets written as "0,E", parsed back as 0.0 (East)
        assert_relative_eq!(parsed.deviation.unwrap(), 0.0);
        assert_relative_eq!(parsed.variation.unwrap(), -12.6);
    }
}
