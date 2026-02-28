use chrono::NaiveTime;
use nom::{
    IResult, Parser as _,
    bytes::complete::{take_until, take_while},
    character::complete::{char, one_of},
    combinator::{map_parser, opt},
    number::complete::float,
    sequence::preceded,
};

use super::{
    FaaModes,
    faa_mode::parse_faa_modes,
    utils::{number, parse_hms, parse_lat_lon},
};
use crate::{Error, SentenceType, parse::NmeaSentence};

/// GNS - Fix data
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_gns_fix_data>
///
/// ```text
///        1         2       3 4        5 6    7  8   9   10  11  12  13
///        |         |       | |        | |    |  |   |   |   |   |   |
/// $--GNS,hhmmss.ss,ddmm.mm,a,dddmm.mm,a,c--c,xx,x.x,x.x,x.x,x.x,x.x*hh
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct GnsData {
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub fix_time: Option<NaiveTime>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub faa_modes: FaaModes,
    pub nsattelites: u16,
    pub hdop: Option<f32>,
    pub alt: Option<f32>,
    pub geoid_separation: Option<f32>,
    pub age_of_differential: Option<f32>,
    pub station_id: Option<u16>,
    pub nav_status: Option<NavigationStatus>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationStatus {
    Safe,
    Caution,
    Unsafe,
    NotValidForNavigation,
}

/// # Parse GNS message
///
/// Information from gpsd:
///
/// Introduced in NMEA 4.0?
///
/// This mostly duplicates RMC, except for the multi GNSS mode
/// indicator.
///
/// ## Example (Ignore the line break):
/// ```text
/// $GPGNS,224749.00,3333.4268304,N,11153.3538273,W,D,19,0.6,406.110,
///        -26.294,6.0,0138,S,*6A
///```
///
/// 1:  224749.00     UTC HHMMSS.SS.  22:47:49.00
/// 2:  3333.4268304  Latitude DDMM.MMMMM. 33 deg. 33.4268304 min
/// 3:  N             Latitude North
/// 4:  12311.12      Longitude 111 deg. 53.3538273 min
/// 5:  W             Longitude West
/// 6:  D             FAA mode indicator
///                     see faa_mode() for possible mode values
///                     May be one to six characters.
///                       Char 1 = GPS
///                       Char 2 = GLONASS
///                       Char 3 = Galileo
///                       Char 4 = BDS
///                       Char 5 = QZSS
///                       Char 6 = NavIC (IRNSS)
/// 7:  19           Number of Satellites used in solution
/// 8:  0.6          HDOP
/// 9:  406110       MSL Altitude in meters
/// 10: -26.294      Geoid separation in meters
/// 11: 6.0          Age of differential corrections, in seconds
/// 12: 0138         Differential reference station ID
/// 13: S            NMEA 4.1+ Navigation status
///                   S = Safe
///                   C = Caution
///                   U = Unsafe
///                   V = Not valid for navigation
/// 8:   *6A          Mandatory NMEA checksum
pub fn parse_gns(sentence: NmeaSentence<'_>) -> Result<GnsData, Error<'_>> {
    if sentence.message_id != SentenceType::GNS {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::GNS,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_gns(sentence.data)?.1)
    }
}

fn do_parse_gns(i: &str) -> IResult<&str, GnsData> {
    let (i, fix_time) = opt(parse_hms).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, lat_lon) = parse_lat_lon(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, faa_modes) = map_parser(take_until(","), parse_faa_modes).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, nsattelites) = number::<u16>(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, hdop) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, alt) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, geoid_separation) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, age_of_differential) = opt(float).parse(i)?;
    let (i, _) = char(',').parse(i)?;
    let (i, station_id_str) = take_while(|c: char| c != ',').parse(i)?;
    let station_id: Option<u16> = if station_id_str.is_empty() {
        None
    } else {
        station_id_str.parse().ok()
    };
    let (i, nav_status) = opt(preceded(char(','), one_of("SCUV"))).parse(i)?;
    let nav_status = nav_status.map(|ch| match ch {
        'S' => NavigationStatus::Safe,
        'C' => NavigationStatus::Caution,
        'U' => NavigationStatus::Unsafe,
        'V' => NavigationStatus::NotValidForNavigation,
        _ => unreachable!(),
    });
    Ok((
        i,
        GnsData {
            fix_time,
            lat: lat_lon.map(|x| x.0),
            lon: lat_lon.map(|x| x.1),
            faa_modes,
            nsattelites,
            hdop,
            alt,
            geoid_separation,
            age_of_differential,
            station_id,
            nav_status,
        },
    ))
}

impl NavigationStatus {
    fn to_nmea_char(self) -> char {
        match self {
            NavigationStatus::Safe => 'S',
            NavigationStatus::Caution => 'C',
            NavigationStatus::Unsafe => 'U',
            NavigationStatus::NotValidForNavigation => 'V',
        }
    }
}

impl crate::generate::GenerateNmeaBody for GnsData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::GNS
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        use crate::sentences::gen_utils::*;

        // 1: fix_time
        write_hms(f, &self.fix_time)?;
        f.write_str(",")?;
        // 2-5: lat,N/S,lon,E/W
        write_lat_lon(f, &self.lat, &self.lon)?;
        f.write_str(",")?;
        // 6: FAA mode indicator string
        self.faa_modes.write_nmea(f)?;
        f.write_str(",")?;
        // 7: number of satellites
        write!(f, "{}", self.nsattelites)?;
        f.write_str(",")?;
        // 8: HDOP
        write_field(f, &self.hdop)?;
        // 9: altitude
        write_field(f, &self.alt)?;
        // 10: geoid separation
        write_field(f, &self.geoid_separation)?;
        // 11: age of differential corrections
        write_field(f, &self.age_of_differential)?;
        // 12: differential reference station ID
        if let Some(sid) = self.station_id {
            write!(f, "{:04}", sid)?;
        }
        if let Some(ns) = self.nav_status {
            // 13: nav status (preceded by comma)
            write!(f, ",{}", ns.to_nmea_char())?;
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
    fn test_parse_gns() {
        let s = parse_nmea_sentence("$GPGNS,224749.00,3333.4268304,N,11153.3538273,W,D,19,0.6,406.110,-26.294,6.0,0138,S,*46").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        assert_eq!(s.checksum, 0x46);
        let gns_data = parse_gns(s).unwrap();
        assert_eq!(
            gns_data.fix_time,
            Some(NaiveTime::from_hms_milli_opt(22, 47, 49, 0).expect("invalid time"))
        );
        assert_relative_eq!(33.0 + 33.4268304 / 60., gns_data.lat.unwrap());
        assert_relative_eq!(-(111.0 + 53.3538273 / 60.), gns_data.lon.unwrap());
        assert_eq!(19, gns_data.nsattelites);
        assert_relative_eq!(0.6, gns_data.hdop.unwrap());
        assert_relative_eq!(406.110, gns_data.alt.unwrap());
        assert_relative_eq!(-26.294, gns_data.geoid_separation.unwrap());
        assert_relative_eq!(6.0, gns_data.age_of_differential.unwrap());
        assert_eq!(Some(138), gns_data.station_id);
        assert_eq!(Some(NavigationStatus::Safe), gns_data.nav_status);
    }

    #[test]
    fn test_generate_gns_roundtrip() {
        // Parse a known-good sentence to get a valid GnsData (FaaModes fields are private)
        let s = parse_nmea_sentence("$GPGNS,224749.00,3333.4268304,N,11153.3538273,W,D,19,0.6,406.110,-26.294,6.0,0138,S,*46").unwrap();
        let original = parse_gns(s).unwrap();

        let mut buf = heapless::String::<256>::new();
        crate::generate::generate_sentence("GP", &original, &mut buf).unwrap();
        let s2 = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s2.checksum, s2.calc_checksum());
        let parsed = parse_gns(s2).unwrap();
        assert_eq!(parsed.fix_time, original.fix_time);
        assert_relative_eq!(parsed.lat.unwrap(), original.lat.unwrap(), epsilon = 1e-5);
        assert_relative_eq!(parsed.lon.unwrap(), original.lon.unwrap(), epsilon = 1e-5);
        assert_eq!(parsed.faa_modes, original.faa_modes);
        assert_eq!(parsed.nsattelites, original.nsattelites);
        assert_relative_eq!(parsed.hdop.unwrap(), original.hdop.unwrap());
        assert_relative_eq!(parsed.alt.unwrap(), original.alt.unwrap());
        assert_relative_eq!(
            parsed.geoid_separation.unwrap(),
            original.geoid_separation.unwrap()
        );
        assert_relative_eq!(
            parsed.age_of_differential.unwrap(),
            original.age_of_differential.unwrap()
        );
        assert_eq!(parsed.station_id, original.station_id);
        assert_eq!(parsed.nav_status, original.nav_status);
    }
}
