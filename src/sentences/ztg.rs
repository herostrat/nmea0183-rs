use arrayvec::ArrayString;
use chrono::{Duration, NaiveTime};
use nom::{Parser as _, bytes::complete::is_not, character::complete::char, combinator::opt};

#[cfg(feature = "serde")]
use serde_with::As;

use crate::{
    Error, SentenceType,
    parse::{NmeaSentence, TEXT_PARAMETER_MAX_LEN},
    sentences::utils::{parse_duration_hms, parse_hms},
};

use super::utils::array_string;

/// ZTG - UTC & Time to Destination Waypoint
///```text
///        1         2         3    4
///        |         |         |    |
/// $--ZTG,hhmmss.ss,hhmmss.ss,c--c*hh<CR><LF>
///```
/// Field Number:
/// 1. UTC of observation hh is hours, mm is minutes, ss.ss is seconds.
/// 2. Time Remaining
/// 3. Destination Waypoint ID
/// 4. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub struct ZtgData {
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub fix_time: Option<NaiveTime>,
    #[cfg_attr(
        feature = "serde",
        serde(with = "As::<Option<serde_with::DurationSecondsWithFrac<f64>>>")
    )]
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub fix_duration: Option<Duration>,
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub waypoint_id: Option<ArrayString<TEXT_PARAMETER_MAX_LEN>>,
}

fn do_parse_ztg(i: &str) -> Result<ZtgData, Error<'_>> {
    // 1. UTC Time or observation
    let (i, fix_time) = opt(parse_hms).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 2. Duration
    let (i, fix_duration) = opt(parse_duration_hms).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    // 12. Waypoint ID
    let (_i, waypoint_id) = opt(is_not(",*")).parse(i)?;

    let waypoint_id = waypoint_id
        .map(array_string::<TEXT_PARAMETER_MAX_LEN>)
        .transpose()?;

    Ok(ZtgData {
        fix_time,
        fix_duration,
        waypoint_id,
    })
}

/// # Parse ZTG message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_ztg_utc_time_to_destination_waypoint>
pub fn parse_ztg(sentence: NmeaSentence<'_>) -> Result<ZtgData, Error<'_>> {
    if sentence.message_id != SentenceType::ZTG {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::ZTG,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_ztg(sentence.data)?)
    }
}

/// Write a `Duration` in `hhmmss.ss` NMEA format.
fn write_duration(f: &mut dyn core::fmt::Write, dur: &Option<Duration>) -> core::fmt::Result {
    if let Some(d) = dur {
        let total_millis = d.num_milliseconds();
        let hours = total_millis / 3_600_000;
        let minutes = (total_millis % 3_600_000) / 60_000;
        let seconds = (total_millis % 60_000) / 1_000;
        let centiseconds = (total_millis % 1_000) / 10;
        write!(f, "{:02}{:02}{:02}", hours, minutes, seconds)?;
        if centiseconds > 0 {
            write!(f, ".{:02}", centiseconds)?;
        }
    }
    Ok(())
}

impl crate::generate::GenerateNmeaBody for ZtgData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::ZTG
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        // 1. UTC Time
        crate::sentences::gen_utils::write_hms(f, &self.fix_time)?;
        f.write_char(',')?;
        // 2. Time Remaining (Duration)
        write_duration(f, &self.fix_duration)?;
        f.write_char(',')?;
        // 3. Waypoint ID
        if let Some(ref wpt) = self.waypoint_id {
            f.write_str(wpt.as_str())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, parse::parse_nmea_sentence};

    fn run_parse_ztg(line: &str) -> Result<ZtgData, Error<'_>> {
        let s = parse_nmea_sentence(line).expect("ZTG sentence initial parse failed");
        assert_eq!(s.checksum, s.calc_checksum());
        parse_ztg(s)
    }

    #[test]
    fn test_parse_ztg() {
        assert_eq!(
            ZtgData {
                fix_duration: Some(
                    Duration::hours(4)
                        + Duration::minutes(23)
                        + Duration::seconds(59)
                        + Duration::milliseconds(170)
                ),
                fix_time: NaiveTime::from_hms_milli_opt(14, 58, 32, 120),
                waypoint_id: Some(ArrayString::from("WPT").unwrap()),
            },
            run_parse_ztg("$GPZTG,145832.12,042359.17,WPT*24").unwrap()
        );
        assert_eq!(
            ZtgData {
                fix_duration: None,
                fix_time: None,
                waypoint_id: None,
            },
            run_parse_ztg("$GPZTG,,,*72").unwrap()
        );
        assert_eq!(
            ZtgData {
                fix_duration: Some(
                    Duration::hours(4)
                        + Duration::minutes(23)
                        + Duration::seconds(59)
                        + Duration::milliseconds(170)
                ),
                fix_time: None,
                waypoint_id: None,
            },
            run_parse_ztg("$GPZTG,,042359.17,*53").unwrap()
        );
    }
    #[test]
    fn test_generate_ztg_roundtrip() {
        let original = ZtgData {
            fix_time: NaiveTime::from_hms_milli_opt(14, 58, 32, 120),
            fix_duration: Some(
                Duration::hours(4)
                    + Duration::minutes(23)
                    + Duration::seconds(59)
                    + Duration::milliseconds(170),
            ),
            waypoint_id: Some(ArrayString::from("WPT").unwrap()),
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("GP", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_ztg(s).unwrap();
        assert_eq!(parsed.fix_time, original.fix_time);
        assert_eq!(parsed.fix_duration, original.fix_duration);
        assert_eq!(parsed.waypoint_id.as_deref(), Some("WPT"));
    }

    #[test]
    fn test_parse_ztg_with_too_long_waypoint() {
        assert_eq!(
            Error::ParameterLength { max_length: 64, parameter_length: 72 },
            run_parse_ztg("$GPZTG,145832.12,042359.17,ABCDEFGHIJKLMNOPRSTUWXYZABCDEFGHIJKLMNOPRSTUWXYZABCDEFGHIJKLMNOPRSTUWXYZ*6B").unwrap_err()
        );
    }
}
