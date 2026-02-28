use arrayvec::ArrayString;
use nom::{
    IResult, Parser as _, bytes::complete::is_not, character::complete::char, combinator::opt,
};

use crate::sentences::utils::array_string;
use crate::{Error, NmeaSentence, ParseResult, SentenceType};

/// Maximum AIS payload length (6-bit encoded, up to 82 chars)
const AIS_PAYLOAD_MAX_LEN: usize = 82;

/// VDM/VDO - AIS VHF Data-Link Message
///
/// VDM = received AIS messages from other vessels
/// VDO = own vessel AIS data
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_vdm_vdo_ais_vhf_data_link_message>
///
/// ```text
///        1 2 3 4 5     6 7
///        | | | | |     | |
/// !--VDM,x,x,x,a,s--s,x*hh<CR><LF>
/// ```
/// 1. Number of fragments (1-9)
/// 2. Fragment number (1-9)
/// 3. Sequential message ID (for multi-fragment, 0-9)
/// 4. AIS channel (A or B)
/// 5. Encapsulated data (6-bit ASCII armored)
/// 6. Number of fill bits (0-5)
/// 7. Checksum
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct VdmData {
    /// Total number of fragments
    pub fragment_count: u8,
    /// This fragment number (1-based)
    pub fragment_number: u8,
    /// Sequential message ID (for multi-fragment messages)
    pub message_id: Option<u8>,
    /// AIS channel: 'A' or 'B'
    pub channel: Option<char>,
    /// Raw encapsulated AIS payload (6-bit encoded)
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub payload: ArrayString<AIS_PAYLOAD_MAX_LEN>,
    /// Number of fill bits (0-5)
    pub fill_bits: u8,
    /// Whether this is own-vessel data (VDO) or received data (VDM)
    pub own_vessel: bool,
}

impl From<VdmData> for ParseResult {
    fn from(value: VdmData) -> Self {
        if value.own_vessel {
            ParseResult::VDO(value)
        } else {
            ParseResult::VDM(value)
        }
    }
}

/// # Parse VDM message (received AIS)
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_vdm_vdo_ais_vhf_data_link_message>
pub fn parse_vdm(sentence: NmeaSentence<'_>) -> Result<VdmData, Error<'_>> {
    if sentence.message_id != SentenceType::VDM && sentence.message_id != SentenceType::VDO {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::VDM,
            found: sentence.message_id,
        })
    } else {
        let own_vessel = sentence.message_id == SentenceType::VDO;
        let mut data = do_parse_vdm(sentence.data)?.1;
        data.own_vessel = own_vessel;
        Ok(data)
    }
}

fn do_parse_vdm(i: &str) -> IResult<&str, VdmData> {
    // 1. Fragment count
    let (i, frag_count) = nom::character::complete::u8(i)?;
    let (i, _) = char(',').parse(i)?;
    // 2. Fragment number
    let (i, frag_num) = nom::character::complete::u8(i)?;
    let (i, _) = char(',').parse(i)?;
    // 3. Sequential message ID (optional)
    let (i, msg_id) = opt(nom::character::complete::u8).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 4. Channel
    let (i, channel) = opt(nom::character::complete::one_of("AB12")).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    // 5. Payload
    let (i, payload_str) = is_not(",*").parse(i)?;
    let payload = array_string::<AIS_PAYLOAD_MAX_LEN>(payload_str)
        .map_err(|_| nom::Err::Failure(nom::error::Error::new(i, nom::error::ErrorKind::Fail)))?;
    let (i, _) = char(',').parse(i)?;
    // 6. Fill bits
    let (i, fill_bits) = nom::character::complete::u8(i)?;

    Ok((
        i,
        VdmData {
            fragment_count: frag_count,
            fragment_number: frag_num,
            message_id: msg_id,
            channel,
            payload,
            fill_bits,
            own_vessel: false, // set by caller
        },
    ))
}

impl crate::generate::GenerateNmeaBody for VdmData {
    fn sentence_type(&self) -> SentenceType {
        if self.own_vessel {
            SentenceType::VDO
        } else {
            SentenceType::VDM
        }
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        // 1. Fragment count
        write!(f, "{}", self.fragment_count)?;
        f.write_char(',')?;
        // 2. Fragment number
        write!(f, "{}", self.fragment_number)?;
        f.write_char(',')?;
        // 3. Sequential message ID (optional)
        if let Some(id) = self.message_id {
            write!(f, "{}", id)?;
        }
        f.write_char(',')?;
        // 4. Channel
        if let Some(ch) = self.channel {
            f.write_char(ch)?;
        }
        f.write_char(',')?;
        // 5. Payload
        f.write_str(&self.payload)?;
        f.write_char(',')?;
        // 6. Fill bits
        write!(f, "{}", self.fill_bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_vdm_single() {
        let s = parse_nmea_sentence("!AIVDM,1,1,,A,13aEOK?P00PD2wVMdLDRhgvL289?,0*26").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vdm(s).unwrap();
        assert_eq!(data.fragment_count, 1);
        assert_eq!(data.fragment_number, 1);
        assert_eq!(data.message_id, None);
        assert_eq!(data.channel, Some('A'));
        assert_eq!(&*data.payload, "13aEOK?P00PD2wVMdLDRhgvL289?");
        assert_eq!(data.fill_bits, 0);
        assert!(!data.own_vessel);
    }

    #[test]
    fn test_parse_vdm_multi_fragment() {
        let s1 = parse_nmea_sentence(
            "!AIVDM,2,1,0,A,53brRt4000010SG700iE@LE8@Tp4000000000153P615t0Ht0SCkjH4jC1C,0*25",
        )
        .unwrap();
        assert_eq!(s1.checksum, s1.calc_checksum());
        let data1 = parse_vdm(s1).unwrap();
        assert_eq!(data1.fragment_count, 2);
        assert_eq!(data1.fragment_number, 1);
        assert_eq!(data1.message_id, Some(0));
        assert_eq!(data1.channel, Some('A'));

        let s2 = parse_nmea_sentence("!AIVDM,2,2,0,A,`0000000001,2*75").unwrap();
        assert_eq!(s2.checksum, s2.calc_checksum());
        let data2 = parse_vdm(s2).unwrap();
        assert_eq!(data2.fragment_count, 2);
        assert_eq!(data2.fragment_number, 2);
        assert_eq!(data2.message_id, Some(0));
        assert_eq!(data2.fill_bits, 2);
    }

    #[test]
    fn test_parse_vdm_channel_b() {
        let s = parse_nmea_sentence("!AIVDM,1,1,,B,13aGra0P00PHid>NK9<2FOvHR624,0*3E").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vdm(s).unwrap();
        assert_eq!(data.channel, Some('B'));
    }

    #[test]
    fn test_parse_bsvdm() {
        // Non-AI talker (BS = base station)
        let s = parse_nmea_sentence("!BSVDM,1,1,,A,B6CdCm0t3`tba35f@V9faHi7kP06,0*41").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_vdm(s).unwrap();
        assert_eq!(data.fragment_count, 1);
    }

    #[test]
    fn test_generate_vdm_roundtrip() {
        let original = VdmData {
            fragment_count: 1,
            fragment_number: 1,
            message_id: None,
            channel: Some('A'),
            payload: ArrayString::from("13aEOK?P00PD2wVMdLDRhgvL289?").unwrap(),
            fill_bits: 0,
            own_vessel: false,
        };
        let mut buf = heapless::String::<256>::new();
        crate::generate::generate_sentence("AI", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_vdm(s).unwrap();
        assert_eq!(parsed.fragment_count, 1);
        assert_eq!(parsed.fragment_number, 1);
        assert_eq!(parsed.message_id, None);
        assert_eq!(parsed.channel, Some('A'));
        assert_eq!(&*parsed.payload, "13aEOK?P00PD2wVMdLDRhgvL289?");
        assert_eq!(parsed.fill_bits, 0);
    }
}
