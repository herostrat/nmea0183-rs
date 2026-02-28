use arrayvec::ArrayString;
use nom::{
    IResult, Parser as _,
    bytes::complete::take_while,
    character::complete::{anychar, char},
    combinator::opt,
};

use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// DSC - Digital Selective Calling Information
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_cddsc_digital_selective_calling_dsc>
///
/// ```text
///        1  2          3  4  5  6          7    8          9  10 11
///        |  |          |  |  |  |          |    |          |  |  |
/// $CDDSC,xx,xxxxxxxxxx,xx,xx,xx,xxxxxxxxxx,xxxx,xxxxxxxxxx,xx,x,x*hh<CR><LF>
/// ```
///
/// 1. Format Specifier (2-digit code, leading '1' omitted from ITU symbol)
/// 2. MMSI of sender (10 digits, last digit is check digit)
/// 3. Category (2-digit code, may be empty)
/// 4. First Telecommand / Nature of Distress
/// 5. Second Telecommand / Type of Communication
/// 6. Position data (QDDMMDDDDMM) or frequency/MMSI (context-dependent)
/// 7. Time UTC (HHMM)
/// 8. Receiver MMSI or additional data (10 digits, may be empty)
/// 9. Additional info (may be empty)
/// 10. Acknowledgment type (R=received, B=able to comply, S=end of call)
/// 11. Expansion indicator (E=DSE follows, S=no expansion)
///
/// Format Specifier codes (ITU-R M.493, leading '1' omitted):
/// * 02 = Area Call (ITU 102)
/// * 12 = Distress (ITU 112)
/// * 14 = All Ships (ITU 114)
/// * 16 = Group Call (ITU 116)
/// * 20 = Individual / Position Report (ITU 120)
/// * 23 = Individual (ITU 123)
///
/// Position encoding (field 6): `QDDMMDDDDMM`
/// * Q = Quadrant (0=NE, 1=NW, 2=SE, 3=SW)
/// * DDMM = Latitude degrees and minutes
/// * DDDMM = Longitude degrees and minutes
///
/// Examples:
/// * `$CDDSC,12,3380400790,12,06,00,1423108312,2019,,,S,E*6A`
/// * `$CDDSC,20,5031105200,08,21,26,2380814428,1800,,,B,E*77`
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct DscData {
    /// Format specifier code (2-digit, leading '1' omitted from ITU symbol)
    pub format_specifier: u8,
    /// MMSI of sender (full 10-digit string including check digit)
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub mmsi: ArrayString<10>,
    /// Category code (00=Routine, 08=Safety, 10=Urgency, 12=Distress)
    pub category: Option<u8>,
    /// First telecommand / Nature of distress code
    pub first_telecommand: Option<u8>,
    /// Second telecommand / Type of communication code
    pub second_telecommand: Option<u8>,
    /// Raw position/data field (10-digit string, may encode position or MMSI)
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub position_or_data: ArrayString<10>,
    /// Time UTC as HHMM string
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub time_utc: ArrayString<4>,
    /// Second MMSI field (receiver or vessel in distress)
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub mmsi_2: ArrayString<10>,
    /// Additional info field
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub additional_info: ArrayString<10>,
    /// Acknowledgment type (R, B, S, etc.)
    pub ack_type: Option<char>,
    /// Expansion indicator (E=DSE follows, S=no expansion)
    pub expansion_indicator: Option<char>,
}

impl DscData {
    /// Decode position from the position_or_data field.
    ///
    /// Position encoding: `QDDMMDDDDMM`
    /// * Q = Quadrant (0=NE, 1=NW, 2=SE, 3=SW)
    /// * DDMM = Latitude degrees (DD) and minutes (MM)
    /// * DDDMM = Longitude degrees (DDD) and minutes (MM)
    ///
    /// Returns `Some((latitude, longitude))` in decimal degrees, or `None`
    /// if the field doesn't contain a valid position.
    pub fn decode_position(&self) -> Option<(f64, f64)> {
        let s = self.position_or_data.as_str();
        if s.len() != 10 {
            return None;
        }

        let quadrant = s.as_bytes()[0].wrapping_sub(b'0');
        if quadrant > 3 {
            return None;
        }

        let lat_deg: f64 = s[1..3].parse().ok()?;
        let lat_min: f64 = s[3..5].parse().ok()?;
        let lon_deg: f64 = s[5..8].parse().ok()?;
        let lon_min: f64 = s[8..10].parse().ok()?;

        if lat_deg > 90.0 || lat_min >= 60.0 || lon_deg > 180.0 || lon_min >= 60.0 {
            return None;
        }

        let mut lat = lat_deg + lat_min / 60.0;
        let mut lon = lon_deg + lon_min / 60.0;

        // Quadrant: 0=NE, 1=NW, 2=SE, 3=SW
        if quadrant >= 2 {
            lat = -lat;
        }
        if quadrant == 1 || quadrant == 3 {
            lon = -lon;
        }

        Some((lat, lon))
    }

    /// Extract the 9-digit MMSI (without check digit) from the sender MMSI field.
    pub fn sender_mmsi(&self) -> Option<u32> {
        let s = self.mmsi.as_str();
        if s.len() >= 9 {
            s[..9].parse().ok()
        } else {
            None
        }
    }
}

impl From<DscData> for ParseResult {
    fn from(value: DscData) -> Self {
        ParseResult::DSC(value)
    }
}

/// Parse optional 2-digit numeric field
fn parse_two_digit(i: &str) -> IResult<&str, Option<u8>> {
    let field_end = i.find(',').unwrap_or(i.len());
    let field = &i[..field_end];
    let i = &i[field_end..];

    if field.is_empty() {
        Ok((i, None))
    } else {
        match field.parse::<u8>() {
            Ok(v) => Ok((i, Some(v))),
            Err(_) => Err(nom::Err::Failure(nom::error::Error::new(
                i,
                nom::error::ErrorKind::Digit,
            ))),
        }
    }
}

/// Parse a string field up to the next comma (or end)
fn parse_field_str<const N: usize>(i: &str) -> IResult<&str, ArrayString<N>> {
    let (i, s) = take_while(|c: char| c != ',').parse(i)?;
    let take = s.len().min(N);
    let mut result = ArrayString::<N>::new();
    result.push_str(&s[..take]);
    Ok((i, result))
}

fn do_parse_dsc(i: &str) -> IResult<&str, DscData> {
    // Field 1: Format specifier (2-digit)
    let (i, format_specifier) = parse_two_digit(i)?;
    let format_specifier = format_specifier.ok_or(nom::Err::Failure(nom::error::Error::new(
        i,
        nom::error::ErrorKind::Digit,
    )))?;
    let (i, _) = char(',').parse(i)?;

    // Field 2: MMSI (10 digits)
    let (i, mmsi) = parse_field_str::<10>(i)?;
    let (i, _) = char(',').parse(i)?;

    // Field 3: Category (2-digit, optional)
    let (i, category) = parse_two_digit(i)?;
    let (i, _) = char(',').parse(i)?;

    // Field 4: First telecommand (2-digit, optional)
    let (i, first_telecommand) = parse_two_digit(i)?;
    let (i, _) = char(',').parse(i)?;

    // Field 5: Second telecommand (2-digit, optional)
    let (i, second_telecommand) = parse_two_digit(i)?;
    let (i, _) = char(',').parse(i)?;

    // Field 6: Position or data (up to 10 chars)
    let (i, position_or_data) = parse_field_str::<10>(i)?;
    let (i, _) = char(',').parse(i)?;

    // Field 7: Time UTC (4 chars)
    let (i, time_utc) = parse_field_str::<4>(i)?;
    let (i, _) = char(',').parse(i)?;

    // Field 8: Second MMSI (up to 10 chars)
    let (i, mmsi_2) = parse_field_str::<10>(i)?;
    let (i, _) = char(',').parse(i)?;

    // Field 9: Additional info (up to 10 chars)
    let (i, additional_info) = parse_field_str::<10>(i)?;
    let (i, _) = char(',').parse(i)?;

    // Field 10: Ack type (single char, optional)
    let (i, ack_type) = opt(anychar).parse(i)?;
    let ack_type = ack_type.filter(|&c| c != ',');
    let i = if ack_type.is_some() {
        let (i, _) = char(',').parse(i)?;
        i
    } else if let Some(stripped) = i.strip_prefix(',') {
        stripped
    } else {
        i
    };

    // Field 11: Expansion indicator (single char, optional)
    let (i, expansion_indicator) = opt(anychar).parse(i)?;
    let expansion_indicator = expansion_indicator.filter(|&c| c != ',');

    Ok((
        i,
        DscData {
            format_specifier,
            mmsi,
            category,
            first_telecommand,
            second_telecommand,
            position_or_data,
            time_utc,
            mmsi_2,
            additional_info,
            ack_type,
            expansion_indicator,
        },
    ))
}

/// # Parse DSC message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_cddsc_digital_selective_calling_dsc>
pub fn parse_dsc(sentence: NmeaSentence<'_>) -> Result<DscData, Error<'_>> {
    if sentence.message_id != SentenceType::DSC {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::DSC,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_dsc(sentence.data)?.1)
    }
}

impl crate::generate::GenerateNmeaBody for DscData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::DSC
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        // Field 1: Format specifier
        write!(f, "{:02}", self.format_specifier)?;
        f.write_char(',')?;
        // Field 2: MMSI
        f.write_str(&self.mmsi)?;
        f.write_char(',')?;
        // Field 3: Category
        if let Some(c) = self.category {
            write!(f, "{:02}", c)?;
        }
        f.write_char(',')?;
        // Field 4: First telecommand
        if let Some(t) = self.first_telecommand {
            write!(f, "{:02}", t)?;
        }
        f.write_char(',')?;
        // Field 5: Second telecommand
        if let Some(t) = self.second_telecommand {
            write!(f, "{:02}", t)?;
        }
        f.write_char(',')?;
        // Field 6: Position/data
        f.write_str(&self.position_or_data)?;
        f.write_char(',')?;
        // Field 7: Time UTC
        f.write_str(&self.time_utc)?;
        f.write_char(',')?;
        // Field 8: MMSI 2
        f.write_str(&self.mmsi_2)?;
        f.write_char(',')?;
        // Field 9: Additional info
        f.write_str(&self.additional_info)?;
        f.write_char(',')?;
        // Field 10: Ack type
        if let Some(a) = self.ack_type {
            f.write_char(a)?;
        }
        f.write_char(',')?;
        // Field 11: Expansion indicator
        if let Some(e) = self.expansion_indicator {
            f.write_char(e)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_dsc_distress() {
        let s =
            parse_nmea_sentence("$CDDSC,12,3380400790,12,06,00,1423108312,2019,,,S,E*6A").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_dsc(s).unwrap();

        assert_eq!(data.format_specifier, 12);
        assert_eq!(data.mmsi.as_str(), "3380400790");
        assert_eq!(data.category, Some(12));
        assert_eq!(data.first_telecommand, Some(6));
        assert_eq!(data.second_telecommand, Some(0));
        assert_eq!(data.position_or_data.as_str(), "1423108312");
        assert_eq!(data.time_utc.as_str(), "2019");
        assert_eq!(data.ack_type, Some('S'));
        assert_eq!(data.expansion_indicator, Some('E'));

        // Check MMSI extraction
        assert_eq!(data.sender_mmsi(), Some(338040079));

        // Check position decoding: 42°31'N, 083°12'W
        let (lat, lon) = data.decode_position().unwrap();
        assert_relative_eq!(lat, 42.0 + 31.0 / 60.0, epsilon = 0.001);
        assert_relative_eq!(lon, -(83.0 + 12.0 / 60.0), epsilon = 0.001);
    }

    #[test]
    fn test_parse_dsc_distress_empty_category() {
        let s =
            parse_nmea_sentence("$CDDSC,12,5031105200,,05,00,2380814428,1800,,,R,E*6C").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_dsc(s).unwrap();

        assert_eq!(data.format_specifier, 12);
        assert_eq!(data.mmsi.as_str(), "5031105200");
        assert_eq!(data.category, None);
        assert_eq!(data.first_telecommand, Some(5));
        assert_eq!(data.second_telecommand, Some(0));
        assert_eq!(data.ack_type, Some('R'));
        assert_eq!(data.expansion_indicator, Some('E'));

        // Position: 38°08'S, 144°28'E (quadrant 2 = SE)
        let (lat, lon) = data.decode_position().unwrap();
        assert_relative_eq!(lat, -(38.0 + 8.0 / 60.0), epsilon = 0.001);
        assert_relative_eq!(lon, 144.0 + 28.0 / 60.0, epsilon = 0.001);
    }

    #[test]
    fn test_parse_dsc_position_report() {
        let s =
            parse_nmea_sentence("$CDDSC,20,5031105200,08,21,26,2380814428,1800,,,B,E*77").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_dsc(s).unwrap();

        assert_eq!(data.format_specifier, 20);
        assert_eq!(data.category, Some(8));
        assert_eq!(data.first_telecommand, Some(21));
        assert_eq!(data.second_telecommand, Some(26));
        assert_eq!(data.ack_type, Some('B'));
    }

    #[test]
    fn test_parse_dsc_distress_relay() {
        let s = parse_nmea_sentence(
            "$CDDSC,16,2350763930,12,12,00,2380814428,1800,5031105200,05,S,E*65",
        )
        .unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_dsc(s).unwrap();

        assert_eq!(data.format_specifier, 16);
        assert_eq!(data.mmsi.as_str(), "2350763930");
        assert_eq!(data.category, Some(12));
        assert_eq!(data.first_telecommand, Some(12));
        assert_eq!(data.second_telecommand, Some(0));
        assert_eq!(data.mmsi_2.as_str(), "5031105200");
        assert_eq!(data.additional_info.as_str(), "05");
        assert_eq!(data.ack_type, Some('S'));
    }

    #[test]
    fn test_decode_position_quadrants() {
        // Quadrant 0 = NE
        let mut data = make_dsc("0523012345");
        let (lat, lon) = data.decode_position().unwrap();
        assert_relative_eq!(lat, 52.0 + 30.0 / 60.0, epsilon = 0.001);
        assert_relative_eq!(lon, 123.0 + 45.0 / 60.0, epsilon = 0.001);

        // Quadrant 1 = NW
        data.position_or_data = arr_str("1523012345");
        let (lat, lon) = data.decode_position().unwrap();
        assert!(lat > 0.0);
        assert!(lon < 0.0);

        // Quadrant 2 = SE
        data.position_or_data = arr_str("2523012345");
        let (lat, lon) = data.decode_position().unwrap();
        assert!(lat < 0.0);
        assert!(lon > 0.0);

        // Quadrant 3 = SW
        data.position_or_data = arr_str("3523012345");
        let (lat, lon) = data.decode_position().unwrap();
        assert!(lat < 0.0);
        assert!(lon < 0.0);
    }

    #[test]
    fn test_decode_position_invalid() {
        // Invalid quadrant
        let mut data = make_dsc("5523012345");
        assert!(data.decode_position().is_none());

        // Too short
        data.position_or_data = arr_str("12345");
        assert!(data.decode_position().is_none());
    }

    #[test]
    fn test_generate_dsc_roundtrip() {
        let original = DscData {
            format_specifier: 12,
            mmsi: arr_str("3380400790"),
            category: Some(12),
            first_telecommand: Some(6),
            second_telecommand: Some(0),
            position_or_data: arr_str("1423108312"),
            time_utc: ArrayString::from("2019").unwrap(),
            mmsi_2: ArrayString::new(),
            additional_info: ArrayString::new(),
            ack_type: Some('S'),
            expansion_indicator: Some('E'),
        };
        let mut buf = heapless::String::<128>::new();
        crate::generate::generate_sentence("CD", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_dsc(s).unwrap();
        assert_eq!(parsed.format_specifier, 12);
        assert_eq!(parsed.mmsi.as_str(), "3380400790");
        assert_eq!(parsed.category, Some(12));
        assert_eq!(parsed.first_telecommand, Some(6));
        assert_eq!(parsed.ack_type, Some('S'));
        assert_eq!(parsed.expansion_indicator, Some('E'));
    }

    // Helper functions for tests
    fn arr_str(s: &str) -> ArrayString<10> {
        ArrayString::from(s).unwrap()
    }

    fn make_dsc(position: &str) -> DscData {
        DscData {
            format_specifier: 12,
            mmsi: arr_str("0000000000"),
            category: None,
            first_telecommand: None,
            second_telecommand: None,
            position_or_data: arr_str(position),
            time_utc: ArrayString::new(),
            mmsi_2: ArrayString::new(),
            additional_info: ArrayString::new(),
            ack_type: None,
            expansion_indicator: None,
        }
    }
}
