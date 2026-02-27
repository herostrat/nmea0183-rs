//! NMEA 0183 conformance test suite.
//!
//! Covers checksum validation, sentence length limits, talker ID acceptance,
//! field edge cases, AIS sentence handling, and error handling for malformed input.

use nmea::{parse_nmea_sentence, parse_str, Error, ParseResult, SentenceType, SENTENCE_MAX_LEN};

mod helpers;

use helpers::{build_ais_sentence, build_sentence, build_sentence_bad_checksum, build_truncated_sentence};

/// Helper macro to unwrap an error from `Result<NmeaSentence, Error>`, since
/// `NmeaSentence` does not implement `Debug` and thus `expect_err` is unavailable.
macro_rules! unwrap_parse_err {
    ($result:expr, $msg:expr $(,)?) => {
        match $result {
            Err(e) => e,
            Ok(_) => panic!("{}: expected Err but got Ok", $msg),
        }
    };
}

// ---------------------------------------------------------------------------
// 1. Checksum tests
// ---------------------------------------------------------------------------

mod checksum_tests {
    use super::*;

    /// A valid GGA sentence must have matching calculated and embedded checksums.
    #[test]
    fn valid_checksum_gga() {
        let sentence = build_sentence("GP", "GGA", "092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum(), "checksum must match for GGA");
    }

    /// A valid RMC sentence must have matching checksums.
    #[test]
    fn valid_checksum_rmc() {
        let sentence = build_sentence("GP", "RMC", "225446.33,A,4916.45,N,12311.12,W,000.5,054.7,191194,020.3,E,A");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A valid GSA sentence must have matching checksums.
    #[test]
    fn valid_checksum_gsa() {
        let sentence = build_sentence("GP", "GSA", "A,3,23,31,22,16,03,07,,,,,,,1.8,1.1,1.4");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A valid GSV sentence must have matching checksums.
    #[test]
    fn valid_checksum_gsv() {
        let sentence = build_sentence("GP", "GSV", "3,1,12,01,49,196,41,03,71,278,32,06,02,323,27,11,21,196,39");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A valid GLL sentence must have matching checksums.
    #[test]
    fn valid_checksum_gll() {
        let sentence = build_sentence("GP", "GLL", "5107.0013414,N,11402.3279144,W,205412.00,A,A");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A valid VTG sentence must have matching checksums.
    #[test]
    fn valid_checksum_vtg() {
        let sentence = build_sentence("GP", "VTG", "360.0,T,348.7,M,000.0,N,000.0,K");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A valid GNS sentence must have matching checksums.
    #[test]
    fn valid_checksum_gns() {
        let sentence = build_sentence("GP", "GNS", "224749.00,3333.4268304,N,11153.3538273,W,D,19,0.6,406.110,-26.294,6.0,0138,S,");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A valid GBS sentence must have matching checksums.
    #[test]
    fn valid_checksum_gbs() {
        let sentence = build_sentence("GP", "GBS", "182141.000,15.5,15.3,7.2,21,0.9,0.5,0.8");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A valid GST sentence must have matching checksums.
    #[test]
    fn valid_checksum_gst() {
        let sentence = build_sentence("GP", "GST", "182141.000,15.5,15.3,7.2,21.8,0.9,0.5,0.8");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A valid HDT sentence must have matching checksums.
    #[test]
    fn valid_checksum_hdt() {
        let sentence = build_sentence("GP", "HDT", "274.07,T");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A valid ZDA sentence must have matching checksums.
    #[test]
    fn valid_checksum_zda() {
        let sentence = build_sentence("GP", "ZDA", "160012.71,11,03,2004,-1,00");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A valid TXT sentence must have matching checksums.
    #[test]
    fn valid_checksum_txt() {
        let sentence = build_sentence("GN", "TXT", "01,01,02,u-blox AG - www.u-blox.com");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A valid DBT sentence must have matching checksums.
    #[test]
    fn valid_checksum_dbt() {
        let sentence = build_sentence("SD", "DBT", "12.3,f,3.75,M,2.05,F");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A valid MWV sentence must have matching checksums.
    #[test]
    fn valid_checksum_mwv() {
        let sentence = build_sentence("WI", "MWV", "041.1,R,01.0,N,A");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A bad checksum must be detected and produce a ChecksumMismatch error
    /// when parsed through parse_str (which validates checksums).
    #[test]
    fn bad_checksum_rejected_by_parse_str() {
        let sentence = build_sentence_bad_checksum("GP", "GGA", "092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,");
        let err = parse_str(&sentence).expect_err("bad checksum should be rejected");
        assert!(
            matches!(err, Error::ChecksumMismatch { .. }),
            "expected ChecksumMismatch, got: {:?}",
            err
        );
    }

    /// When parse_nmea_sentence succeeds on a bad-checksum sentence, the
    /// checksum field and calc_checksum() must disagree.
    #[test]
    fn bad_checksum_detected_via_nmea_sentence() {
        let sentence = build_sentence_bad_checksum("GP", "RMC", "225446.33,A,4916.45,N,12311.12,W,000.5,054.7,191194,020.3,E,A");
        let parsed = parse_nmea_sentence(&sentence).expect("low-level parse should succeed");
        assert_ne!(
            parsed.checksum,
            parsed.calc_checksum(),
            "bad checksum should not match calculated checksum"
        );
    }

    /// Multiple sentence types with bad checksums should all be rejected.
    #[test]
    fn bad_checksum_rejected_multiple_types() {
        let sentences = [
            build_sentence_bad_checksum("GP", "GSA", "A,3,,,,,,,,,,,,,99.99,99.99,99.99"),
            build_sentence_bad_checksum("GP", "VTG", "360.0,T,348.7,M,000.0,N,000.0,K"),
            build_sentence_bad_checksum("GP", "GLL", "5107.0013414,N,11402.3279144,W,205412.00,A,A"),
            build_sentence_bad_checksum("GP", "HDT", "274.07,T"),
        ];
        for sentence in &sentences {
            let err = parse_str(sentence).expect_err("bad checksum should be rejected");
            assert!(
                matches!(err, Error::ChecksumMismatch { .. }),
                "expected ChecksumMismatch for sentence '{}', got: {:?}",
                sentence,
                err
            );
        }
    }

    /// Verify the checksum XOR computation matches known values.
    #[test]
    fn checksum_known_value() {
        // Known good sentence from the NMEA spec / gpsd test data
        let sentence = "$GPGGA,092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,*76";
        let parsed = parse_nmea_sentence(sentence).expect("should parse");
        assert_eq!(parsed.checksum, 0x76);
        assert_eq!(parsed.calc_checksum(), 0x76);
    }
}

// ---------------------------------------------------------------------------
// 2. Sentence length tests
// ---------------------------------------------------------------------------

mod sentence_length_tests {
    use super::*;

    /// The maximum sentence length constant must be 102.
    #[test]
    fn max_len_constant_is_102() {
        assert_eq!(SENTENCE_MAX_LEN, 102);
    }

    /// A sentence of exactly SENTENCE_MAX_LEN (102) characters should be accepted.
    #[test]
    fn sentence_at_max_length_accepted() {
        // Build a sentence that is exactly 102 characters.
        // $GPGGA,<fields>*HH  -- we need to pad fields to hit exactly 102 chars.
        let prefix = "$GPGGA,";
        let suffix_len = 3; // "*HH"
        let body_for_checksum_prefix = "GPGGA,";
        // Total = prefix.len() + fields.len() + suffix_len = 102
        // fields.len() = 102 - 7 - 3 = 92
        let fields_len = SENTENCE_MAX_LEN - prefix.len() - suffix_len;
        // Create fields of the right length: commas are valid NMEA field content.
        let fields: String = std::iter::repeat_n(',', fields_len).collect();

        let body = format!("{}{}", body_for_checksum_prefix, &fields);
        let cs = body.as_bytes().iter().fold(0u8, |c, x| c ^ x);
        let sentence = format!("{}{}*{:02X}", prefix, fields, cs);

        assert_eq!(sentence.len(), SENTENCE_MAX_LEN);
        assert!(
            parse_nmea_sentence(&sentence).is_ok(),
            "102-char sentence should be accepted"
        );
    }

    /// A sentence of SENTENCE_MAX_LEN + 1 (103) characters must be rejected
    /// with a SentenceLength error.
    #[test]
    fn sentence_over_max_length_rejected() {
        let prefix = "$GPGGA,";
        let suffix_len = 3; // "*HH"
        let body_for_checksum_prefix = "GPGGA,";
        let fields_len = (SENTENCE_MAX_LEN + 1) - prefix.len() - suffix_len;
        let fields: String = std::iter::repeat_n(',', fields_len).collect();

        let body = format!("{}{}", body_for_checksum_prefix, &fields);
        let cs = body.as_bytes().iter().fold(0u8, |c, x| c ^ x);
        let sentence = format!("{}{}*{:02X}", prefix, fields, cs);

        assert_eq!(sentence.len(), SENTENCE_MAX_LEN + 1);
        let err = unwrap_parse_err!(
            parse_nmea_sentence(&sentence),
            "103-char sentence should be rejected",
        );
        assert!(
            matches!(err, Error::SentenceLength(len) if len == SENTENCE_MAX_LEN + 1),
            "expected SentenceLength(103), got: {:?}",
            err
        );
    }

    /// An empty string must produce a parsing error.
    #[test]
    fn empty_sentence_rejected() {
        let err = unwrap_parse_err!(
            parse_nmea_sentence(""),
            "empty sentence should fail",
        );
        assert!(
            matches!(err, Error::ParsingError(_)),
            "expected ParsingError for empty input, got: {:?}",
            err
        );
    }

    /// A very short incomplete sentence must produce a parsing error.
    #[test]
    fn minimal_incomplete_sentence_rejected() {
        let err = unwrap_parse_err!(
            parse_nmea_sentence("$"),
            "single $ should fail",
        );
        assert!(
            matches!(err, Error::ParsingError(_)),
            "expected ParsingError for '$', got: {:?}",
            err
        );
    }

    /// parse_str also rejects oversized sentences (it calls parse_nmea_sentence internally).
    #[test]
    fn parse_str_rejects_oversized() {
        let prefix = "$GPGGA,";
        let suffix_len = 3;
        let body_for_checksum_prefix = "GPGGA,";
        let fields_len = (SENTENCE_MAX_LEN + 1) - prefix.len() - suffix_len;
        let fields: String = std::iter::repeat_n(',', fields_len).collect();

        let body = format!("{}{}", body_for_checksum_prefix, &fields);
        let cs = body.as_bytes().iter().fold(0u8, |c, x| c ^ x);
        let sentence = format!("{}{}*{:02X}", prefix, fields, cs);

        let err = parse_str(&sentence).expect_err("oversized sentence should be rejected");
        assert!(matches!(err, Error::SentenceLength(_)));
    }
}

// ---------------------------------------------------------------------------
// 3. Talker ID tests
// ---------------------------------------------------------------------------

mod talker_id_tests {
    use super::*;

    /// All standard GNSS talker IDs should be accepted by the low-level parser.
    /// GP = GPS, GL = GLONASS, GA = Galileo, GB = BeiDou,
    /// GN = multi-constellation, BD = BeiDou (alt), GI = NavIC, QZ = QZSS.
    #[test]
    fn standard_talker_ids_accepted() {
        let talker_ids = ["GP", "GL", "GA", "GB", "GN", "BD", "GI", "QZ"];
        // Use GGA with minimal valid fields for each talker ID
        let fields = "092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,";
        for talker_id in &talker_ids {
            let sentence = build_sentence(talker_id, "GGA", fields);
            let parsed = parse_nmea_sentence(&sentence)
                .unwrap_or_else(|e| panic!("talker ID '{}' should be accepted, got error: {:?}", talker_id, e));
            assert_eq!(parsed.talker_id, *talker_id, "talker ID should be preserved");
            assert_eq!(parsed.message_id, SentenceType::GGA);
        }
    }

    /// parse_str should successfully parse GGA with various talker IDs.
    #[test]
    fn talker_ids_with_parse_str() {
        let talker_ids = ["GP", "GL", "GA", "GB", "GN"];
        let fields = "092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,";
        for talker_id in &talker_ids {
            let sentence = build_sentence(talker_id, "GGA", fields);
            let result = parse_str(&sentence);
            assert!(
                result.is_ok(),
                "talker ID '{}' should parse via parse_str, got: {:?}",
                talker_id,
                result.err()
            );
        }
    }

    /// Maritime talker IDs should work with heading sentences.
    #[test]
    fn maritime_talker_ids() {
        // HC is a common heading compass talker ID, HE is gyro
        let talker_ids = ["HC", "HE", "II"];
        let fields = "274.07,T";
        for talker_id in &talker_ids {
            let sentence = build_sentence(talker_id, "HDT", fields);
            let parsed = parse_nmea_sentence(&sentence)
                .unwrap_or_else(|e| panic!("talker ID '{}' should be accepted, got error: {:?}", talker_id, e));
            assert_eq!(parsed.talker_id, *talker_id);
            assert_eq!(parsed.message_id, SentenceType::HDT);
        }
    }

    /// Weather instrument talker ID (WI) should work.
    #[test]
    fn weather_instrument_talker_id() {
        let sentence = build_sentence("WI", "MWV", "041.1,R,01.0,N,A");
        let result = parse_str(&sentence);
        assert!(result.is_ok(), "WI talker ID should work with MWV: {:?}", result.err());
    }

    /// Sounder / depth talker ID (SD) should work.
    #[test]
    fn sounder_talker_id() {
        let sentence = build_sentence("SD", "DBT", "12.3,f,3.75,M,2.05,F");
        let result = parse_str(&sentence);
        assert!(result.is_ok(), "SD talker ID should work with DBT: {:?}", result.err());
    }

    /// The talker ID field should be exactly preserved in parsing output.
    #[test]
    fn talker_id_preserved_in_output() {
        let sentence = build_sentence("QZ", "GGA", "092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.talker_id, "QZ");
    }
}

// ---------------------------------------------------------------------------
// 4. Field edge cases
// ---------------------------------------------------------------------------

mod field_edge_cases {
    use super::*;

    /// A sentence with all empty fields (just commas) should be parseable
    /// at the low-level NmeaSentence layer.
    #[test]
    fn all_empty_fields_low_level() {
        let sentence = build_sentence("GP", "GGA", ",,,,,,,,,,,,,,");
        assert!(
            parse_nmea_sentence(&sentence).is_ok(),
            "empty fields should parse at low level"
        );
    }

    /// A sentence with a single empty field should parse at the low level.
    #[test]
    fn single_empty_field() {
        let sentence = build_sentence("GP", "HDT", ",T");
        assert!(
            parse_nmea_sentence(&sentence).is_ok(),
            "single empty field should parse"
        );
    }

    /// An HDT sentence with an empty heading field should still be parseable
    /// through parse_str (the high-level parser handles optional fields).
    #[test]
    fn hdt_empty_heading() {
        let sentence = build_sentence("GP", "HDT", ",T");
        let result = parse_str(&sentence);
        assert!(result.is_ok(), "HDT with empty heading should parse: {:?}", result.err());
    }

    /// A VTG sentence with all empty numeric fields should be parseable.
    #[test]
    fn vtg_all_empty_numeric_fields() {
        let sentence = build_sentence("GP", "VTG", ",T,,M,,N,,K");
        let result = parse_str(&sentence);
        assert!(result.is_ok(), "VTG with all empty fields should parse: {:?}", result.err());
    }

    /// An RMC sentence with many empty fields should parse
    /// (mode indicator and other fields often omitted).
    #[test]
    fn rmc_minimal_fields() {
        let sentence = build_sentence("GP", "RMC", "225446.33,A,4916.45,N,12311.12,W,,,191194,,,A");
        let result = parse_str(&sentence);
        assert!(result.is_ok(), "RMC with some empty fields should parse: {:?}", result.err());
    }

    /// A GGA with zero satellites and invalid fix should still parse.
    #[test]
    fn gga_zero_satellites() {
        let sentence = build_sentence("GP", "GGA", "133605.0,5521.75946,N,03731.93769,E,0,00,,,M,,M,,");
        let result = parse_str(&sentence);
        assert!(result.is_ok(), "GGA with 0 fix should parse: {:?}", result.err());
    }

    /// GSV with minimal satellite data (some fields empty) should parse.
    #[test]
    fn gsv_partial_satellite_data() {
        let sentence = build_sentence("GP", "GSV", "1,1,01,36,,,");
        let result = parse_str(&sentence);
        assert!(result.is_ok(), "GSV with partial sat data should parse: {:?}", result.err());
    }

    /// The data field of a parsed sentence should contain the expected content.
    #[test]
    fn data_field_content_preserved() {
        let fields = "274.07,T";
        let sentence = build_sentence("GP", "HDT", fields);
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.data, fields, "data field must be preserved exactly");
    }

    /// The message_id must correctly identify the sentence type.
    #[test]
    fn message_id_correct() {
        let sentence_types = [
            ("GGA", SentenceType::GGA),
            ("RMC", SentenceType::RMC),
            ("GSA", SentenceType::GSA),
            ("GSV", SentenceType::GSV),
            ("GLL", SentenceType::GLL),
            ("VTG", SentenceType::VTG),
            ("HDT", SentenceType::HDT),
            ("ZDA", SentenceType::ZDA),
            ("TXT", SentenceType::TXT),
            ("MWV", SentenceType::MWV),
        ];
        for (type_str, expected_type) in &sentence_types {
            let sentence = build_sentence("GP", type_str, ",,");
            let parsed = parse_nmea_sentence(&sentence).expect("should parse");
            assert_eq!(parsed.message_id, *expected_type, "message_id should be {:?}", expected_type);
        }
    }
}

// ---------------------------------------------------------------------------
// 5. AIS sentence tests
// ---------------------------------------------------------------------------

mod ais_tests {
    use super::*;

    /// An AIS VDM sentence with the `!` prefix should parse at the low level.
    #[test]
    fn ais_vdm_low_level_parse() {
        let sentence = build_ais_sentence("AI", "VDM", "1,1,,A,13aEOK?P00PD2wVMdLDRhgvL289?,0");
        let parsed = parse_nmea_sentence(&sentence)
            .unwrap_or_else(|e| panic!("AIS VDM should parse: {:?}", e));
        assert_eq!(parsed.message_id, SentenceType::VDM);
        assert_eq!(parsed.talker_id, "AI");
    }

    /// An AIS VDM sentence should parse through parse_str.
    #[test]
    fn ais_vdm_parse_str() {
        let sentence = build_ais_sentence("AI", "VDM", "1,1,,A,13aEOK?P00PD2wVMdLDRhgvL289?,0");
        let result = parse_str(&sentence);
        assert!(result.is_ok(), "AIS VDM should parse via parse_str: {:?}", result.err());
    }

    /// An AIS VDO sentence should also parse with the `!` prefix.
    #[test]
    fn ais_vdo_parse() {
        let sentence = build_ais_sentence("AI", "VDO", "1,1,,A,13aEOK?P00PD2wVMdLDRhgvL289?,0");
        let result = parse_str(&sentence);
        assert!(result.is_ok(), "AIS VDO should parse: {:?}", result.err());
    }

    /// An AIS sentence with a bad checksum should be rejected.
    #[test]
    fn ais_bad_checksum_rejected() {
        // Build with correct checksum, then corrupt it.
        let body = "AIVDM,1,1,,A,13aEOK?P00PD2wVMdLDRhgvL289?,0";
        let cs = body.as_bytes().iter().fold(0u8, |c, x| c ^ x) ^ 0xFF;
        let sentence = format!("!{}*{:02X}", body, cs);
        let err = parse_str(&sentence).expect_err("bad AIS checksum should be rejected");
        assert!(matches!(err, Error::ChecksumMismatch { .. }));
    }

    /// The `!` prefix must be used for AIS, verify `$` also works for VDM
    /// (some receivers emit VDM with `$`).
    #[test]
    fn vdm_with_dollar_prefix() {
        let sentence = build_sentence("AI", "VDM", "1,1,,A,13aEOK?P00PD2wVMdLDRhgvL289?,0");
        assert!(
            parse_nmea_sentence(&sentence).is_ok(),
            "VDM with $ prefix should parse"
        );
    }
}

// ---------------------------------------------------------------------------
// 6. Error handling
// ---------------------------------------------------------------------------

mod error_handling {
    use super::*;

    /// Non-ASCII characters must be rejected by parse_str with an ASCII error.
    #[test]
    fn non_ascii_rejected() {
        let sentence = "$GPGGA,\u{00e9}92750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,*76";
        let err = parse_str(sentence).expect_err("non-ASCII should be rejected");
        assert_eq!(err, Error::ASCII, "expected ASCII error, got: {:?}", err);
    }

    /// Non-ASCII with a high Unicode codepoint must also be rejected.
    #[test]
    fn unicode_rejected() {
        let sentence = "$GPGGA,\u{1F600},,,,,,,,,,,,,,*00";
        let err = parse_str(sentence).expect_err("unicode emoji should be rejected");
        assert_eq!(err, Error::ASCII);
    }

    /// An unknown three-letter sentence type should produce an error
    /// from parse_nmea_sentence (the TryFrom for SentenceType fails).
    #[test]
    fn unknown_sentence_type_rejected() {
        let sentence = build_sentence("GP", "XYZ", "some,data,here");
        let err = unwrap_parse_err!(
            parse_nmea_sentence(&sentence),
            "unknown type XYZ should fail",
        );
        assert!(
            matches!(err, Error::ParsingError(_)),
            "expected ParsingError for unknown type, got: {:?}",
            err
        );
    }

    /// A sentence missing the `*` checksum delimiter should fail parsing.
    #[test]
    fn missing_checksum_delimiter() {
        let sentence = build_truncated_sentence("GP", "GGA", "092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,");
        let err = unwrap_parse_err!(
            parse_nmea_sentence(&sentence),
            "truncated sentence should fail",
        );
        assert!(
            matches!(err, Error::ParsingError(_)),
            "expected ParsingError for truncated sentence, got: {:?}",
            err
        );
    }

    /// A truncated sentence should also be rejected by parse_str.
    #[test]
    fn truncated_sentence_rejected_by_parse_str() {
        let sentence = build_truncated_sentence("GP", "RMC", "225446.33,A,4916.45,N,12311.12,W,000.5,054.7,191194,020.3,E,A");
        let err = parse_str(&sentence).expect_err("truncated sentence should fail");
        assert!(matches!(err, Error::ParsingError(_)));
    }

    /// Missing the leading `$` or `!` should fail.
    #[test]
    fn missing_prefix_rejected() {
        let err = unwrap_parse_err!(
            parse_nmea_sentence("GPGGA,092750.000,*00"),
            "missing $ prefix should fail",
        );
        assert!(matches!(err, Error::ParsingError(_)));
    }

    /// A sentence with just the header and no data/checksum should fail.
    #[test]
    fn header_only_rejected() {
        let err = unwrap_parse_err!(
            parse_nmea_sentence("$GPGGA"),
            "header only should fail",
        );
        assert!(matches!(err, Error::ParsingError(_)));
    }

    /// A completely garbage string should produce a parsing error.
    #[test]
    fn garbage_input_rejected() {
        let err = unwrap_parse_err!(
            parse_nmea_sentence("hello world"),
            "garbage should fail",
        );
        assert!(matches!(err, Error::ParsingError(_)));
    }

    /// A sentence with only `$` and talker ID but no sentence type should fail.
    #[test]
    fn incomplete_header_rejected() {
        let err = unwrap_parse_err!(
            parse_nmea_sentence("$GP"),
            "incomplete header should fail",
        );
        assert!(matches!(err, Error::ParsingError(_)));
    }

    /// Non-ASCII in parse_str is checked before parsing, so it returns ASCII error.
    #[test]
    fn non_ascii_fails_high_level() {
        let sentence_with_non_ascii = "$GPGGA,\u{00e9}*00";
        let err = parse_str(sentence_with_non_ascii).expect_err("non-ASCII should fail");
        assert_eq!(err, Error::ASCII);
    }

    /// Verify that parse_str returns the correct result for a sentence type
    /// that is recognized at the SentenceType level but not actively parsed
    /// (falls into the catch-all branch as Unsupported).
    #[test]
    fn unsupported_but_known_sentence_type() {
        // "BEC" is defined in SentenceType but not handled by parse_str's match arms
        let sentence = build_sentence("GP", "BEC", "some,data,here");
        let result = parse_str(&sentence);
        // Should either parse as Unsupported or succeed - not error
        match result {
            Ok(ParseResult::Unsupported(st)) => {
                assert_eq!(st, SentenceType::BEC);
            }
            Ok(_) => {
                // This would mean BEC got a parser, which is fine
            }
            Err(e) => {
                panic!("known sentence type BEC should not produce an error, got: {:?}", e);
            }
        }
    }

    /// Verify that a sentence with any valid checksum value parses correctly.
    #[test]
    fn checksum_always_validated() {
        let sentence = build_sentence("GP", "GGA", ",,,,,,,,,,,,,,");
        let parsed = parse_nmea_sentence(&sentence).expect("should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }

    /// A well-formed sentence without trailing characters should parse.
    #[test]
    fn clean_sentence_parses() {
        let sentence = build_sentence("GP", "HDT", "274.07,T");
        assert!(parse_nmea_sentence(&sentence).is_ok());
    }
}

// ---------------------------------------------------------------------------
// 7. Cross-cutting: build_sentence correctness
// ---------------------------------------------------------------------------

mod build_sentence_tests {
    use super::*;

    /// build_sentence must produce a sentence that parse_nmea_sentence accepts.
    #[test]
    fn build_sentence_roundtrip() {
        let sentence = build_sentence("GP", "GGA", "092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,");
        let parsed = parse_nmea_sentence(&sentence).expect("roundtrip should work");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
        assert_eq!(parsed.talker_id, "GP");
        assert_eq!(parsed.message_id, SentenceType::GGA);
    }

    /// build_sentence_bad_checksum must produce a sentence where checksums disagree.
    #[test]
    fn build_bad_checksum_produces_mismatch() {
        let sentence = build_sentence_bad_checksum("GP", "GGA", "test,data");
        let parsed = parse_nmea_sentence(&sentence).expect("low-level parse should succeed");
        assert_ne!(parsed.checksum, parsed.calc_checksum());
    }

    /// build_truncated_sentence must produce a sentence without a checksum.
    #[test]
    fn build_truncated_has_no_checksum() {
        let sentence = build_truncated_sentence("GP", "GGA", "test,data");
        assert!(!sentence.contains('*'), "truncated sentence should have no '*'");
        assert!(parse_nmea_sentence(&sentence).is_err());
    }

    /// build_ais_sentence must start with `!` and have a valid checksum.
    #[test]
    fn build_ais_sentence_format() {
        let sentence = build_ais_sentence("AI", "VDM", "1,1,,A,payload,0");
        assert!(sentence.starts_with('!'), "AIS sentence must start with '!'");
        let parsed = parse_nmea_sentence(&sentence).expect("AIS sentence should parse");
        assert_eq!(parsed.checksum, parsed.calc_checksum());
    }
}

// ---------------------------------------------------------------------------
// 8. Comprehensive sentence type parsing
// ---------------------------------------------------------------------------

mod sentence_type_parsing {
    use super::*;

    /// All major GNSS sentence types should successfully round-trip through
    /// parse_str when given valid data.
    #[test]
    fn parse_str_gga() {
        let s = build_sentence("GP", "GGA", "092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,");
        assert!(matches!(parse_str(&s), Ok(ParseResult::GGA(_))));
    }

    #[test]
    fn parse_str_rmc() {
        let s = build_sentence("GP", "RMC", "225446.33,A,4916.45,N,12311.12,W,000.5,054.7,191194,020.3,E,A");
        assert!(matches!(parse_str(&s), Ok(ParseResult::RMC(_))));
    }

    #[test]
    fn parse_str_gsa() {
        let s = build_sentence("GP", "GSA", "A,3,23,31,22,16,03,07,,,,,,,1.8,1.1,1.4");
        assert!(matches!(parse_str(&s), Ok(ParseResult::GSA(_))));
    }

    #[test]
    fn parse_str_gsv() {
        let s = build_sentence("GP", "GSV", "3,1,12,01,49,196,41,03,71,278,32,06,02,323,27,11,21,196,39");
        assert!(matches!(parse_str(&s), Ok(ParseResult::GSV(_))));
    }

    #[test]
    fn parse_str_gll() {
        let s = build_sentence("GP", "GLL", "5107.0013414,N,11402.3279144,W,205412.00,A,A");
        assert!(matches!(parse_str(&s), Ok(ParseResult::GLL(_))));
    }

    #[test]
    fn parse_str_vtg() {
        let s = build_sentence("GP", "VTG", "360.0,T,348.7,M,000.0,N,000.0,K");
        assert!(matches!(parse_str(&s), Ok(ParseResult::VTG(_))));
    }

    #[test]
    fn parse_str_hdt() {
        let s = build_sentence("GP", "HDT", "274.07,T");
        assert!(matches!(parse_str(&s), Ok(ParseResult::HDT(_))));
    }

    #[test]
    fn parse_str_zda() {
        let s = build_sentence("GP", "ZDA", "160012.71,11,03,2004,-1,00");
        assert!(matches!(parse_str(&s), Ok(ParseResult::ZDA(_))));
    }

    #[test]
    fn parse_str_txt() {
        let s = build_sentence("GN", "TXT", "01,01,02,u-blox AG - www.u-blox.com");
        assert!(matches!(parse_str(&s), Ok(ParseResult::TXT(_))));
    }

    #[test]
    fn parse_str_dbt() {
        let s = build_sentence("SD", "DBT", "12.3,f,3.75,M,2.05,F");
        assert!(matches!(parse_str(&s), Ok(ParseResult::DBT(_))));
    }

    #[test]
    fn parse_str_mwv() {
        let s = build_sentence("WI", "MWV", "041.1,R,01.0,N,A");
        assert!(matches!(parse_str(&s), Ok(ParseResult::MWV(_))));
    }

    #[test]
    fn parse_str_gbs() {
        let s = build_sentence("GP", "GBS", "182141.000,15.5,15.3,7.2,21,0.9,0.5,0.8");
        assert!(matches!(parse_str(&s), Ok(ParseResult::GBS(_))));
    }

    #[test]
    fn parse_str_gst() {
        let s = build_sentence("GP", "GST", "182141.000,15.5,15.3,7.2,21.8,0.9,0.5,0.8");
        assert!(matches!(parse_str(&s), Ok(ParseResult::GST(_))));
    }

    #[test]
    fn parse_str_gns() {
        let s = build_sentence("GP", "GNS", "224749.00,3333.4268304,N,11153.3538273,W,D,19,0.6,406.110,-26.294,6.0,0138,S,");
        assert!(matches!(parse_str(&s), Ok(ParseResult::GNS(_))));
    }
}
