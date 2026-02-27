use heapless::Vec;
use nom::{IResult, Parser as _, character::complete::anychar, combinator::opt};

use super::{FixType, nom_parse_failure};

/// FAA mode indicators for multi-GNSS systems.
///
/// NMEA 4.1+ GNS sentences can carry up to 6 mode characters:
/// 1=GPS, 2=GLONASS, 3=Galileo, 4=BDS, 5=QZSS, 6=NavIC (IRNSS).
///
/// Older sentences (RMC, GLL, VTG) use a single character.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaaModes {
    modes: Vec<FaaMode, 6>,
}

impl FaaModes {
    /// Returns the first (primary) mode.
    pub fn primary(&self) -> FaaMode {
        self.modes[0]
    }

    /// Returns all mode indicators as a slice.
    pub fn as_slice(&self) -> &[FaaMode] {
        &self.modes
    }

    /// Returns the mode for a specific system index (0=GPS, 1=GLONASS, etc.)
    pub fn get(&self, index: usize) -> Option<&FaaMode> {
        self.modes.get(index)
    }

    /// Returns the number of mode indicators.
    pub fn len(&self) -> usize {
        self.modes.len()
    }

    /// Returns true if there are no mode indicators (should not happen for valid data).
    pub fn is_empty(&self) -> bool {
        self.modes.is_empty()
    }
}

impl From<FaaModes> for FixType {
    fn from(modes: FaaModes) -> Self {
        // Return the first valid fix type found across all systems
        for mode in &modes.modes {
            let fix_type: FixType = (*mode).into();
            if fix_type.is_valid() {
                return fix_type;
            }
        }
        // Fall back to primary mode's fix type
        modes.modes[0].into()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaaMode {
    /// A - Autonomous mode
    Autonomous,
    /// C - Quectel Querk, "Caution"
    Caution,
    /// D - Differential Mode
    Differential,
    /// E - Estimated (dead-reckoning) mode
    Estimated,
    /// F - RTK Float mode
    FloatRtk,
    /// M - Manual Input Mode
    Manual,
    /// N - Data Not Valid
    DataNotValid,
    /// P - Precise (4.00 and later)
    ///
    /// Sort of DGPS, NMEA 4+
    Precise,
    /// R - RTK Integer mode
    FixedRtk,
    /// S - Simulated Mode
    Simulator,
    /// U - Quectel Querk, "Unsafe"
    Unsafe,
}

impl From<FaaMode> for FixType {
    fn from(mode: FaaMode) -> Self {
        match mode {
            FaaMode::Autonomous => FixType::Gps,
            FaaMode::Caution => FixType::Invalid,
            FaaMode::Differential => FixType::DGps,
            FaaMode::Estimated => FixType::Estimated,
            FaaMode::FloatRtk => FixType::FloatRtk,
            FaaMode::DataNotValid => FixType::Invalid,
            FaaMode::Precise => FixType::DGps,
            FaaMode::FixedRtk => FixType::Rtk,
            FaaMode::Manual => FixType::Manual,
            FaaMode::Simulator => FixType::Simulation,
            FaaMode::Unsafe => FixType::Invalid,
        }
    }
}

impl FaaModes {
    /// Write the NMEA mode indicator string (e.g. "D", "ANN", "DAAN").
    pub fn write_nmea(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        for mode in &self.modes {
            f.write_char(mode.to_nmea_char())?;
        }
        Ok(())
    }
}

pub(crate) fn parse_faa_modes(i: &str) -> IResult<&str, FaaModes> {
    let (mut rest, sym) = anychar(i)?;
    let mut modes = Vec::<FaaMode, 6>::new();
    modes
        .push(parse_faa_mode(sym).ok_or_else(|| nom_parse_failure(i))?)
        .unwrap();

    // Parse up to 5 more mode characters
    for _ in 0..5 {
        let (next, sym) = match opt(anychar).parse(rest)? {
            (r, Some(s)) => (r, s),
            (r, None) => {
                rest = r;
                break;
            }
        };
        let mode = parse_faa_mode(sym).ok_or_else(|| nom_parse_failure(rest))?;
        let _ = modes.push(mode);
        rest = next;
    }

    Ok((rest, FaaModes { modes }))
}

impl FaaMode {
    /// Convert to the single-character NMEA representation.
    #[inline]
    pub fn to_nmea_char(self) -> char {
        match self {
            FaaMode::Autonomous => 'A',
            FaaMode::Caution => 'C',
            FaaMode::Differential => 'D',
            FaaMode::Estimated => 'E',
            FaaMode::FloatRtk => 'F',
            FaaMode::Manual => 'M',
            FaaMode::DataNotValid => 'N',
            FaaMode::Precise => 'P',
            FaaMode::FixedRtk => 'R',
            FaaMode::Simulator => 'S',
            FaaMode::Unsafe => 'U',
        }
    }
}

pub(crate) fn parse_faa_mode(value: char) -> Option<FaaMode> {
    match value {
        'A' => Some(FaaMode::Autonomous),
        'C' => Some(FaaMode::Caution),
        'D' => Some(FaaMode::Differential),
        'E' => Some(FaaMode::Estimated),
        'F' => Some(FaaMode::FloatRtk),
        'N' => Some(FaaMode::DataNotValid),
        'P' => Some(FaaMode::Precise),
        'R' => Some(FaaMode::FixedRtk),
        'M' => Some(FaaMode::Manual),
        'S' => Some(FaaMode::Simulator),
        'U' => Some(FaaMode::Unsafe),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn modes(chars: &[FaaMode]) -> FaaModes {
        let mut modes = Vec::<FaaMode, 6>::new();
        for &m in chars {
            modes.push(m).unwrap();
        }
        FaaModes { modes }
    }

    #[test]
    fn test_parse_faa_modes() {
        assert_eq!(
            nom::Err::Error(nom::error::Error::new("", nom::error::ErrorKind::Eof)),
            parse_faa_modes("").unwrap_err(),
            "Should return a Digit error on empty string"
        );
        assert_eq!(
            ("", modes(&[FaaMode::Autonomous])),
            parse_faa_modes("A").unwrap()
        );

        assert_eq!(
            ("", modes(&[FaaMode::DataNotValid, FaaMode::Autonomous])),
            parse_faa_modes("NA").unwrap()
        );
    }

    #[test]
    fn test_parse_faa_modes_multi_gnss() {
        // 3-char: GPS=A, GLONASS=N, Galileo=N
        let (rest, parsed) = parse_faa_modes("ANN").unwrap();
        assert_eq!(rest, "");
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed.primary(), FaaMode::Autonomous);
        assert_eq!(parsed.get(1), Some(&FaaMode::DataNotValid));
        assert_eq!(parsed.get(2), Some(&FaaMode::DataNotValid));

        // 4-char: GPS=D, GLONASS=A, Galileo=A, BDS=N
        let (rest, parsed) = parse_faa_modes("DAAN").unwrap();
        assert_eq!(rest, "");
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed.primary(), FaaMode::Differential);

        // 6-char: all systems
        let (rest, parsed) = parse_faa_modes("DAANNP").unwrap();
        assert_eq!(rest, "");
        assert_eq!(parsed.len(), 6);
    }

    #[test]
    fn test_faa_modes_fix_type_multi() {
        // First valid fix type wins
        let m = modes(&[
            FaaMode::DataNotValid,
            FaaMode::Autonomous,
            FaaMode::DataNotValid,
        ]);
        let fix: FixType = m.into();
        assert_eq!(fix, FixType::Gps); // Autonomous maps to GPS
    }

    #[test]
    fn test_faa_modes_write_nmea() {
        let m = modes(&[
            FaaMode::Autonomous,
            FaaMode::DataNotValid,
            FaaMode::DataNotValid,
        ]);
        let mut buf = heapless::String::<8>::new();
        m.write_nmea(&mut buf).unwrap();
        assert_eq!(&*buf, "ANN");
    }
}
