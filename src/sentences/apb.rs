use arrayvec::ArrayString;
use nom::{
    IResult, Parser as _,
    bytes::complete::is_not,
    character::complete::{char, one_of},
    combinator::opt,
    number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType, parse::TEXT_PARAMETER_MAX_LEN};
use crate::sentences::utils::array_string;

/// APB - Autopilot Sentence "B"
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_apb_autopilot_sentence_b>
///
/// This is a superset of APA and the preferred sentence for autopilot control.
///
/// ```text
///                                         13    15
///        1 2 3   4 5 6 7 8   9 10   11  12|  14|
///        | | |   | | | | |   | |    |   | |  | |
/// $--APB,A,A,x.x,a,N,A,A,x.x,a,c--c,x.x,a,x.x,a*hh<CR><LF>
/// ```
/// 1. Status V = Loran-C Blink or SNR warning, A = general warning
/// 2. Status V = Loran-C Cycle Lock warning, A = OK
/// 3. Cross Track Error Magnitude
/// 4. Direction to steer, L or R
/// 5. Cross Track Units, N = Nautical Miles
/// 6. Status A = Arrival Circle Entered
/// 7. Status A = Perpendicular passed at waypoint
/// 8. Bearing origin to destination
/// 9. M = Magnetic, T = True
/// 10. Destination Waypoint ID
/// 11. Bearing, present position to Destination
/// 12. M = Magnetic, T = True
/// 13. Heading to steer to destination waypoint
/// 14. M = Magnetic, T = True
/// 15. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct ApbData {
    /// General warning status (true = OK)
    pub status_warning: bool,
    /// Cycle lock status (true = OK)
    pub status_cycle_lock: bool,
    /// Cross track error magnitude in nautical miles (negative = steer left)
    pub cross_track_error: Option<f32>,
    /// Arrival circle entered
    pub arrival_circle_entered: bool,
    /// Perpendicular passed at waypoint
    pub perpendicular_passed: bool,
    /// Bearing origin to destination
    pub bearing_orig_to_dest: Option<f32>,
    /// Bearing origin to dest is magnetic (true) or true (false)
    pub bearing_orig_to_dest_magnetic: bool,
    /// Destination waypoint ID
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub waypoint_id: Option<ArrayString<TEXT_PARAMETER_MAX_LEN>>,
    /// Bearing, present position to destination
    pub bearing_pos_to_dest: Option<f32>,
    /// Bearing pos to dest is magnetic (true) or true (false)
    pub bearing_pos_to_dest_magnetic: bool,
    /// Heading to steer to destination
    pub heading_to_dest: Option<f32>,
    /// Heading to dest is magnetic (true) or true (false)
    pub heading_to_dest_magnetic: bool,
}

impl From<ApbData> for ParseResult {
    fn from(value: ApbData) -> Self {
        ParseResult::APB(value)
    }
}

/// # Parse APB message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_apb_autopilot_sentence_b>
pub fn parse_apb(sentence: NmeaSentence<'_>) -> Result<ApbData, Error<'_>> {
    if sentence.message_id != SentenceType::APB {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::APB,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_apb(sentence.data)?.1)
    }
}

fn do_parse_apb(i: &str) -> IResult<&str, ApbData> {
    // 1. Status warning
    let (i, status1) = one_of("AV").parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 2. Status cycle lock
    let (i, status2) = one_of("AV").parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 3. Cross track error magnitude
    let (i, xte_mag) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 4. Direction to steer L/R
    let (i, steer_dir) = opt(one_of("LR")).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 5. Cross track units
    let (i, _) = opt(char('N')).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 6. Arrival circle entered
    let (i, arr_circle) = one_of("AV").parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 7. Perpendicular passed
    let (i, perp_passed) = one_of("AV").parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 8. Bearing origin to destination
    let (i, bearing_orig) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 9. M/T
    let (i, bearing_orig_mt) = opt(one_of("MT")).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 10. Destination waypoint ID
    let (i, wp_id) = opt(is_not(",*")).parse(i)?;
    let wp_id = match wp_id {
        Some("") | None => None,
        Some(s) => Some(
            array_string::<TEXT_PARAMETER_MAX_LEN>(s)
                .map_err(|_| nom::Err::Failure(nom::error::Error::new(i, nom::error::ErrorKind::Fail)))?,
        ),
    };
    let (i, _) = char(',').parse(i)?;
    // 11. Bearing, present position to destination
    let (i, bearing_pos) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 12. M/T
    let (i, bearing_pos_mt) = opt(one_of("MT")).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 13. Heading to steer
    let (i, heading) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 14. M/T
    let (i, heading_mt) = opt(one_of("MT")).parse(i)?;
    // Optional FAA mode (NMEA 2.3+)
    let (i, _) = opt(|i| {
        let (i, _) = char(',').parse(i)?;
        nom::character::complete::anychar(i)
    })
    .parse(i)?;

    let cross_track_error = match (xte_mag, steer_dir) {
        (Some(m), Some('R')) => Some(-m),
        (Some(m), _) => Some(m),
        _ => None,
    };

    Ok((
        i,
        ApbData {
            status_warning: status1 == 'A',
            status_cycle_lock: status2 == 'A',
            cross_track_error,
            arrival_circle_entered: arr_circle == 'A',
            perpendicular_passed: perp_passed == 'A',
            bearing_orig_to_dest: bearing_orig,
            bearing_orig_to_dest_magnetic: bearing_orig_mt == Some('M'),
            waypoint_id: wp_id,
            bearing_pos_to_dest: bearing_pos,
            bearing_pos_to_dest_magnetic: bearing_pos_mt == Some('M'),
            heading_to_dest: heading,
            heading_to_dest_magnetic: heading_mt == Some('M'),
        },
    ))
}

impl crate::generate::GenerateNmeaBody for ApbData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::APB
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        // 1. Status warning
        f.write_char(if self.status_warning { 'A' } else { 'V' })?;
        f.write_char(',')?;
        // 2. Status cycle lock
        f.write_char(if self.status_cycle_lock { 'A' } else { 'V' })?;
        f.write_char(',')?;
        // 3-4. Cross track error magnitude and direction
        match self.cross_track_error {
            Some(v) if v < 0.0 => {
                write!(f, "{}", -v)?;
                f.write_str(",R")?;
            }
            Some(v) => {
                write!(f, "{}", v)?;
                f.write_str(",L")?;
            }
            None => f.write_char(',')?,
        }
        f.write_char(',')?;
        // 5. Cross track units (always N)
        f.write_char('N')?;
        f.write_char(',')?;
        // 6. Arrival circle entered
        f.write_char(if self.arrival_circle_entered { 'A' } else { 'V' })?;
        f.write_char(',')?;
        // 7. Perpendicular passed
        f.write_char(if self.perpendicular_passed { 'A' } else { 'V' })?;
        f.write_char(',')?;
        // 8. Bearing origin to dest
        if let Some(v) = self.bearing_orig_to_dest {
            write!(f, "{}", v)?;
        }
        f.write_char(',')?;
        // 9. M/T
        f.write_char(if self.bearing_orig_to_dest_magnetic { 'M' } else { 'T' })?;
        f.write_char(',')?;
        // 10. Waypoint ID
        if let Some(ref wp) = self.waypoint_id {
            f.write_str(wp)?;
        }
        f.write_char(',')?;
        // 11. Bearing present position to dest
        if let Some(v) = self.bearing_pos_to_dest {
            write!(f, "{}", v)?;
        }
        f.write_char(',')?;
        // 12. M/T
        f.write_char(if self.bearing_pos_to_dest_magnetic { 'M' } else { 'T' })?;
        f.write_char(',')?;
        // 13. Heading to dest
        if let Some(v) = self.heading_to_dest {
            write!(f, "{}", v)?;
        }
        f.write_char(',')?;
        // 14. M/T
        f.write_char(if self.heading_to_dest_magnetic { 'M' } else { 'T' })
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_apb() {
        let s = parse_nmea_sentence(
            "$GPAPB,A,A,0.10,R,N,V,V,011,M,DEST,011,M,011,M*3C",
        )
        .unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_apb(s).unwrap();
        assert!(data.status_warning);
        assert!(data.status_cycle_lock);
        assert_relative_eq!(data.cross_track_error.unwrap(), -0.10); // R = negative
        assert!(!data.arrival_circle_entered);
        assert!(!data.perpendicular_passed);
        assert_relative_eq!(data.bearing_orig_to_dest.unwrap(), 11.0);
        assert!(data.bearing_orig_to_dest_magnetic);
        assert_eq!(&data.waypoint_id.unwrap(), "DEST");
        assert_relative_eq!(data.bearing_pos_to_dest.unwrap(), 11.0);
        assert!(data.bearing_pos_to_dest_magnetic);
        assert_relative_eq!(data.heading_to_dest.unwrap(), 11.0);
        assert!(data.heading_to_dest_magnetic);
    }

    #[test]
    fn test_parse_apb_empty() {
        let s = parse_nmea_sentence("$GPAPB,,,,,,,,,,,,,,*44").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        // This should fail because status fields require A/V
        assert!(parse_apb(s).is_err());
    }

    #[test]
    fn test_generate_apb_roundtrip() {
        let original = ApbData {
            status_warning: true,
            status_cycle_lock: true,
            cross_track_error: Some(-0.1),
            arrival_circle_entered: false,
            perpendicular_passed: false,
            bearing_orig_to_dest: Some(11.0),
            bearing_orig_to_dest_magnetic: true,
            waypoint_id: Some(ArrayString::from("DEST").unwrap()),
            bearing_pos_to_dest: Some(11.0),
            bearing_pos_to_dest_magnetic: true,
            heading_to_dest: Some(11.0),
            heading_to_dest_magnetic: true,
        };
        let mut buf = heapless::String::<256>::new();
        crate::generate::generate_sentence("GP", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_apb(s).unwrap();
        assert!(parsed.status_warning);
        assert!(parsed.status_cycle_lock);
        assert_relative_eq!(parsed.cross_track_error.unwrap(), -0.1);
        assert!(!parsed.arrival_circle_entered);
        assert!(!parsed.perpendicular_passed);
        assert_relative_eq!(parsed.bearing_orig_to_dest.unwrap(), 11.0);
        assert!(parsed.bearing_orig_to_dest_magnetic);
        assert_eq!(&parsed.waypoint_id.unwrap(), "DEST");
        assert_relative_eq!(parsed.bearing_pos_to_dest.unwrap(), 11.0);
        assert!(parsed.bearing_pos_to_dest_magnetic);
        assert_relative_eq!(parsed.heading_to_dest.unwrap(), 11.0);
        assert!(parsed.heading_to_dest_magnetic);
    }
}
