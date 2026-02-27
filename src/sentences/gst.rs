use crate::{Error, SentenceType, parse::NmeaSentence, sentences::utils::parse_hms};
use chrono::NaiveTime;
use nom::{
    IResult, Parser as _, character::complete::char, combinator::opt, number::complete::float,
};

/// GST - GPS Pseudorange Noise Statistics
/// ```text
///              1    2 3 4 5 6 7 8   9
///              |    | | | | | | |   |
/// $ --GST,hhmmss.ss,x,x,x,x,x,x,x*hh<CR><LF>
/// ```
/// Example: `$GPGST,182141.000,15.5,15.3,7.2,21.8,0.9,0.5,0.8*54`
///
/// 1. UTC time of associated GGA fix
/// 2. Total RMS standard deviation of ranges inputs to the navigation solution
/// 3. Standard deviation (meters) of semi-major axis of error ellipse
/// 4. Standard deviation (meters) of semi-minor axis of error ellipse
/// 5. Orientation of semi-major axis of error ellipse (true north degrees)
/// 6. Standard deviation (meters) of latitude error
/// 7. Standard deviation (meters) of longitude error
/// 8. Standard deviation (meters) of altitude error
/// 9. Checksum
///
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq)]
pub struct GstData {
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub time: Option<NaiveTime>,
    pub rms_sd: Option<f32>,
    pub ellipse_semi_major_sd: Option<f32>,
    pub ellipse_semi_minor_sd: Option<f32>,
    pub err_ellipse_orientation: Option<f32>,
    pub lat_sd: Option<f32>,
    pub long_sd: Option<f32>,
    pub alt_sd: Option<f32>,
}

fn do_parse_gst(i: &str) -> IResult<&str, GstData> {
    let (i, time) = opt(parse_hms).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    let (i, rms_sd) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    let (i, ellipse_semi_major_sd) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    let (i, ellipse_semi_minor_sd) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    let (i, err_ellipse_orientation) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    let (i, lat_sd) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    let (i, long_sd) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    let (i, alt_sd) = opt(float).parse(i)?;

    Ok((
        i,
        GstData {
            time,
            rms_sd,
            ellipse_semi_major_sd,
            ellipse_semi_minor_sd,
            err_ellipse_orientation,
            lat_sd,
            long_sd,
            alt_sd,
        },
    ))
}
pub fn parse_gst(sentence: NmeaSentence<'_>) -> Result<GstData, Error<'_>> {
    if sentence.message_id != SentenceType::GST {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::GST,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_gst(sentence.data)?.1)
    }
}

impl crate::generate::GenerateNmeaBody for GstData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::GST
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        use crate::sentences::gen_utils::*;

        // 1: time
        write_hms(f, &self.time)?;
        f.write_str(",")?;
        // 2: RMS standard deviation
        write_field(f, &self.rms_sd)?;
        // 3: semi-major axis std dev
        write_field(f, &self.ellipse_semi_major_sd)?;
        // 4: semi-minor axis std dev
        write_field(f, &self.ellipse_semi_minor_sd)?;
        // 5: error ellipse orientation
        write_field(f, &self.err_ellipse_orientation)?;
        // 6: latitude std dev
        write_field(f, &self.lat_sd)?;
        // 7: longitude std dev
        write_field(f, &self.long_sd)?;
        // 8: altitude std dev
        write_opt(f, &self.alt_sd)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, parse::parse_nmea_sentence};

    fn run_parse_gst(line: &str) -> Result<GstData, Error<'_>> {
        let s = parse_nmea_sentence(line).expect("GST sentence initial parse failed");
        assert_eq!(s.checksum, s.calc_checksum());
        parse_gst(s)
    }

    #[test]
    fn test_parse_gst() {
        assert_eq!(
            GstData {
                time: NaiveTime::from_hms_micro_opt(18, 21, 41, 00),
                rms_sd: Some(15.5),
                ellipse_semi_major_sd: Some(15.3),
                ellipse_semi_minor_sd: Some(7.2),
                err_ellipse_orientation: Some(21.8),
                lat_sd: Some(0.9),
                long_sd: Some(0.5),
                alt_sd: Some(0.8),
            },
            run_parse_gst("$GPGST,182141.000,15.5,15.3,7.2,21.8,0.9,0.5,0.8*54").unwrap()
        );
        assert_eq!(
            GstData {
                time: None,
                rms_sd: None,
                ellipse_semi_major_sd: None,
                ellipse_semi_minor_sd: None,
                err_ellipse_orientation: None,
                lat_sd: None,
                long_sd: None,
                alt_sd: None,
            },
            run_parse_gst("$GPGST,,,,,,,,*57").unwrap()
        );
    }

    #[test]
    fn test_generate_gst_roundtrip() {
        use approx::assert_relative_eq;

        let original = GstData {
            time: NaiveTime::from_hms_micro_opt(18, 21, 41, 0),
            rms_sd: Some(15.5),
            ellipse_semi_major_sd: Some(15.3),
            ellipse_semi_minor_sd: Some(7.2),
            err_ellipse_orientation: Some(21.8),
            lat_sd: Some(0.9),
            long_sd: Some(0.5),
            alt_sd: Some(0.8),
        };
        let mut buf = heapless::String::<256>::new();
        crate::generate::generate_sentence("GP", &original, &mut buf).unwrap();
        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_gst(s).unwrap();
        assert_eq!(parsed.time, original.time);
        assert_relative_eq!(parsed.rms_sd.unwrap(), original.rms_sd.unwrap());
        assert_relative_eq!(parsed.ellipse_semi_major_sd.unwrap(), original.ellipse_semi_major_sd.unwrap());
        assert_relative_eq!(parsed.ellipse_semi_minor_sd.unwrap(), original.ellipse_semi_minor_sd.unwrap());
        assert_relative_eq!(parsed.err_ellipse_orientation.unwrap(), original.err_ellipse_orientation.unwrap());
        assert_relative_eq!(parsed.lat_sd.unwrap(), original.lat_sd.unwrap());
        assert_relative_eq!(parsed.long_sd.unwrap(), original.long_sd.unwrap());
        assert_relative_eq!(parsed.alt_sd.unwrap(), original.alt_sd.unwrap());
    }

    #[test]
    fn test_generate_gst_empty() {
        let original = GstData {
            time: None,
            rms_sd: None,
            ellipse_semi_major_sd: None,
            ellipse_semi_minor_sd: None,
            err_ellipse_orientation: None,
            lat_sd: None,
            long_sd: None,
            alt_sd: None,
        };
        let mut buf = heapless::String::<256>::new();
        crate::generate::generate_sentence("GP", &original, &mut buf).unwrap();
        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_gst(s).unwrap();
        assert_eq!(parsed, original);
    }
}
