use chrono::NaiveTime;
use nom::{
    IResult, Parser as _,
    character::complete::char,
    combinator::opt,
    number::complete::{double, float},
};

use crate::{
    Error, SentenceType,
    parse::NmeaSentence,
    sentences::utils::{number, parse_hms},
};

/// GBS - GPS Satellite Fault Detection
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_gbs_gps_satellite_fault_detection>
///
/// ```text
/// 1      2   3   4   5   6   7   8   9
/// |      |   |   |   |   |   |   |   |
/// $--GBS,hhmmss.ss,x.x,x.x,x.x,x.x,x.x,x.x,x.x*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GbsData {
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub time: Option<NaiveTime>,
    pub lat_error: Option<f64>,
    pub lon_error: Option<f64>,
    pub alt_error: Option<f32>,
    pub most_likely_failed_sat: Option<u8>,
    pub missed_probability: Option<f32>,
    pub bias_estimate: Option<f32>,
    pub bias_standard_deviation: Option<f32>,
}

/// GBS - GPS Satellite Fault Detection
///
/// ```text
/// 1      2   3   4   5   6   7   8   9
/// |      |   |   |   |   |   |   |   |
/// $--GBS,hhmmss.ss,x.x,x.x,x.x,x.x,x.x,x.x,x.x*hh<CR><LF>
/// ```
fn do_parse_gbs(i: &str) -> IResult<&str, GbsData> {
    // 1. UTC time of the GGA or GNS fix associated with this sentence. hh is hours, mm is minutes, ss.ss is seconds
    let (i, time) = opt(parse_hms).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    // 2. Expected 1-sigma error in latitude (meters)
    let (i, lat_error) = opt(double).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    // 3. Expected 1-sigma error in longitude (meters)
    let (i, lon_error) = opt(double).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    // 4. Expected 1-sigma error in altitude (meters)
    let (i, alt_error) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    // 5. ID of most likely failed satellite (1 to 138)
    let (i, most_likely_failed_sat) = opt(number::<u8>).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    // 6. Probability of missed detection for most likely failed satellite
    let (i, missed_probability) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    // 7. Estimate of bias in meters on most likely failed satellite
    let (i, bias_estimate) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 8. Standard deviation of bias estimate
    let (i, bias_standard_deviation) = opt(float).parse(i)?;
    // 9. Checksum

    Ok((
        i,
        GbsData {
            time,
            lat_error,
            lon_error,
            alt_error,
            most_likely_failed_sat,
            missed_probability,
            bias_estimate,
            bias_standard_deviation,
        },
    ))
}

/// # Parse GBS message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_gbs_gps_satellite_fault_detection>
pub fn parse_gbs(sentence: NmeaSentence<'_>) -> Result<GbsData, Error<'_>> {
    if sentence.message_id != SentenceType::GBS {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::GBS,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_gbs(sentence.data)?.1)
    }
}

impl crate::generate::GenerateNmeaBody for GbsData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::GBS
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        use crate::sentences::gen_utils::*;

        // 1: time
        write_hms(f, &self.time)?;
        f.write_str(",")?;
        // 2: expected error in latitude
        write_field(f, &self.lat_error)?;
        // 3: expected error in longitude
        write_field(f, &self.lon_error)?;
        // 4: expected error in altitude
        write_field(f, &self.alt_error)?;
        // 5: most likely failed satellite
        write_field(f, &self.most_likely_failed_sat)?;
        // 6: probability of missed detection
        write_field(f, &self.missed_probability)?;
        // 7: estimate of bias
        write_field(f, &self.bias_estimate)?;
        // 8: standard deviation of bias
        write_opt(f, &self.bias_standard_deviation)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_generate_gbs_roundtrip() {
        let original = GbsData {
            time: Some(NaiveTime::from_hms_opt(18, 23, 45).unwrap()),
            lat_error: Some(1.2),
            lon_error: Some(3.4),
            alt_error: Some(5.6),
            most_likely_failed_sat: Some(12),
            missed_probability: Some(0.01),
            bias_estimate: Some(-2.3),
            bias_standard_deviation: Some(1.1),
        };
        let mut buf = heapless::String::<256>::new();
        crate::generate::generate_sentence("GP", &original, &mut buf).unwrap();
        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_gbs(s).unwrap();
        assert_eq!(parsed.time, original.time);
        assert_relative_eq!(parsed.lat_error.unwrap(), original.lat_error.unwrap());
        assert_relative_eq!(parsed.lon_error.unwrap(), original.lon_error.unwrap());
        assert_relative_eq!(parsed.alt_error.unwrap(), original.alt_error.unwrap());
        assert_eq!(parsed.most_likely_failed_sat, original.most_likely_failed_sat);
        assert_relative_eq!(parsed.missed_probability.unwrap(), original.missed_probability.unwrap());
        assert_relative_eq!(parsed.bias_estimate.unwrap(), original.bias_estimate.unwrap());
        assert_relative_eq!(
            parsed.bias_standard_deviation.unwrap(),
            original.bias_standard_deviation.unwrap()
        );
    }

    #[test]
    fn test_generate_gbs_empty() {
        let original = GbsData {
            time: None,
            lat_error: None,
            lon_error: None,
            alt_error: None,
            most_likely_failed_sat: None,
            missed_probability: None,
            bias_estimate: None,
            bias_standard_deviation: None,
        };
        let mut buf = heapless::String::<256>::new();
        crate::generate::generate_sentence("GP", &original, &mut buf).unwrap();
        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_gbs(s).unwrap();
        assert_eq!(parsed, original);
    }
}
