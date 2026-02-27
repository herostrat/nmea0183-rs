use nom::{
    IResult, Parser as _,
    character::complete::{char, one_of},
    combinator::opt,
    number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// RPM - Revolutions
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_rpm_revolutions>
///
/// ```text
///        1 2 3   4   5 6
///        | | |   |   | |
/// $--RPM,a,x,x.x,x.x,A*hh<CR><LF>
/// ```
/// 1. Source, S = Shaft, E = Engine
/// 2. Engine or shaft number
/// 3. Speed, Revolutions per minute
/// 4. Propeller pitch, % of maximum, negative means astern
/// 5. Status: A = data is valid
/// 6. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RpmData {
    /// Source: Shaft or Engine
    pub source: Option<RpmSource>,
    /// Engine or shaft number
    pub source_number: Option<u8>,
    /// Speed in revolutions per minute
    pub rpm: Option<f32>,
    /// Propeller pitch (% of max, negative = astern)
    pub pitch: Option<f32>,
    /// Data validity status
    pub valid: bool,
}

/// RPM source type
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpmSource {
    Shaft,
    Engine,
}

impl From<RpmData> for ParseResult {
    fn from(value: RpmData) -> Self {
        ParseResult::RPM(value)
    }
}

/// # Parse RPM message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_rpm_revolutions>
pub fn parse_rpm(sentence: NmeaSentence<'_>) -> Result<RpmData, Error<'_>> {
    if sentence.message_id != SentenceType::RPM {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::RPM,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_rpm(sentence.data)?.1)
    }
}

fn do_parse_rpm(i: &str) -> IResult<&str, RpmData> {
    let (i, source) = opt(one_of("SE")).parse(i)?;
    let source = source.map(|ch| match ch {
        'S' => RpmSource::Shaft,
        'E' => RpmSource::Engine,
        _ => unreachable!(),
    });
    let (i, _) = char(',').parse(i)?;
    let (i, source_number) = opt(nom::character::complete::u8).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, rpm) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, pitch) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, status) = opt(one_of("AV")).parse(i)?;
    let valid = status == Some('A');

    Ok((
        i,
        RpmData {
            source,
            source_number,
            rpm,
            pitch,
            valid,
        },
    ))
}

impl crate::generate::GenerateNmeaBody for RpmData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::RPM
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        match self.source {
            Some(RpmSource::Shaft) => f.write_str("S")?,
            Some(RpmSource::Engine) => f.write_str("E")?,
            None => {}
        }
        f.write_str(",")?;
        if let Some(n) = self.source_number {
            write!(f, "{}", n)?;
        }
        f.write_str(",")?;
        if let Some(r) = self.rpm {
            write!(f, "{}", r)?;
        }
        f.write_str(",")?;
        if let Some(p) = self.pitch {
            write!(f, "{}", p)?;
        }
        f.write_str(",")?;
        if self.valid {
            f.write_str("A")
        } else {
            f.write_str("V")
        }
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_rpm() {
        let s = parse_nmea_sentence("$IIRPM,E,1,2418.2,10.5,A*5F").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_rpm(s).unwrap();
        assert_eq!(data.source, Some(RpmSource::Engine));
        assert_eq!(data.source_number, Some(1));
        assert_relative_eq!(data.rpm.unwrap(), 2418.2);
        assert_relative_eq!(data.pitch.unwrap(), 10.5);
        assert!(data.valid);
    }

    #[test]
    fn test_parse_rpm_shaft() {
        let s = parse_nmea_sentence("$IIRPM,S,2,1200.0,50.0,A*45").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_rpm(s).unwrap();
        assert_eq!(data.source, Some(RpmSource::Shaft));
        assert_eq!(data.source_number, Some(2));
        assert_relative_eq!(data.rpm.unwrap(), 1200.0);
        assert_relative_eq!(data.pitch.unwrap(), 50.0);
        assert!(data.valid);
    }

    #[test]
    fn test_generate_rpm_roundtrip() {
        let original = RpmData {
            source: Some(RpmSource::Engine),
            source_number: Some(1),
            rpm: Some(2418.2),
            pitch: Some(10.5),
            valid: true,
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("II", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_rpm(s).unwrap();
        assert_eq!(parsed.source, Some(RpmSource::Engine));
        assert_relative_eq!(parsed.rpm.unwrap(), 2418.2);
    }
}
