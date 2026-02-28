use arrayvec::ArrayString;
use heapless::Vec;
use nom::{
    IResult, Parser as _,
    character::complete::{anychar, char},
    combinator::opt,
    number::complete::double,
};

use crate::{Error, ParseResult, SentenceType, parse::NmeaSentence};

/// XDR - Transducer Measurement
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_xdr_transducer_measurement>
///
/// ```text
///        1 2   3 4       n
///        | |   | |       |
/// $--XDR,a,x.x,a,c--c,…*hh<CR><LF>
/// ```
///
/// Fields repeat in groups of 4:
///
/// 1. Transducer Type (A=Angular, C=Temperature, etc.)
/// 2. Measurement Data
/// 3. Units of Measurement
/// 4. Name of Transducer
///
/// Examples:
/// * `$WIXDR,C,24.3,C,ENV_TEMP*45`
/// * `$HCXDR,A,171,D,PITCH,A,-37,D,ROLL*00`
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct XdrData {
    pub measurements: Vec<XdrMeasurement, 8>,
}

/// A single transducer measurement group within an XDR sentence.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct XdrMeasurement {
    /// Transducer type: A=Angular, C=Temperature, D=Linear, F=Frequency,
    /// H=Humidity, N=Force, P=Pressure, R=Flow, T=Tachometer, U=Volume, V=Generic
    pub transducer_type: Option<char>,
    /// Measurement data value
    pub value: Option<f64>,
    /// Units: B=Bars, C=Celsius, D=Degrees, H=Hertz, I=Liters/sec,
    /// K=Kelvin, M=Meters, N=Newton, P=Pascal/Percent, R=RPM, S=ppt, V=Volts
    pub units: Option<char>,
    /// Name of transducer (e.g. "PITCH", "ROLL", "ENV_TEMP")
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub name: ArrayString<16>,
}

impl From<XdrData> for ParseResult {
    fn from(value: XdrData) -> Self {
        ParseResult::XDR(value)
    }
}

/// Parse one measurement group: type,value,units,name
fn parse_xdr_group(i: &str) -> IResult<&str, XdrMeasurement> {
    // Type (single char, optional)
    let (i, transducer_type) = opt(anychar).parse(i)?;
    // Filter out comma parsed as type
    let transducer_type = transducer_type.filter(|&c| c != ',');
    let i = if transducer_type.is_some() {
        let (i, _) = char(',').parse(i)?;
        i
    } else if let Some(stripped) = i.strip_prefix(',') {
        stripped
    } else {
        i
    };

    // Value (optional float)
    let (i, value) = opt(double).parse(i)?;
    let (i, _) = char(',').parse(i)?;

    // Units (single char, optional)
    let (i, units) = opt(anychar).parse(i)?;
    let units = units.filter(|&c| c != ',');
    let i = if units.is_some() {
        let (i, _) = char(',').parse(i)?;
        i
    } else if let Some(stripped) = i.strip_prefix(',') {
        stripped
    } else {
        i
    };

    // Name (until comma or end)
    let comma_pos = i.find(',').unwrap_or(i.len());
    let name_str = &i[..comma_pos];
    let i = &i[comma_pos..];

    let mut name = ArrayString::<16>::new();
    // Truncate if too long
    let take = name_str.len().min(16);
    name.push_str(&name_str[..take]);

    Ok((
        i,
        XdrMeasurement {
            transducer_type,
            value,
            units,
            name,
        },
    ))
}

fn do_parse_xdr(i: &str) -> IResult<&str, XdrData> {
    let mut measurements = Vec::<XdrMeasurement, 8>::new();
    let mut remaining = i;

    loop {
        if remaining.is_empty() {
            break;
        }

        let (rest, measurement) = parse_xdr_group(remaining)?;
        let _ = measurements.push(measurement);

        if rest.is_empty() {
            remaining = rest;
            break;
        }

        // Consume separator comma between groups
        if let Some(stripped) = rest.strip_prefix(',') {
            remaining = stripped;
        } else {
            remaining = rest;
            break;
        }
    }

    Ok((remaining, XdrData { measurements }))
}

/// # Parse XDR message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_xdr_transducer_measurement>
pub fn parse_xdr(sentence: NmeaSentence<'_>) -> Result<XdrData, Error<'_>> {
    if sentence.message_id != SentenceType::XDR {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::XDR,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_xdr(sentence.data)?.1)
    }
}

impl crate::generate::GenerateNmeaBody for XdrData {
    fn sentence_type(&self) -> SentenceType {
        SentenceType::XDR
    }

    fn write_body(&self, f: &mut dyn core::fmt::Write) -> core::fmt::Result {
        for (idx, m) in self.measurements.iter().enumerate() {
            if idx > 0 {
                f.write_char(',')?;
            }
            if let Some(t) = m.transducer_type {
                f.write_char(t)?;
            }
            f.write_char(',')?;
            if let Some(v) = m.value {
                write!(f, "{}", v)?;
            }
            f.write_char(',')?;
            if let Some(u) = m.units {
                f.write_char(u)?;
            }
            f.write_char(',')?;
            f.write_str(&m.name)?;
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
    fn test_parse_xdr_single_measurement() {
        let s = parse_nmea_sentence("$WIXDR,C,24.3,C,ENV_TEMP*45").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_xdr(s).unwrap();
        assert_eq!(data.measurements.len(), 1);
        let m = &data.measurements[0];
        assert_eq!(m.transducer_type, Some('C'));
        assert_relative_eq!(m.value.unwrap(), 24.3);
        assert_eq!(m.units, Some('C'));
        assert_eq!(m.name.as_str(), "ENV_TEMP");
    }

    #[test]
    fn test_parse_xdr_multiple_measurements() {
        let s = parse_nmea_sentence("$HCXDR,A,171,D,PITCH,A,-37,D,ROLL*00").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_xdr(s).unwrap();
        assert_eq!(data.measurements.len(), 2);

        assert_eq!(data.measurements[0].transducer_type, Some('A'));
        assert_relative_eq!(data.measurements[0].value.unwrap(), 171.0);
        assert_eq!(data.measurements[0].units, Some('D'));
        assert_eq!(data.measurements[0].name.as_str(), "PITCH");

        assert_eq!(data.measurements[1].transducer_type, Some('A'));
        assert_relative_eq!(data.measurements[1].value.unwrap(), -37.0);
        assert_eq!(data.measurements[1].units, Some('D'));
        assert_eq!(data.measurements[1].name.as_str(), "ROLL");
    }

    #[test]
    fn test_parse_xdr_pressure() {
        let s = parse_nmea_sentence("$WIXDR,P,1.0135,B,BARO*44").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_xdr(s).unwrap();
        assert_eq!(data.measurements.len(), 1);
        assert_eq!(data.measurements[0].transducer_type, Some('P'));
        assert_relative_eq!(data.measurements[0].value.unwrap(), 1.0135);
        assert_eq!(data.measurements[0].units, Some('B'));
        assert_eq!(data.measurements[0].name.as_str(), "BARO");
    }

    #[test]
    fn test_parse_xdr_empty_value() {
        let s = parse_nmea_sentence("$WIXDR,C,,C,TEMP*5C").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let data = parse_xdr(s).unwrap();
        assert_eq!(data.measurements.len(), 1);
        assert_eq!(data.measurements[0].transducer_type, Some('C'));
        assert_eq!(data.measurements[0].value, None);
        assert_eq!(data.measurements[0].units, Some('C'));
        assert_eq!(data.measurements[0].name.as_str(), "TEMP");
    }

    #[test]
    fn test_generate_xdr_roundtrip() {
        let original = XdrData {
            measurements: {
                let mut v = Vec::new();
                v.push(XdrMeasurement {
                    transducer_type: Some('C'),
                    value: Some(24.3),
                    units: Some('C'),
                    name: {
                        let mut s = ArrayString::new();
                        s.push_str("ENV_TEMP");
                        s
                    },
                })
                .unwrap();
                v
            },
        };
        let mut buf = heapless::String::<256>::new();
        crate::generate::generate_sentence("WI", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_xdr(s).unwrap();
        assert_eq!(parsed.measurements.len(), 1);
        assert_eq!(parsed.measurements[0].transducer_type, Some('C'));
        assert_relative_eq!(parsed.measurements[0].value.unwrap(), 24.3);
        assert_eq!(parsed.measurements[0].name.as_str(), "ENV_TEMP");
    }

    #[test]
    fn test_generate_xdr_multi_roundtrip() {
        let original = XdrData {
            measurements: {
                let mut v = Vec::new();
                v.push(XdrMeasurement {
                    transducer_type: Some('A'),
                    value: Some(5.2),
                    units: Some('D'),
                    name: {
                        let mut s = ArrayString::new();
                        s.push_str("PITCH");
                        s
                    },
                })
                .unwrap();
                v.push(XdrMeasurement {
                    transducer_type: Some('A'),
                    value: Some(-3.1),
                    units: Some('D'),
                    name: {
                        let mut s = ArrayString::new();
                        s.push_str("ROLL");
                        s
                    },
                })
                .unwrap();
                v
            },
        };
        let mut buf = heapless::String::<256>::new();
        crate::generate::generate_sentence("HC", &original, &mut buf).unwrap();

        let s = parse_nmea_sentence(&buf).unwrap();
        assert_eq!(s.checksum, s.calc_checksum());
        let parsed = parse_xdr(s).unwrap();
        assert_eq!(parsed.measurements.len(), 2);
        assert_eq!(parsed.measurements[0].name.as_str(), "PITCH");
        assert_eq!(parsed.measurements[1].name.as_str(), "ROLL");
    }
}
