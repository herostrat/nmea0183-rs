use arrayvec::ArrayString;
use nom::{
    IResult, Parser as _,
    bytes::complete::is_not,
    character::complete::{char, one_of},
    combinator::opt,
    number::complete::float,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType, parse::TEXT_PARAMETER_MAX_LEN};
use crate::sentences::utils::{array_string, parse_lat_lon};

/// RMB - Recommended Minimum Navigation Information
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_rmb_recommended_minimum_navigation_information>
///
/// ```text
///                                                              14
///        1 2   3 4    5    6       7 8        9 10  11  12  13 |
///        | |   | |    |    |       | |        | |   |   |   |  |
/// $--RMB,A,x.x,a,c--c,c--c,llll.ll,a,yyyyy.yy,a,x.x,x.x,x.x,A,m*hh<CR><LF>
/// ```
/// 1. Status: A=Active, V=Void (warning)
/// 2. Cross track error, nautical miles
/// 3. Direction to steer, L/R
/// 4. Origin waypoint ID
/// 5. Destination waypoint ID
/// 6. Destination waypoint latitude
/// 7. N/S
/// 8. Destination waypoint longitude
/// 9. E/W
/// 10. Range to destination, nautical miles
/// 11. Bearing to destination, degrees true
/// 12. Destination closing velocity, knots
/// 13. Arrival status: A=Arrived, V=Not arrived
/// 14. FAA mode indicator (NMEA 2.3+)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct RmbData {
    /// Data status: true = active, false = void
    pub status: bool,
    /// Cross track error in nautical miles (negative = steer left)
    pub cross_track_error: Option<f32>,
    /// Origin waypoint ID
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub origin_waypoint_id: Option<ArrayString<TEXT_PARAMETER_MAX_LEN>>,
    /// Destination waypoint ID
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub dest_waypoint_id: Option<ArrayString<TEXT_PARAMETER_MAX_LEN>>,
    /// Destination waypoint latitude (decimal degrees, positive = north)
    pub dest_latitude: Option<f64>,
    /// Destination waypoint longitude (decimal degrees, positive = east)
    pub dest_longitude: Option<f64>,
    /// Range to destination in nautical miles
    pub range_to_dest: Option<f32>,
    /// Bearing to destination, degrees true
    pub bearing_to_dest: Option<f32>,
    /// Destination closing velocity in knots
    pub closing_velocity: Option<f32>,
    /// Arrival status: true = arrived
    pub arrived: bool,
}

impl From<RmbData> for ParseResult {
    fn from(value: RmbData) -> Self {
        ParseResult::RMB(value)
    }
}

/// # Parse RMB message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_rmb_recommended_minimum_navigation_information>
pub fn parse_rmb(sentence: NmeaSentence<'_>) -> Result<RmbData, Error<'_>> {
    if sentence.message_id != SentenceType::RMB {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::RMB,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_rmb(sentence.data)?.1)
    }
}

fn parse_waypoint_id(i: &str) -> IResult<&str, Option<ArrayString<TEXT_PARAMETER_MAX_LEN>>> {
    let (i, id) = opt(is_not(",*")).parse(i)?;
    match id {
        Some("") | None => Ok((i, None)),
        Some(s) => {
            let arr = array_string::<TEXT_PARAMETER_MAX_LEN>(s)
                .map_err(|_| nom::Err::Failure(nom::error::Error::new(i, nom::error::ErrorKind::Fail)))?;
            Ok((i, Some(arr)))
        }
    }
}

fn do_parse_rmb(i: &str) -> IResult<&str, RmbData> {
    // 1. Status
    let (i, status) = one_of("AV").parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 2. Cross track error
    let (i, xte_magnitude) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 3. Direction to steer
    let (i, steer_dir) = opt(one_of("LR")).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 4. Origin waypoint ID
    let (i, origin_id) = parse_waypoint_id(i)?;
    let (i, _) = char(',').parse(i)?;
    // 5. Destination waypoint ID
    let (i, dest_id) = parse_waypoint_id(i)?;
    let (i, _) = char(',').parse(i)?;
    // 6-9. Destination lat/lon
    let (i, dest_lat_lon) = parse_lat_lon(i)?;
    let (i, _) = char(',').parse(i)?;
    // 10. Range to destination
    let (i, range) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 11. Bearing to destination
    let (i, bearing) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 12. Closing velocity
    let (i, velocity) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 13. Arrival status
    let (i, arrival) = opt(one_of("AV")).parse(i)?;
    // 14. Optional FAA mode (NMEA 2.3+)
    let (i, _) = opt(|i| {
        let (i, _) = char(',').parse(i)?;
        nom::character::complete::anychar(i)
    })
    .parse(i)?;

    let cross_track_error = match (xte_magnitude, steer_dir) {
        (Some(m), Some('R')) => Some(-m), // steer right = negative XTE
        (Some(m), _) => Some(m),
        _ => None,
    };

    Ok((
        i,
        RmbData {
            status: status == 'A',
            cross_track_error,
            origin_waypoint_id: origin_id,
            dest_waypoint_id: dest_id,
            dest_latitude: dest_lat_lon.map(|ll| ll.0),
            dest_longitude: dest_lat_lon.map(|ll| ll.1),
            range_to_dest: range,
            bearing_to_dest: bearing,
            closing_velocity: velocity,
            arrived: arrival == Some('A'),
        },
    ))
}

impl crate::generate::GenerateNmeaBody for RmbData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::RMB
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        use crate::sentences::gen_utils::write_lat_lon;

        // 1. Status
        f.write_char(if self.status { 'A' } else { 'V' })?;
        f.write_char(',')?;
        // 2-3. Cross track error magnitude and direction
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
        // 4. Origin waypoint ID
        if let Some(ref wp) = self.origin_waypoint_id {
            f.write_str(wp)?;
        }
        f.write_char(',')?;
        // 5. Destination waypoint ID
        if let Some(ref wp) = self.dest_waypoint_id {
            f.write_str(wp)?;
        }
        f.write_char(',')?;
        // 6-9. Destination lat/lon
        write_lat_lon(f, &self.dest_latitude, &self.dest_longitude)?;
        f.write_char(',')?;
        // 10. Range to destination
        if let Some(v) = self.range_to_dest {
            write!(f, "{}", v)?;
        }
        f.write_char(',')?;
        // 11. Bearing to destination
        if let Some(v) = self.bearing_to_dest {
            write!(f, "{}", v)?;
        }
        f.write_char(',')?;
        // 12. Closing velocity
        if let Some(v) = self.closing_velocity {
            write!(f, "{}", v)?;
        }
        f.write_char(',')?;
        // 13. Arrival status
        f.write_char(if self.arrived { 'A' } else { 'V' })
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_rmb() {
        let s = parse_nmea_sentence(
            "$ECRMB,A,0.000,L,001,002,4653.550,N,07115.984,W,2.505,334.205,0.000,V*04",
        )
        .unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_rmb(s).unwrap();
        assert!(data.status);
        assert_relative_eq!(data.cross_track_error.unwrap(), 0.0);
        assert_eq!(&data.origin_waypoint_id.unwrap(), "001");
        assert_eq!(&data.dest_waypoint_id.unwrap(), "002");
        assert_relative_eq!(data.dest_latitude.unwrap(), 46.8925, epsilon = 0.001);
        assert_relative_eq!(data.dest_longitude.unwrap(), -71.2664, epsilon = 0.001);
        assert_relative_eq!(data.range_to_dest.unwrap(), 2.505);
        assert_relative_eq!(data.bearing_to_dest.unwrap(), 334.205);
        assert_relative_eq!(data.closing_velocity.unwrap(), 0.0);
        assert!(!data.arrived);
    }

    #[test]
    fn test_parse_rmb_steer_right() {
        let s = parse_nmea_sentence(
            "$ECRMB,A,0.432,R,001,002,4653.550,N,07115.984,W,2.505,334.205,0.000,V*1F",
        )
        .unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_rmb(s).unwrap();
        assert_relative_eq!(data.cross_track_error.unwrap(), -0.432); // right = negative
    }

    #[test]
    fn test_parse_rmb_from_hooks() {
        let s = parse_nmea_sentence(
            "$GPRMB,A,0.66,L,003,004,4917.24,N,12309.57,W,001.3,052.5,000.5,V*20",
        )
        .unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_rmb(s).unwrap();
        assert!(data.status);
        assert_relative_eq!(data.cross_track_error.unwrap(), 0.66);
        assert_relative_eq!(data.range_to_dest.unwrap(), 1.3);
    }

    #[test]
    fn test_generate_rmb_roundtrip() {
        let original = RmbData {
            status: true,
            cross_track_error: Some(0.5),
            origin_waypoint_id: Some(ArrayString::from("001").unwrap()),
            dest_waypoint_id: Some(ArrayString::from("002").unwrap()),
            dest_latitude: Some(46.8925),
            dest_longitude: Some(-71.2664),
            range_to_dest: Some(2.505),
            bearing_to_dest: Some(334.2),
            closing_velocity: Some(0.5),
            arrived: false,
        };
        let mut buf = heapless::String::<256>::new();
        crate::generate::generate_sentence("GP", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_rmb(s).unwrap();
        assert!(parsed.status);
        assert_relative_eq!(parsed.cross_track_error.unwrap(), 0.5);
        assert_eq!(&parsed.origin_waypoint_id.unwrap(), "001");
        assert_eq!(&parsed.dest_waypoint_id.unwrap(), "002");
        assert_relative_eq!(parsed.dest_latitude.unwrap(), 46.8925, epsilon = 0.001);
        assert_relative_eq!(parsed.dest_longitude.unwrap(), -71.2664, epsilon = 0.001);
        assert_relative_eq!(parsed.range_to_dest.unwrap(), 2.505);
        assert_relative_eq!(parsed.bearing_to_dest.unwrap(), 334.2);
        assert_relative_eq!(parsed.closing_velocity.unwrap(), 0.5);
        assert!(!parsed.arrived);
    }
}
