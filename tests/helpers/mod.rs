use heapless::Vec;
use nmea::Satellite;

/// ensure right order before dump to string
pub fn format_satellites(mut sats: Vec<Satellite, 58>) -> std::vec::Vec<String> {
    sats.sort_by_key(|s| (s.gnss_type() as u8, s.prn()));
    // to not depend on Debug impl for `Satellite` stability

    sats.iter()
        .map(|s| {
            format!(
                "{{{gnss_type:?} {prn} {elevation:?} {azimuth:?} {snr:?}}}",
                gnss_type = s.gnss_type(),
                prn = s.prn(),
                elevation = s.elevation(),
                azimuth = s.azimuth(),
                snr = s.snr(),
            )
        })
        .collect::<std::vec::Vec<String>>()
}

/// Build a complete NMEA sentence with the correct checksum.
///
/// `talker_id` is the 2-char talker (e.g. "GP"),
/// `sentence_type` is the 3-char type (e.g. "GGA"),
/// `fields` is the comma-separated data after the first comma.
///
/// Returns a string like `$GPGGA,<fields>*HH`.
pub fn build_sentence(talker_id: &str, sentence_type: &str, fields: &str) -> String {
    let body = format!("{}{},{}", talker_id, sentence_type, fields);
    let cs = body.as_bytes().iter().fold(0u8, |c, x| c ^ x);
    format!("${}*{:02X}", body, cs)
}

/// Build a sentence with a deliberately wrong checksum (XOR'd with 0xFF).
pub fn build_sentence_bad_checksum(talker_id: &str, sentence_type: &str, fields: &str) -> String {
    let body = format!("{}{},{}", talker_id, sentence_type, fields);
    let cs = body.as_bytes().iter().fold(0u8, |c, x| c ^ x) ^ 0xFF;
    format!("${}*{:02X}", body, cs)
}

/// Build a sentence that is truncated (no checksum).
pub fn build_truncated_sentence(talker_id: &str, sentence_type: &str, fields: &str) -> String {
    format!("${}{},{}", talker_id, sentence_type, fields)
}

/// Build an AIS sentence with `!` prefix and correct checksum.
pub fn build_ais_sentence(talker_id: &str, sentence_type: &str, fields: &str) -> String {
    let body = format!("{}{},{}", talker_id, sentence_type, fields);
    let cs = body.as_bytes().iter().fold(0u8, |c, x| c ^ x);
    format!("!{}*{:02X}", body, cs)
}
