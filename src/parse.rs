use core::str;

use nom::{
    IResult, Parser as _,
    bytes::complete::{take, take_until},
    character::complete::{char, one_of},
    combinator::map_res,
    sequence::preceded,
};

use cfg_if::cfg_if;

use crate::{Error, SentenceType, sentences::*};

/// The maximum message length parsable by the crate.
///
/// From `gpsd`:
///
/// > We've had reports that on the Garmin GPS-10 the device sometimes
/// > (1:1000 or so) sends garbage packets that have a valid checksum
/// > but are like 2 successive NMEA packets merged together in one
/// > with some fields lost. Usually these are much longer than the
/// > legal limit for NMEA, so we can cope by just tossing out overlong
/// > packets.  This may be a generic bug of all Garmin chipsets.
/// > NMEA 3.01, Section 5.3 says the max sentence length shall be
/// > 82 chars, including the leading $ and terminating \r\n.
///
/// > Some receivers (TN-200, GSW 2.3.2) emit oversized sentences.
/// > The Trimble BX-960 receiver emits a 91-character GGA message.
/// > The current hog champion is the Skytraq S2525F8 which emits
/// > a 100-character PSTI message.
pub const SENTENCE_MAX_LEN: usize = 102;

/// Maximum length of a single waypoint id data in sentence
pub const TEXT_PARAMETER_MAX_LEN: usize = 64;

/// A known and parsable Nmea sentence type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NmeaSentence<'a> {
    pub talker_id: &'a str,
    pub message_id: SentenceType,
    pub data: &'a str,
    pub checksum: u8,
}

impl NmeaSentence<'_> {
    pub fn calc_checksum(&self) -> u8 {
        checksum(
            self.talker_id
                .as_bytes()
                .iter()
                .chain(self.message_id.as_str().as_bytes())
                .chain(b",")
                .chain(self.data.as_bytes()),
        )
    }
}

pub(crate) fn checksum<'a, I: Iterator<Item = &'a u8>>(bytes: I) -> u8 {
    bytes.fold(0, |c, x| c ^ *x)
}

fn parse_hex(data: &str) -> Result<u8, &'static str> {
    u8::from_str_radix(data, 16).map_err(|_| "Failed to parse checksum as hex number")
}

fn parse_checksum(i: &str) -> IResult<&str, u8> {
    map_res(preceded(char('*'), take(2usize)), parse_hex).parse(i)
}

fn parse_sentence_type(i: &str) -> IResult<&str, SentenceType> {
    map_res(take(3usize), |sentence_type: &str| {
        SentenceType::try_from(sentence_type).map_err(|_| "Unknown sentence type")
    })
    .parse(i)
}

fn do_parse_nmea_sentence(i: &str) -> IResult<&str, NmeaSentence<'_>> {
    let (i, talker_id) = preceded(one_of("$!"), take(2usize)).parse(i)?;
    let (i, message_id) = parse_sentence_type(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, data) = take_until("*").parse(i)?;
    let (i, checksum) = parse_checksum(i)?;

    Ok((
        i,
        NmeaSentence {
            talker_id,
            message_id,
            data,
            checksum,
        },
    ))
}

pub fn parse_nmea_sentence(sentence: &str) -> core::result::Result<NmeaSentence<'_>, Error<'_>> {
    if sentence.len() > SENTENCE_MAX_LEN {
        Err(Error::SentenceLength(sentence.len()))
    } else {
        Ok(do_parse_nmea_sentence(sentence)?.1)
    }
}

/// The result of parsing a single NMEA message.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq)]
pub enum ParseResult {
    AAM(AamData),
    ALM(AlmData),
    APA(ApaData),
    APB(ApbData),
    BOD(BodData),
    BWC(BwcData),
    BWW(BwwData),
    DBK(DbkData),
    DBS(DbsData),
    DBT(DbtData),
    DPT(DptData),
    GBS(GbsData),
    GGA(GgaData),
    GLL(GllData),
    GNS(GnsData),
    GSA(GsaData),
    GST(GstData),
    GSV(GsvData),
    HDG(HdgData),
    HDM(HdmData),
    HDT(HdtData),
    HSC(HscData),
    MDA(MdaData),
    MTW(MtwData),
    MWD(MwdData),
    MWV(MwvData),
    RMB(RmbData),
    RMC(RmcData),
    ROT(RotData),
    RPM(RpmData),
    RSA(RsaData),
    TTM(TtmData),
    TXT(TxtData),
    VDM(VdmData),
    VDO(VdmData),
    VDR(VdrData),
    VHW(VhwData),
    VLW(VlwData),
    VPW(VpwData),
    VTG(VtgData),
    VWR(VwrData),
    VWT(VwtData),
    XDR(XdrData),
    XTE(XteData),
    WNC(WncData),
    ZDA(ZdaData),
    ZFO(ZfoData),
    ZTG(ZtgData),
    PGRMZ(PgrmzData),
    /// A message that is not supported by the crate and cannot be parsed.
    Unsupported(SentenceType),
}

impl From<&ParseResult> for SentenceType {
    fn from(parse_result: &ParseResult) -> Self {
        match parse_result {
            ParseResult::AAM(_) => SentenceType::AAM,
            ParseResult::ALM(_) => SentenceType::ALM,
            ParseResult::APA(_) => SentenceType::APA,
            ParseResult::APB(_) => SentenceType::APB,
            ParseResult::BOD(_) => SentenceType::BOD,
            ParseResult::BWC(_) => SentenceType::BWC,
            ParseResult::BWW(_) => SentenceType::BWW,
            ParseResult::DBK(_) => SentenceType::DBK,
            ParseResult::DBS(_) => SentenceType::DBS,
            ParseResult::DBT(_) => SentenceType::DBT,
            ParseResult::GBS(_) => SentenceType::GBS,
            ParseResult::GGA(_) => SentenceType::GGA,
            ParseResult::GLL(_) => SentenceType::GLL,
            ParseResult::GNS(_) => SentenceType::GNS,
            ParseResult::GSA(_) => SentenceType::GSA,
            ParseResult::GST(_) => SentenceType::GST,
            ParseResult::GSV(_) => SentenceType::GSV,
            ParseResult::HDG(_) => SentenceType::HDG,
            ParseResult::HDM(_) => SentenceType::HDM,
            ParseResult::HDT(_) => SentenceType::HDT,
            ParseResult::HSC(_) => SentenceType::HSC,
            ParseResult::MDA(_) => SentenceType::MDA,
            ParseResult::MTW(_) => SentenceType::MTW,
            ParseResult::MWD(_) => SentenceType::MWD,
            ParseResult::MWV(_) => SentenceType::MWV,
            ParseResult::RMB(_) => SentenceType::RMB,
            ParseResult::RMC(_) => SentenceType::RMC,
            ParseResult::ROT(_) => SentenceType::ROT,
            ParseResult::RPM(_) => SentenceType::RPM,
            ParseResult::RSA(_) => SentenceType::RSA,
            ParseResult::TTM(_) => SentenceType::TTM,
            ParseResult::TXT(_) => SentenceType::TXT,
            ParseResult::VDM(_) => SentenceType::VDM,
            ParseResult::VDO(_) => SentenceType::VDO,
            ParseResult::VDR(_) => SentenceType::VDR,
            ParseResult::VHW(_) => SentenceType::VHW,
            ParseResult::VLW(_) => SentenceType::VLW,
            ParseResult::VPW(_) => SentenceType::VPW,
            ParseResult::VTG(_) => SentenceType::VTG,
            ParseResult::VWR(_) => SentenceType::VWR,
            ParseResult::VWT(_) => SentenceType::VWT,
            ParseResult::XDR(_) => SentenceType::XDR,
            ParseResult::XTE(_) => SentenceType::XTE,
            ParseResult::WNC(_) => SentenceType::WNC,
            ParseResult::ZFO(_) => SentenceType::ZFO,
            ParseResult::ZTG(_) => SentenceType::ZTG,
            ParseResult::PGRMZ(_) => SentenceType::RMZ,
            ParseResult::ZDA(_) => SentenceType::ZDA,
            ParseResult::DPT(_) => SentenceType::DPT,
            ParseResult::Unsupported(sentence_type) => *sentence_type,
        }
    }
}

/// Parse a NMEA 0183 sentence from bytes and extract data from it.
///
/// # Errors
///
/// Apart from errors returned by the message parsing itself, it will return
/// [`Error::Utf8Decoding`] when the bytes are not a valid UTF-8 string.
pub fn parse_bytes(sentence_input: &[u8]) -> Result<ParseResult, Error<'_>> {
    let string = core::str::from_utf8(sentence_input).map_err(|_err| Error::Utf8Decoding)?;

    parse_str(string)
}

/// Parse a NMEA 0183 sentence from a string slice and extract data from it.
///
/// Should not contain `\r\n` ending.
///
/// # Errors
///
/// - [`Error::ASCII`] when string contains non-ASCII characters.
pub fn parse_str(sentence_input: &str) -> Result<ParseResult, Error<'_>> {
    if !sentence_input.is_ascii() {
        return Err(Error::ASCII);
    }

    let nmea_sentence = parse_nmea_sentence(sentence_input)?;
    let calculated_checksum = nmea_sentence.calc_checksum();

    if nmea_sentence.checksum == calculated_checksum {
        // Ordered alphabetically
        match nmea_sentence.message_id {
            SentenceType::AAM => {
                cfg_if! {
                    if #[cfg(feature = "AAM")] {
                        parse_aam(nmea_sentence).map(ParseResult::AAM)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::ALM => {
                cfg_if! {
                    if #[cfg(feature = "ALM")] {
                        parse_alm(nmea_sentence).map(ParseResult::ALM)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::APA => {
                cfg_if! {
                    if #[cfg(feature = "APA")] {
                        parse_apa(nmea_sentence).map(ParseResult::APA)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::APB => {
                cfg_if! {
                    if #[cfg(feature = "APB")] {
                        parse_apb(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::BOD => {
                cfg_if! {
                    if #[cfg(feature = "BOD")] {
                        parse_bod(nmea_sentence).map(ParseResult::BOD)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::BWC => {
                cfg_if! {
                    if #[cfg(feature = "BWC")] {
                        parse_bwc(nmea_sentence).map(ParseResult::BWC)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::BWW => {
                cfg_if! {
                    if #[cfg(feature = "BWW")] {
                        parse_bww(nmea_sentence).map(ParseResult::BWW)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::DBK => {
                cfg_if! {
                    if #[cfg(feature = "DBK")] {
                        parse_dbk(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::DBS => {
                cfg_if! {
                    if #[cfg(feature = "DBS")] {
                        parse_dbs(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::DBT => {
                cfg_if! {
                    if #[cfg(feature = "DBT")] {
                        parse_dbt(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::GBS => {
                cfg_if! {
                    if #[cfg(feature = "GBS")] {
                        parse_gbs(nmea_sentence).map(ParseResult::GBS)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::GGA => {
                cfg_if! {
                    if #[cfg(feature = "GGA")] {
                        parse_gga(nmea_sentence).map(ParseResult::GGA)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::GLL => {
                cfg_if! {
                    if #[cfg(feature = "GLL")] {
                        parse_gll(nmea_sentence).map(ParseResult::GLL)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::GNS => {
                cfg_if! {
                    if #[cfg(feature = "GNS")] {
                        parse_gns(nmea_sentence).map(ParseResult::GNS)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::GSA => {
                cfg_if! {
                    if #[cfg(feature = "GSA")] {
                        parse_gsa(nmea_sentence).map(ParseResult::GSA)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::GST => {
                cfg_if! {
                    if #[cfg(feature = "GST")] {
                        parse_gst(nmea_sentence).map(ParseResult::GST)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::GSV => {
                cfg_if! {
                    if #[cfg(feature = "GSV")] {
                        parse_gsv(nmea_sentence).map(ParseResult::GSV)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::HDG => {
                cfg_if! {
                    if #[cfg(feature = "HDG")] {
                        parse_hdg(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::HDM => {
                cfg_if! {
                    if #[cfg(feature = "HDM")] {
                        parse_hdm(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::HDT => {
                cfg_if! {
                    if #[cfg(feature = "HDT")] {
                        parse_hdt(nmea_sentence).map(ParseResult::HDT)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::HSC => {
                cfg_if! {
                    if #[cfg(feature = "HSC")] {
                        parse_hsc(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::MDA => {
                cfg_if! {
                    if #[cfg(feature = "MDA")] {
                        parse_mda(nmea_sentence).map(ParseResult::MDA)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::MTW => {
                cfg_if! {
                    if #[cfg(feature = "MTW")] {
                        parse_mtw(nmea_sentence).map(ParseResult::MTW)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::MWD => {
                cfg_if! {
                    if #[cfg(feature = "MWD")] {
                        parse_mwd(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::MWV => {
                cfg_if! {
                    if #[cfg(feature = "MWV")] {
                        parse_mwv(nmea_sentence).map(ParseResult::MWV)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::RMB => {
                cfg_if! {
                    if #[cfg(feature = "RMB")] {
                        parse_rmb(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::RMC => {
                cfg_if! {
                    if #[cfg(feature = "RMC")] {
                        parse_rmc(nmea_sentence).map(ParseResult::RMC)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::RMZ => {
                cfg_if! {
                    if #[cfg(feature = "RMZ")] {
                        parse_pgrmz(nmea_sentence).map(ParseResult::PGRMZ)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::ROT => {
                cfg_if! {
                    if #[cfg(feature = "ROT")] {
                        parse_rot(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::RPM => {
                cfg_if! {
                    if #[cfg(feature = "RPM")] {
                        parse_rpm(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::RSA => {
                cfg_if! {
                    if #[cfg(feature = "RSA")] {
                        parse_rsa(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::TTM => {
                cfg_if! {
                    if #[cfg(feature = "TTM")] {
                        parse_ttm(nmea_sentence).map(ParseResult::TTM)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::TXT => {
                cfg_if! {
                    if #[cfg(feature = "TXT")] {
                        parse_txt(nmea_sentence).map(ParseResult::TXT)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::VDM | SentenceType::VDO => {
                cfg_if! {
                    if #[cfg(feature = "VDM")] {
                        parse_vdm(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::VDR => {
                cfg_if! {
                    if #[cfg(feature = "VDR")] {
                        parse_vdr(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::VHW => {
                cfg_if! {
                    if #[cfg(feature = "VHW")] {
                        parse_vhw(nmea_sentence).map(ParseResult::VHW)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::VLW => {
                cfg_if! {
                    if #[cfg(feature = "VLW")] {
                        parse_vlw(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::VPW => {
                cfg_if! {
                    if #[cfg(feature = "VPW")] {
                        parse_vpw(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::VTG => {
                cfg_if! {
                    if #[cfg(feature = "VTG")] {
                        parse_vtg(nmea_sentence).map(ParseResult::VTG)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::VWR => {
                cfg_if! {
                    if #[cfg(feature = "VWR")] {
                        parse_vwr(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::VWT => {
                cfg_if! {
                    if #[cfg(feature = "VWT")] {
                        parse_vwt(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::WNC => {
                cfg_if! {
                    if #[cfg(feature = "WNC")] {
                        parse_wnc(nmea_sentence).map(ParseResult::WNC)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::ZDA => {
                cfg_if! {
                    if #[cfg(feature = "ZDA")] {
                        parse_zda(nmea_sentence).map(ParseResult::ZDA)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::ZFO => {
                cfg_if! {
                    if #[cfg(feature = "ZFO")] {
                        parse_zfo(nmea_sentence).map(ParseResult::ZFO)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::ZTG => {
                cfg_if! {
                    if #[cfg(feature = "ZTG")] {
                        parse_ztg(nmea_sentence).map(ParseResult::ZTG)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::DPT => {
                cfg_if! {
                    if #[cfg(feature = "DPT")] {
                        parse_dpt(nmea_sentence).map(ParseResult::DPT)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::XDR => {
                cfg_if! {
                    if #[cfg(feature = "XDR")] {
                        parse_xdr(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            SentenceType::XTE => {
                cfg_if! {
                    if #[cfg(feature = "XTE")] {
                        parse_xte(nmea_sentence).map(Into::into)
                    } else {
                        return Err(Error::DisabledSentence);
                    }
                }
            }
            sentence_type => Ok(ParseResult::Unsupported(sentence_type)),
        }
    } else {
        Err(Error::ChecksumMismatch {
            calculated: calculated_checksum,
            found: nmea_sentence.checksum,
        })
    }
}
