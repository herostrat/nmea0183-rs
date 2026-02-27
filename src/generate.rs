//! NMEA 0183 sentence generation.
//!
//! This module provides the [`GenerateNmeaBody`] trait and [`generate_sentence`]
//! function for serializing data structures back into valid NMEA 0183 sentences.
//!
//! # Example
//!
//! ```
//! use nmea::generate::{GenerateNmeaBody, generate_sentence};
//! use nmea::sentences::HdtData;
//!
//! let data = HdtData { heading: Some(274.07) };
//! let mut buf = heapless::String::<128>::new();
//! generate_sentence("GP", &data, &mut buf).unwrap();
//! assert!(buf.starts_with("$GPHDT,"));
//! assert!(buf.contains('*'));
//! ```

use core::fmt::{self, Write as _};

use crate::SentenceType;
use crate::parse::{SENTENCE_MAX_LEN, checksum};

/// Trait for types that can be serialized as an NMEA 0183 sentence body.
///
/// Implementors write the comma-separated data fields of the sentence
/// (everything between `$XXYY,` and `*HH`).
pub trait GenerateNmeaBody {
    /// The NMEA sentence type identifier (e.g., GGA, RMC).
    fn sentence_type(&self) -> SentenceType;

    /// Write the comma-separated data fields of the sentence body.
    ///
    /// This should NOT include the leading `$XX<TYPE>,` header
    /// or the trailing `*HH` checksum.
    fn write_body(&self, f: &mut dyn fmt::Write) -> fmt::Result;
}

/// Generate a complete NMEA 0183 sentence.
///
/// Writes `$<talker_id><sentence_type>,<body>*<checksum>` to `out`.
/// Does NOT append `\r\n` -- the caller can add that if needed for serial output.
///
/// # Arguments
/// * `talker_id` - Two-character talker ID (e.g., "GP", "GN", "SD")
/// * `data` - The sentence data implementing [`GenerateNmeaBody`]
/// * `out` - The output writer
///
/// # Example
///
/// ```
/// use nmea::generate::{GenerateNmeaBody, generate_sentence};
/// use nmea::sentences::HdtData;
///
/// let data = HdtData { heading: Some(274.07) };
/// let mut buf = String::new();
/// generate_sentence("GP", &data, &mut buf).unwrap();
/// ```
pub fn generate_sentence(
    talker_id: &str,
    data: &dyn GenerateNmeaBody,
    out: &mut dyn fmt::Write,
) -> fmt::Result {
    // Build the checksummed portion into a temporary buffer.
    // The checksummed portion is: talker_id + sentence_type + "," + body
    let mut buf = heapless::String::<SENTENCE_MAX_LEN>::new();

    write!(buf, "{}{},", talker_id, data.sentence_type().as_str())?;
    data.write_body(&mut buf)?;

    let cs = checksum(buf.as_bytes().iter());

    write!(out, "${}*{:02X}", buf.as_str(), cs)
}
