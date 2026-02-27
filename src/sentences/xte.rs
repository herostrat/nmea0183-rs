use nom::{
    IResult, Parser as _,
    character::complete::{char, one_of},
    combinator::opt,
    number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// XTE - Cross-Track Error, Measured
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_xte_cross_track_error_measured>
///
/// ```text
///        1 2 3   4 5 6
///        | | |   | | |
/// $--XTE,A,A,x.x,a,N*hh<CR><LF>
/// ```
///
/// NMEA 2.3+:
/// ```text
/// $--XTE,A,A,x.x,a,N,m*hh<CR><LF>
/// ```
///
/// 1. Status: V = LORAN-C Blink or SNR warning, A = general warning flag or target data
/// 2. Status: V = Loran-C Cycle Lock warning flag, A = OK
/// 3. Cross Track Error Magnitude
/// 4. Direction to steer, L or R
/// 5. Units, N = Nautical Miles
/// 6. FAA mode indicator (NMEA 2.3+)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XteData {
    /// Cross track error magnitude in nautical miles.
    /// Positive means steer right, negative means steer left.
    pub cross_track_error: Option<f32>,
    /// General status (true = valid)
    pub status_general: bool,
    /// Cycle lock status (true = valid)
    pub status_cycle_lock: bool,
}

impl From<XteData> for ParseResult {
    fn from(value: XteData) -> Self {
        ParseResult::XTE(value)
    }
}

/// # Parse XTE message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_xte_cross_track_error_measured>
pub fn parse_xte(sentence: NmeaSentence<'_>) -> Result<XteData, Error<'_>> {
    if sentence.message_id != SentenceType::XTE {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::XTE,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_xte(sentence.data)?.1)
    }
}

fn do_parse_xte(i: &str) -> IResult<&str, XteData> {
    // 1. Status general
    let (i, status1) = one_of("AV").parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 2. Status cycle lock
    let (i, status2) = one_of("AV").parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 3. Cross track error magnitude
    let (i, magnitude) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 4. Direction to steer L/R
    let (i, direction) = opt(one_of("LR")).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 5. Units (N)
    let (i, _) = opt(char('N')).parse(i)?;
    // 6. Optional FAA mode (NMEA 2.3+)
    let (i, _) = opt(|i| {
        let (i, _) = char(',').parse(i)?;
        nom::character::complete::anychar(i)
    })
    .parse(i)?;

    let cross_track_error = match (magnitude, direction) {
        (Some(m), Some('L')) => Some(-m),
        (Some(m), _) => Some(m),
        _ => None,
    };

    Ok((
        i,
        XteData {
            cross_track_error,
            status_general: status1 == 'A',
            status_cycle_lock: status2 == 'A',
        },
    ))
}

impl crate::generate::GenerateNmeaBody for XteData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::XTE
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        f.write_char(if self.status_general { 'A' } else { 'V' })?;
        f.write_char(',')?;
        f.write_char(if self.status_cycle_lock { 'A' } else { 'V' })?;
        f.write_char(',')?;
        match self.cross_track_error {
            Some(v) if v < 0.0 => {
                write!(f, "{}", -v)?;
                f.write_str(",L")
            }
            Some(v) => {
                write!(f, "{}", v)?;
                f.write_str(",R")
            }
            None => f.write_char(','),
        }?;
        f.write_str(",N")
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_xte() {
        let s = parse_nmea_sentence("$GPXTE,A,A,0.67,L,N*6F").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_xte(s).unwrap();
        assert_relative_eq!(data.cross_track_error.unwrap(), -0.67);
        assert!(data.status_general);
        assert!(data.status_cycle_lock);
    }

    #[test]
    fn test_parse_xte_right() {
        let s = parse_nmea_sentence("$GPXTE,A,A,1.50,R,N*74").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_xte(s).unwrap();
        assert_relative_eq!(data.cross_track_error.unwrap(), 1.50);
    }

    #[test]
    fn test_parse_xte_with_faa_mode() {
        let s = parse_nmea_sentence("$GPXTE,A,A,0.67,L,N,D*07").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_xte(s).unwrap();
        assert_relative_eq!(data.cross_track_error.unwrap(), -0.67);
    }

    #[test]
    fn test_generate_xte_roundtrip() {
        let original = XteData {
            cross_track_error: Some(-0.67),
            status_general: true,
            status_cycle_lock: true,
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("GP", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_xte(s).unwrap();
        assert_relative_eq!(parsed.cross_track_error.unwrap(), -0.67);
        assert!(parsed.status_general);
    }
}
