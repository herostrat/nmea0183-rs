//! Comprehensive serialization integration tests for the NMEA 0183 crate.
//!
//! Tests the `GenerateNmeaBody` trait and `generate_sentence` function across
//! all sentence types that support generation. Validates:
//!
//! 1. Format correctness: Generated sentences start with `$`, end with `*HH` checksum
//! 2. Checksum validation: All generated sentences have valid checksums
//! 3. Roundtrip fidelity: Generate -> Parse -> Compare for all supported types

use nmea::generate::{GenerateNmeaBody, generate_sentence};
use nmea::{parse_nmea_sentence, parse_str, ParseResult, SentenceType};

// ============================================================================
// Helper functions
// ============================================================================

/// Validate that a generated NMEA sentence has correct structure:
/// - Starts with `$` (or `!` for AIS)
/// - Contains `*` followed by two hex checksum digits
/// - Checksum is valid
fn assert_valid_sentence_format(sentence: &str) {
    assert!(
        sentence.starts_with('$') || sentence.starts_with('!'),
        "Sentence must start with '$' or '!': {}",
        sentence,
    );

    let star_pos = sentence
        .rfind('*')
        .unwrap_or_else(|| panic!("Sentence must contain '*': {}", sentence));

    let checksum_str = &sentence[star_pos + 1..];
    assert_eq!(
        checksum_str.len(),
        2,
        "Checksum must be exactly 2 hex digits, got '{}' in: {}",
        checksum_str,
        sentence,
    );
    assert!(
        checksum_str.chars().all(|c| c.is_ascii_hexdigit()),
        "Checksum must be hex digits, got '{}' in: {}",
        checksum_str,
        sentence,
    );

    // Verify the checksum is correct by parsing
    let parsed = parse_nmea_sentence(sentence)
        .unwrap_or_else(|e| panic!("Failed to parse generated sentence '{}': {:?}", sentence, e));
    assert_eq!(
        parsed.checksum,
        parsed.calc_checksum(),
        "Checksum mismatch for generated sentence: {}",
        sentence,
    );
}

/// Generate a sentence from a data struct, validate format, and return the string.
fn generate_and_validate(talker_id: &str, data: &dyn GenerateNmeaBody) -> String {
    let mut buf = String::new();
    generate_sentence(talker_id, data, &mut buf)
        .unwrap_or_else(|e| panic!("Failed to generate sentence: {:?}", e));
    assert_valid_sentence_format(&buf);
    buf
}

/// Parse a known-good NMEA sentence string, extract data, generate a new sentence,
/// parse the generated sentence, and return both ParseResults for comparison.
fn roundtrip_sentence(sentence: &str) -> (ParseResult, ParseResult) {
    let original = parse_str(sentence)
        .unwrap_or_else(|e| panic!("Failed to parse original sentence '{}': {:?}", sentence, e));

    // Extract talker_id from the original sentence
    let nmea_sentence = parse_nmea_sentence(sentence)
        .unwrap_or_else(|e| panic!("Failed to parse NMEA sentence '{}': {:?}", sentence, e));
    let talker_id = nmea_sentence.talker_id;

    // Generate from parsed data
    let generated = generate_from_parse_result(talker_id, &original);
    assert_valid_sentence_format(&generated);

    // Parse the generated sentence
    let reparsed = parse_str(&generated).unwrap_or_else(|e| {
        panic!(
            "Failed to parse generated sentence '{}' (original: '{}'): {:?}",
            generated, sentence, e
        )
    });

    (original, reparsed)
}

/// Generate a sentence string from a ParseResult variant.
fn generate_from_parse_result(talker_id: &str, result: &ParseResult) -> String {
    let mut buf = String::new();
    match result {
        ParseResult::AAM(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::ALM(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::APA(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::APB(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::BOD(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::BWC(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::BWW(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::DBK(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::DBS(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::DBT(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::DPT(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::DSC(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::GBS(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::GGA(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::GLL(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::GNS(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::GSA(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::GST(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::GSV(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::HDG(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::HDM(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::HDT(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::HSC(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::MDA(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::MTW(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::MWD(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::MWV(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::RMB(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::RMC(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::ROT(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::RPM(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::RSA(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::STALK(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::TTM(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::TXT(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::VDM(data) | ParseResult::VDO(data) => {
            // VDM uses '!' prefix, but generate_sentence uses '$'.
            // We generate with the standard API and the sentence type handles VDM/VDO.
            generate_sentence(talker_id, data, &mut buf).unwrap();
        }
        ParseResult::VDR(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::VHW(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::VLW(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::VPW(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::VTG(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::VWR(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::VWT(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::XDR(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::XTE(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::WNC(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::ZDA(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::ZFO(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::ZTG(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::PGRMZ(data) => generate_sentence(talker_id, data, &mut buf).unwrap(),
        ParseResult::Unsupported(_) => panic!("Cannot generate unsupported sentence type"),
    };
    buf
}

// ============================================================================
// Test 1: Format correctness for all types
// ============================================================================

/// Test that generate_sentence produces structurally correct output for every
/// sentence type that can be parsed from known-good test data.
#[test]
fn test_all_generated_sentences_have_valid_format() {
    // These are known-good sentences from the crate's own test suite.
    // We skip VDM because it uses '!' prefix (AIS sentences) which generate_sentence
    // produces with '$' instead.
    let sentences = [
        "$GPAAM,A,A,0.10,N,WPTNME*32",
        "$GPALM,1,1,15,1159,00,441D,4E,16BE,FD5E,A10C9F,4A2DA4,686E81,58CBE1,0A4,001*77",
        "$GPAPA,A,A,0.10,R,N,V,V,011,M,DEST,011,M*42",
        "$GPAPB,A,A,0.10,R,N,V,V,011,M,DEST,011,M,011,M*3C",
        "$GPBOD,045.,T,023.,M,DEST,START*01",
        "$GPBWC,220516,5130.02,N,00046.34,W,213.8,T,218.0,M,0004.6,N,EGLM*21",
        "$GPBWW,213.8,T,218.0,M,TOWPT,FROMWPT*42",
        "$GPGBS,182141.000,15.5,15.3,7.2,21,0.9,0.5,0.8*54",
        "$GPGGA,133605.0,5521.75946,N,03731.93769,E,0,00,,,M,,M,,*4F",
        "$GPGLL,5107.0013414,N,11402.3279144,W,205412.00,A,A*73",
        "$GPGNS,224749.00,3333.4268304,N,11153.3538273,W,D,19,0.6,406.110,-26.294,6.0,0138,S,*46",
        "$GPGSA,A,3,23,31,22,16,03,07,,,,,,,1.8,1.1,1.4*3E",
        "$GPGST,182141.000,15.5,15.3,7.2,21.8,0.9,0.5,0.8*54",
        "$GPGSV,3,1,12,01,49,196,41,03,71,278,32,06,02,323,27,11,21,196,39*72",
        "$GPHDT,274.07,T*03",
        "$WIMDA,29.7544,I,1.0076,B,35.5,C,,,42.1,,20.6,C,116.4,T,107.7,M,1.2,N,0.6,M*66",
        "$INMTW,17.9,C*1B",
        "$WIMWV,041.1,R,01.0,N,A*16",
        "$ECRMB,A,0.000,L,001,002,4653.550,N,07115.984,W,2.505,334.205,0.000,V*04",
        "$GPRMC,225446.33,A,4916.45,N,12311.12,W,000.5,054.7,191194,020.3,E,A*2B",
        "$PGRMZ,2282,f,3*21",
        "$RATTM,01,0.2,190.8,T,12.1,109.7,T,0.1,0.5,N,TGT01,T,,100021.00,A*79",
        "$GNTXT,01,01,02,u-blox AG - www.u-blox.com*4E",
        "$GPVHW,100.5,T,105.5,M,10.5,N,19.4,K*4F",
        "$GPVTG,360.0,T,348.7,M,000.0,N,000.0,K*43",
        "$IIVWR,75,R,1.0,N,0.51,M,1.85,K*6C",
        "$IIVWT,030.,R,10.1,N,05.2,M,018.7,K*75",
        "$GPWNC,200.00,N,370.40,K,Dest,Origin*58",
        "$GPZDA,160012.71,11,03,2004,-1,00*7D",
        "$GPZFO,145832.12,042359.17,WPT*3E",
        "$GPZTG,145832.12,042359.17,WPT*24",
        "$SDDBK,1330.5,f,0405.5,M,0221.6,F*2E",
        "$SDDBS,12.3,f,3.75,M,2.05,F*37",
        "$SDDBT,12.3,f,3.75,M,2.05,F*30",
        "$SDDPT,17.9,0.5*6D",
        "$HCHDG,98.3,0.0,E,12.6,W*57",
        "$HCHDM,238.9,M*29",
        "$GPHSC,128.5,T,135.2,M*5D",
        "$WIMWD,270.0,T,265.5,M,10.2,N,5.3,M*6E",
        "$TIROT,-0.3,A*15",
        "$IIRPM,E,1,2418.2,10.5,A*5F",
        "$IIRSA,10.5,A,,V*4D",
        "$IIVDR,180.0,T,175.5,M,1.5,N*32",
        "$VWVLW,7803.2,N,0.00,N*42",
        "$IIVPW,4.5,N,2.3,M*52",
        "$GPXTE,A,A,0.67,L,N*6F",
    ];

    for sentence in &sentences {
        let parsed = parse_str(sentence)
            .unwrap_or_else(|e| panic!("Failed to parse '{}': {:?}", sentence, e));

        let nmea = parse_nmea_sentence(sentence).unwrap();
        let generated = generate_from_parse_result(nmea.talker_id, &parsed);

        assert!(
            generated.starts_with('$'),
            "Generated sentence should start with '$': {} -> {}",
            sentence,
            generated,
        );

        assert!(
            generated.contains('*'),
            "Generated sentence should contain '*': {} -> {}",
            sentence,
            generated,
        );

        // Verify checksum is valid
        let gen_parsed = parse_nmea_sentence(&generated).unwrap_or_else(|e| {
            panic!(
                "Failed to parse generated sentence '{}' (from '{}'): {:?}",
                generated, sentence, e,
            )
        });
        assert_eq!(
            gen_parsed.checksum,
            gen_parsed.calc_checksum(),
            "Checksum mismatch for generated sentence: {} (from {})",
            generated,
            sentence,
        );
    }
}

// ============================================================================
// Test 2: Checksum validation
// ============================================================================

#[test]
fn test_checksum_computation_matches_after_generation() {
    use nmea::sentences::HdtData;

    // Test with a simple type that we can construct directly.
    let data = HdtData {
        heading: Some(274.07),
    };
    let generated = generate_and_validate("GP", &data);

    // The checksum in the generated sentence should match what we compute.
    let parsed = parse_nmea_sentence(&generated).unwrap();
    assert_eq!(parsed.checksum, parsed.calc_checksum());

    // Test with None heading
    let data_none = HdtData { heading: None };
    let generated_none = generate_and_validate("GP", &data_none);
    let parsed_none = parse_nmea_sentence(&generated_none).unwrap();
    assert_eq!(parsed_none.checksum, parsed_none.calc_checksum());
}

// ============================================================================
// Test 3: Roundtrip tests for individual sentence types
// ============================================================================

// --- Simple heading/temperature types ---

#[test]
fn test_roundtrip_hdt() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$GPHDT,274.07,T*03");
    if let (ParseResult::HDT(o), ParseResult::HDT(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.heading.unwrap(), r.heading.unwrap());
    } else {
        panic!("Expected HDT, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_hdm() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$HCHDM,238.9,M*29");
    if let (ParseResult::HDM(o), ParseResult::HDM(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.heading.unwrap(), r.heading.unwrap());
    } else {
        panic!("Expected HDM, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_mtw() {
    let (orig, reparsed) = roundtrip_sentence("$INMTW,17.9,C*1B");
    if let (ParseResult::MTW(o), ParseResult::MTW(r)) = (&orig, &reparsed) {
        assert_eq!(o.temperature, r.temperature);
    } else {
        panic!("Expected MTW, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_rot() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$TIROT,-0.3,A*15");
    if let (ParseResult::ROT(o), ParseResult::ROT(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.rate.unwrap(), r.rate.unwrap());
        assert_eq!(o.valid, r.valid);
    } else {
        panic!("Expected ROT, got {:?} and {:?}", orig, reparsed);
    }
}

// --- Depth types ---

#[test]
fn test_roundtrip_dbk() {
    let (orig, reparsed) = roundtrip_sentence("$SDDBK,1330.5,f,0405.5,M,0221.6,F*2E");
    if let (ParseResult::DBK(o), ParseResult::DBK(r)) = (&orig, &reparsed) {
        assert_eq!(o.depth_feet, r.depth_feet);
        assert_eq!(o.depth_meters, r.depth_meters);
        assert_eq!(o.depth_fathoms, r.depth_fathoms);
    } else {
        panic!("Expected DBK, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_dbs() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$SDDBS,12.3,f,3.75,M,2.05,F*37");
    if let (ParseResult::DBS(o), ParseResult::DBS(r)) = (&orig, &reparsed) {
        assert_relative_eq!(
            o.water_depth_feet.unwrap(),
            r.water_depth_feet.unwrap()
        );
        assert_relative_eq!(
            o.water_depth_meters.unwrap(),
            r.water_depth_meters.unwrap()
        );
        assert_relative_eq!(
            o.water_depth_fathoms.unwrap(),
            r.water_depth_fathoms.unwrap()
        );
    } else {
        panic!("Expected DBS, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_dbt() {
    let (orig, reparsed) = roundtrip_sentence("$SDDBT,12.3,f,3.75,M,2.05,F*30");
    if let (ParseResult::DBT(o), ParseResult::DBT(r)) = (&orig, &reparsed) {
        assert_eq!(o.depth_feet, r.depth_feet);
        assert_eq!(o.depth_meters, r.depth_meters);
        assert_eq!(o.depth_fathoms, r.depth_fathoms);
    } else {
        panic!("Expected DBT, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_dpt() {
    let (orig, reparsed) = roundtrip_sentence("$SDDPT,17.9,0.5*6D");
    if let (ParseResult::DPT(o), ParseResult::DPT(r)) = (&orig, &reparsed) {
        assert_eq!(o.water_depth, r.water_depth);
        assert_eq!(o.offset, r.offset);
    } else {
        panic!("Expected DPT, got {:?} and {:?}", orig, reparsed);
    }
}

// --- Navigation / GNSS types ---

#[test]
fn test_roundtrip_gga() {
    use approx::assert_relative_eq;

    let (orig, reparsed) =
        roundtrip_sentence("$GPGGA,133605.0,5521.75946,N,03731.93769,E,0,00,,,M,,M,,*4F");
    if let (ParseResult::GGA(o), ParseResult::GGA(r)) = (&orig, &reparsed) {
        assert_eq!(o.fix_time, r.fix_time);
        assert_eq!(o.fix_type, r.fix_type);
        // Latitude and longitude may have small rounding differences due to
        // decimal-degree <-> DDmm.mm conversion
        match (o.latitude, r.latitude) {
            (Some(ol), Some(rl)) => assert_relative_eq!(ol, rl, epsilon = 1e-4),
            (None, None) => {}
            _ => panic!(
                "Latitude mismatch: {:?} vs {:?}",
                o.latitude, r.latitude
            ),
        }
        match (o.longitude, r.longitude) {
            (Some(ol), Some(rl)) => assert_relative_eq!(ol, rl, epsilon = 1e-4),
            (None, None) => {}
            _ => panic!(
                "Longitude mismatch: {:?} vs {:?}",
                o.longitude, r.longitude
            ),
        }
    } else {
        panic!("Expected GGA, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_rmc() {
    use approx::assert_relative_eq;

    let (orig, reparsed) =
        roundtrip_sentence("$GPRMC,225446.33,A,4916.45,N,12311.12,W,000.5,054.7,191194,020.3,E,A*2B");
    if let (ParseResult::RMC(o), ParseResult::RMC(r)) = (&orig, &reparsed) {
        assert_eq!(o.fix_time, r.fix_time);
        assert_eq!(o.fix_date, r.fix_date);
        assert_eq!(o.status_of_fix, r.status_of_fix);
        match (o.lat, r.lat) {
            (Some(ol), Some(rl)) => assert_relative_eq!(ol, rl, epsilon = 1e-4),
            (None, None) => {}
            _ => panic!("Lat mismatch"),
        }
        match (o.lon, r.lon) {
            (Some(ol), Some(rl)) => assert_relative_eq!(ol, rl, epsilon = 1e-4),
            (None, None) => {}
            _ => panic!("Lon mismatch"),
        }
        match (o.speed_over_ground, r.speed_over_ground) {
            (Some(os), Some(rs)) => assert_relative_eq!(os, rs),
            (None, None) => {}
            _ => panic!("Speed mismatch"),
        }
        assert_eq!(o.faa_mode, r.faa_mode);
    } else {
        panic!("Expected RMC, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_gll() {
    use approx::assert_relative_eq;

    let (orig, reparsed) =
        roundtrip_sentence("$GPGLL,5107.0013414,N,11402.3279144,W,205412.00,A,A*73");
    if let (ParseResult::GLL(o), ParseResult::GLL(r)) = (&orig, &reparsed) {
        match (o.latitude, r.latitude) {
            (Some(ol), Some(rl)) => assert_relative_eq!(ol, rl, epsilon = 1e-4),
            (None, None) => {}
            _ => panic!("Latitude mismatch"),
        }
        assert_eq!(o.fix_time, r.fix_time);
        assert_eq!(o.valid, r.valid);
        assert_eq!(o.faa_mode, r.faa_mode);
    } else {
        panic!("Expected GLL, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_vtg() {
    use approx::assert_relative_eq;

    let (orig, reparsed) =
        roundtrip_sentence("$GPVTG,360.0,T,348.7,M,000.0,N,000.0,K*43");
    if let (ParseResult::VTG(o), ParseResult::VTG(r)) = (&orig, &reparsed) {
        assert_eq!(o.true_course, r.true_course);
        match (o.speed_over_ground, r.speed_over_ground) {
            (Some(os), Some(rs)) => assert_relative_eq!(os, rs, epsilon = 0.01),
            (None, None) => {}
            _ => panic!("Speed mismatch"),
        }
    } else {
        panic!("Expected VTG, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_gsa() {
    let (orig, reparsed) =
        roundtrip_sentence("$GPGSA,A,3,23,31,22,16,03,07,,,,,,,1.8,1.1,1.4*3E");
    if let (ParseResult::GSA(o), ParseResult::GSA(r)) = (&orig, &reparsed) {
        assert_eq!(o.mode1, r.mode1);
        assert_eq!(o.mode2, r.mode2);
        assert_eq!(o.fix_sats_prn, r.fix_sats_prn);
        // DOP values may have small rounding differences
        assert_eq!(o.pdop, r.pdop);
        assert_eq!(o.hdop, r.hdop);
        assert_eq!(o.vdop, r.vdop);
    } else {
        panic!("Expected GSA, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_gst() {
    use approx::assert_relative_eq;

    let (orig, reparsed) =
        roundtrip_sentence("$GPGST,182141.000,15.5,15.3,7.2,21.8,0.9,0.5,0.8*54");
    if let (ParseResult::GST(o), ParseResult::GST(r)) = (&orig, &reparsed) {
        assert_eq!(o.time, r.time);
        match (o.rms_sd, r.rms_sd) {
            (Some(ov), Some(rv)) => assert_relative_eq!(ov, rv),
            (None, None) => {}
            _ => panic!("rms_sd mismatch"),
        }
    } else {
        panic!("Expected GST, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_gbs() {
    use approx::assert_relative_eq;

    let (orig, reparsed) =
        roundtrip_sentence("$GPGBS,182141.000,15.5,15.3,7.2,21,0.9,0.5,0.8*54");
    if let (ParseResult::GBS(o), ParseResult::GBS(r)) = (&orig, &reparsed) {
        assert_eq!(o.time, r.time);
        match (o.lat_error, r.lat_error) {
            (Some(ov), Some(rv)) => assert_relative_eq!(ov, rv),
            (None, None) => {}
            _ => panic!("lat_error mismatch"),
        }
    } else {
        panic!("Expected GBS, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_gns() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence(
        "$GPGNS,224749.00,3333.4268304,N,11153.3538273,W,D,19,0.6,406.110,-26.294,6.0,0138,S,*46",
    );
    if let (ParseResult::GNS(o), ParseResult::GNS(r)) = (&orig, &reparsed) {
        assert_eq!(o.fix_time, r.fix_time);
        match (o.lat, r.lat) {
            (Some(ol), Some(rl)) => assert_relative_eq!(ol, rl, epsilon = 1e-4),
            (None, None) => {}
            _ => panic!("Lat mismatch"),
        }
    } else {
        panic!("Expected GNS, got {:?} and {:?}", orig, reparsed);
    }
}

// --- Heading types ---

#[test]
fn test_roundtrip_hdg() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$HCHDG,98.3,0.0,E,12.6,W*57");
    if let (ParseResult::HDG(o), ParseResult::HDG(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.heading.unwrap(), r.heading.unwrap());
        assert_relative_eq!(o.deviation.unwrap(), r.deviation.unwrap());
        assert_relative_eq!(o.variation.unwrap(), r.variation.unwrap());
    } else {
        panic!("Expected HDG, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_hsc() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$GPHSC,128.5,T,135.2,M*5D");
    if let (ParseResult::HSC(o), ParseResult::HSC(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.heading_true.unwrap(), r.heading_true.unwrap());
        assert_relative_eq!(o.heading_magnetic.unwrap(), r.heading_magnetic.unwrap());
    } else {
        panic!("Expected HSC, got {:?} and {:?}", orig, reparsed);
    }
}

// --- Water types ---

#[test]
fn test_roundtrip_vhw() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$GPVHW,100.5,T,105.5,M,10.5,N,19.4,K*4F");
    if let (ParseResult::VHW(o), ParseResult::VHW(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.heading_true.unwrap(), r.heading_true.unwrap());
        assert_relative_eq!(o.heading_magnetic.unwrap(), r.heading_magnetic.unwrap());
        assert_relative_eq!(
            o.relative_speed_knots.unwrap(),
            r.relative_speed_knots.unwrap()
        );
        assert_relative_eq!(
            o.relative_speed_kmph.unwrap(),
            r.relative_speed_kmph.unwrap()
        );
    } else {
        panic!("Expected VHW, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_vlw() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$VWVLW,7803.2,N,0.00,N*42");
    if let (ParseResult::VLW(o), ParseResult::VLW(r)) = (&orig, &reparsed) {
        assert_relative_eq!(
            o.total_water_distance.unwrap(),
            r.total_water_distance.unwrap()
        );
        assert_relative_eq!(
            o.trip_water_distance.unwrap(),
            r.trip_water_distance.unwrap()
        );
    } else {
        panic!("Expected VLW, got {:?} and {:?}", orig, reparsed);
    }
}

// --- Wind types ---

#[test]
fn test_roundtrip_mwv() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$WIMWV,041.1,R,01.0,N,A*16");
    if let (ParseResult::MWV(o), ParseResult::MWV(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.wind_direction.unwrap(), r.wind_direction.unwrap());
        assert_eq!(o.reference, r.reference);
        assert_relative_eq!(o.wind_speed.unwrap(), r.wind_speed.unwrap());
        assert_eq!(o.wind_speed_units, r.wind_speed_units);
        assert_eq!(o.data_valid, r.data_valid);
    } else {
        panic!("Expected MWV, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_mwd() {
    use approx::assert_relative_eq;

    let (orig, reparsed) =
        roundtrip_sentence("$WIMWD,270.0,T,265.5,M,10.2,N,5.3,M*6E");
    if let (ParseResult::MWD(o), ParseResult::MWD(r)) = (&orig, &reparsed) {
        assert_relative_eq!(
            o.wind_direction_true.unwrap(),
            r.wind_direction_true.unwrap()
        );
        assert_relative_eq!(
            o.wind_direction_magnetic.unwrap(),
            r.wind_direction_magnetic.unwrap()
        );
        assert_relative_eq!(o.wind_speed_knots.unwrap(), r.wind_speed_knots.unwrap());
        assert_relative_eq!(o.wind_speed_mps.unwrap(), r.wind_speed_mps.unwrap());
    } else {
        panic!("Expected MWD, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_vwr() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$IIVWR,75,R,1.0,N,0.51,M,1.85,K*6C");
    if let (ParseResult::VWR(o), ParseResult::VWR(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.wind_angle.unwrap(), r.wind_angle.unwrap());
        assert_relative_eq!(o.speed_knots.unwrap(), r.speed_knots.unwrap());
    } else {
        panic!("Expected VWR, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_vwt() {
    use approx::assert_relative_eq;

    let (orig, reparsed) =
        roundtrip_sentence("$IIVWT,030.,R,10.1,N,05.2,M,018.7,K*75");
    if let (ParseResult::VWT(o), ParseResult::VWT(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.wind_angle.unwrap(), r.wind_angle.unwrap());
        assert_relative_eq!(o.speed_knots.unwrap(), r.speed_knots.unwrap());
        assert_relative_eq!(o.speed_mps.unwrap(), r.speed_mps.unwrap());
        assert_relative_eq!(o.speed_kmph.unwrap(), r.speed_kmph.unwrap());
    } else {
        panic!("Expected VWT, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_vpw() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$IIVPW,4.5,N,2.3,M*52");
    if let (ParseResult::VPW(o), ParseResult::VPW(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.speed_knots.unwrap(), r.speed_knots.unwrap());
        assert_relative_eq!(o.speed_mps.unwrap(), r.speed_mps.unwrap());
    } else {
        panic!("Expected VPW, got {:?} and {:?}", orig, reparsed);
    }
}

// --- Waypoint types ---

#[test]
fn test_roundtrip_aam() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$GPAAM,A,A,0.10,N,WPTNME*32");
    if let (ParseResult::AAM(o), ParseResult::AAM(r)) = (&orig, &reparsed) {
        assert_eq!(o.arrival_circle_entered, r.arrival_circle_entered);
        assert_eq!(o.perpendicular_passed, r.perpendicular_passed);
        assert_relative_eq!(
            o.arrival_circle_radius.unwrap(),
            r.arrival_circle_radius.unwrap()
        );
        assert_eq!(o.radius_units, r.radius_units);
        assert_eq!(o.waypoint_id, r.waypoint_id);
    } else {
        panic!("Expected AAM, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_bod() {
    use approx::assert_relative_eq;

    let (orig, reparsed) =
        roundtrip_sentence("$GPBOD,045.,T,023.,M,DEST,START*01");
    if let (ParseResult::BOD(o), ParseResult::BOD(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.bearing_true.unwrap(), r.bearing_true.unwrap());
        assert_relative_eq!(o.bearing_magnetic.unwrap(), r.bearing_magnetic.unwrap());
        assert_eq!(o.to_waypoint, r.to_waypoint);
        assert_eq!(o.from_waypoint, r.from_waypoint);
    } else {
        panic!("Expected BOD, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_bwc() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence(
        "$GPBWC,220516,5130.02,N,00046.34,W,213.8,T,218.0,M,0004.6,N,EGLM*21",
    );
    if let (ParseResult::BWC(o), ParseResult::BWC(r)) = (&orig, &reparsed) {
        assert_eq!(o.fix_time, r.fix_time);
        match (o.latitude, r.latitude) {
            (Some(ol), Some(rl)) => assert_relative_eq!(ol, rl, epsilon = 1e-4),
            (None, None) => {}
            _ => panic!("Lat mismatch"),
        }
        assert_relative_eq!(o.true_bearing.unwrap(), r.true_bearing.unwrap());
        assert_relative_eq!(o.magnetic_bearing.unwrap(), r.magnetic_bearing.unwrap());
        assert_relative_eq!(o.distance.unwrap(), r.distance.unwrap());
        assert_eq!(o.waypoint_id, r.waypoint_id);
    } else {
        panic!("Expected BWC, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_bww() {
    use approx::assert_relative_eq;

    let (orig, reparsed) =
        roundtrip_sentence("$GPBWW,213.8,T,218.0,M,TOWPT,FROMWPT*42");
    if let (ParseResult::BWW(o), ParseResult::BWW(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.true_bearing.unwrap(), r.true_bearing.unwrap());
        assert_relative_eq!(o.magnetic_bearing.unwrap(), r.magnetic_bearing.unwrap());
        assert_eq!(o.to_waypoint_id, r.to_waypoint_id);
        assert_eq!(o.from_waypoint_id, r.from_waypoint_id);
    } else {
        panic!("Expected BWW, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_wnc() {
    use approx::assert_relative_eq;

    let (orig, reparsed) =
        roundtrip_sentence("$GPWNC,200.00,N,370.40,K,Dest,Origin*58");
    if let (ParseResult::WNC(o), ParseResult::WNC(r)) = (&orig, &reparsed) {
        assert_relative_eq!(
            o.distance_nautical_miles.unwrap(),
            r.distance_nautical_miles.unwrap()
        );
        assert_relative_eq!(
            o.distance_kilometers.unwrap(),
            r.distance_kilometers.unwrap()
        );
        assert_eq!(o.waypoint_id_destination, r.waypoint_id_destination);
        assert_eq!(o.waypoint_id_origin, r.waypoint_id_origin);
    } else {
        panic!("Expected WNC, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_zfo() {
    let (orig, reparsed) = roundtrip_sentence("$GPZFO,145832.12,042359.17,WPT*3E");
    if let (ParseResult::ZFO(o), ParseResult::ZFO(r)) = (&orig, &reparsed) {
        assert_eq!(o.fix_time, r.fix_time);
        assert_eq!(o.fix_duration, r.fix_duration);
        assert_eq!(o.waypoint_id, r.waypoint_id);
    } else {
        panic!("Expected ZFO, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_ztg() {
    let (orig, reparsed) = roundtrip_sentence("$GPZTG,145832.12,042359.17,WPT*24");
    if let (ParseResult::ZTG(o), ParseResult::ZTG(r)) = (&orig, &reparsed) {
        assert_eq!(o.fix_time, r.fix_time);
        assert_eq!(o.fix_duration, r.fix_duration);
        assert_eq!(o.waypoint_id, r.waypoint_id);
    } else {
        panic!("Expected ZTG, got {:?} and {:?}", orig, reparsed);
    }
}

// --- Navigation types ---

#[test]
fn test_roundtrip_xte() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$GPXTE,A,A,0.67,L,N*6F");
    if let (ParseResult::XTE(o), ParseResult::XTE(r)) = (&orig, &reparsed) {
        assert_relative_eq!(
            o.cross_track_error.unwrap(),
            r.cross_track_error.unwrap()
        );
        assert_eq!(o.status_general, r.status_general);
        assert_eq!(o.status_cycle_lock, r.status_cycle_lock);
    } else {
        panic!("Expected XTE, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_rmb() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence(
        "$ECRMB,A,0.000,L,001,002,4653.550,N,07115.984,W,2.505,334.205,0.000,V*04",
    );
    if let (ParseResult::RMB(o), ParseResult::RMB(r)) = (&orig, &reparsed) {
        assert_eq!(o.status, r.status);
        match (o.cross_track_error, r.cross_track_error) {
            (Some(ov), Some(rv)) => assert_relative_eq!(ov, rv, epsilon = 0.01),
            (None, None) => {}
            _ => panic!("Cross track error mismatch"),
        }
        assert_eq!(o.origin_waypoint_id, r.origin_waypoint_id);
        assert_eq!(o.dest_waypoint_id, r.dest_waypoint_id);
    } else {
        panic!("Expected RMB, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_apa() {
    let (orig, reparsed) =
        roundtrip_sentence("$GPAPA,A,A,0.10,R,N,V,V,011,M,DEST,011,M*42");
    if let (ParseResult::APA(o), ParseResult::APA(r)) = (&orig, &reparsed) {
        assert_eq!(o.status_warning, r.status_warning);
        assert_eq!(o.status_cycle_warning, r.status_cycle_warning);
        assert_eq!(o.waypoint_id, r.waypoint_id);
    } else {
        panic!("Expected APA, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_apb() {
    let (orig, reparsed) =
        roundtrip_sentence("$GPAPB,A,A,0.10,R,N,V,V,011,M,DEST,011,M,011,M*3C");
    if let (ParseResult::APB(o), ParseResult::APB(r)) = (&orig, &reparsed) {
        assert_eq!(o.status_warning, r.status_warning);
        assert_eq!(o.status_cycle_lock, r.status_cycle_lock);
        assert_eq!(o.waypoint_id, r.waypoint_id);
    } else {
        panic!("Expected APB, got {:?} and {:?}", orig, reparsed);
    }
}

// --- Equipment types ---

#[test]
fn test_roundtrip_rpm() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$IIRPM,E,1,2418.2,10.5,A*5F");
    if let (ParseResult::RPM(o), ParseResult::RPM(r)) = (&orig, &reparsed) {
        assert_eq!(o.source, r.source);
        assert_eq!(o.source_number, r.source_number);
        assert_relative_eq!(o.rpm.unwrap(), r.rpm.unwrap());
        assert_relative_eq!(o.pitch.unwrap(), r.pitch.unwrap());
        assert_eq!(o.valid, r.valid);
    } else {
        panic!("Expected RPM, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_rsa() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$IIRSA,10.5,A,,V*4D");
    if let (ParseResult::RSA(o), ParseResult::RSA(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.starboard.unwrap(), r.starboard.unwrap());
        assert_eq!(o.starboard_valid, r.starboard_valid);
        assert_eq!(o.port, r.port);
        assert_eq!(o.port_valid, r.port_valid);
    } else {
        panic!("Expected RSA, got {:?} and {:?}", orig, reparsed);
    }
}

// --- Current types ---

#[test]
fn test_roundtrip_vdr() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence("$IIVDR,180.0,T,175.5,M,1.5,N*32");
    if let (ParseResult::VDR(o), ParseResult::VDR(r)) = (&orig, &reparsed) {
        assert_relative_eq!(o.direction_true.unwrap(), r.direction_true.unwrap());
        assert_relative_eq!(
            o.direction_magnetic.unwrap(),
            r.direction_magnetic.unwrap()
        );
        assert_relative_eq!(o.speed.unwrap(), r.speed.unwrap());
    } else {
        panic!("Expected VDR, got {:?} and {:?}", orig, reparsed);
    }
}

// --- Other types ---

#[test]
fn test_roundtrip_zda() {
    let (orig, reparsed) = roundtrip_sentence("$GPZDA,160012.71,11,03,2004,-1,00*7D");
    if let (ParseResult::ZDA(o), ParseResult::ZDA(r)) = (&orig, &reparsed) {
        assert_eq!(o.utc_time, r.utc_time);
        assert_eq!(o.day, r.day);
        assert_eq!(o.month, r.month);
        assert_eq!(o.year, r.year);
        assert_eq!(o.local_zone_hours, r.local_zone_hours);
        assert_eq!(o.local_zone_minutes, r.local_zone_minutes);
    } else {
        panic!("Expected ZDA, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_txt() {
    let (orig, reparsed) =
        roundtrip_sentence("$GNTXT,01,01,02,u-blox AG - www.u-blox.com*4E");
    if let (ParseResult::TXT(o), ParseResult::TXT(r)) = (&orig, &reparsed) {
        assert_eq!(o.count, r.count);
        assert_eq!(o.seq, r.seq);
        assert_eq!(o.text_ident, r.text_ident);
        assert_eq!(o.text, r.text);
    } else {
        panic!("Expected TXT, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_mda() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence(
        "$WIMDA,29.7544,I,1.0076,B,35.5,C,,,42.1,,20.6,C,116.4,T,107.7,M,1.2,N,0.6,M*66",
    );
    if let (ParseResult::MDA(o), ParseResult::MDA(r)) = (&orig, &reparsed) {
        match (o.pressure_in_hg, r.pressure_in_hg) {
            (Some(ov), Some(rv)) => assert_relative_eq!(ov, rv, epsilon = 0.01),
            (None, None) => {}
            _ => panic!("Pressure mismatch"),
        }
        match (o.air_temp_deg, r.air_temp_deg) {
            (Some(ov), Some(rv)) => assert_relative_eq!(ov, rv, epsilon = 0.1),
            (None, None) => {}
            _ => panic!("Air temp mismatch"),
        }
    } else {
        panic!("Expected MDA, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_rmz() {
    let (orig, reparsed) = roundtrip_sentence("$PGRMZ,2282,f,3*21");
    if let (ParseResult::PGRMZ(o), ParseResult::PGRMZ(r)) = (&orig, &reparsed) {
        assert_eq!(o.altitude, r.altitude);
        assert_eq!(o.fix_type, r.fix_type);
    } else {
        panic!("Expected PGRMZ, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_ttm() {
    use approx::assert_relative_eq;

    let (orig, reparsed) = roundtrip_sentence(
        "$RATTM,01,0.2,190.8,T,12.1,109.7,T,0.1,0.5,N,TGT01,T,,100021.00,A*79",
    );
    if let (ParseResult::TTM(o), ParseResult::TTM(r)) = (&orig, &reparsed) {
        assert_eq!(o.target_number, r.target_number);
        assert_relative_eq!(
            o.target_distance.unwrap(),
            r.target_distance.unwrap()
        );
        assert_eq!(o.target_name, r.target_name);
    } else {
        panic!("Expected TTM, got {:?} and {:?}", orig, reparsed);
    }
}

#[test]
fn test_roundtrip_alm() {
    let (orig, reparsed) = roundtrip_sentence(
        "$GPALM,1,1,15,1159,00,441D,4E,16BE,FD5E,A10C9F,4A2DA4,686E81,58CBE1,0A4,001*77",
    );
    if let (ParseResult::ALM(o), ParseResult::ALM(r)) = (&orig, &reparsed) {
        assert_eq!(o.total_number_of_messages, r.total_number_of_messages);
        assert_eq!(o.sentence_number, r.sentence_number);
        assert_eq!(o.satellite_prn_number, r.satellite_prn_number);
        assert_eq!(o.gps_week_number, r.gps_week_number);
    } else {
        panic!("Expected ALM, got {:?} and {:?}", orig, reparsed);
    }
}

// ============================================================================
// Test 4: GSV roundtrip (satellite data) - special handling needed
// ============================================================================

#[test]
fn test_roundtrip_gsv() {
    let (orig, reparsed) = roundtrip_sentence(
        "$GPGSV,3,1,12,01,49,196,41,03,71,278,32,06,02,323,27,11,21,196,39*72",
    );
    if let (ParseResult::GSV(o), ParseResult::GSV(r)) = (&orig, &reparsed) {
        assert_eq!(o.number_of_sentences, r.number_of_sentences);
        assert_eq!(o.sentence_num, r.sentence_num);
        assert_eq!(o.sats_in_view, r.sats_in_view);
        // Compare satellite count
        assert_eq!(o.sats_info.len(), r.sats_info.len());
    } else {
        panic!("Expected GSV, got {:?} and {:?}", orig, reparsed);
    }
}

// ============================================================================
// Test 5: Construct-and-roundtrip for types with simple constructors
// ============================================================================

#[test]
fn test_construct_and_roundtrip_hdt() {
    use approx::assert_relative_eq;
    use nmea::sentences::HdtData;

    let data = HdtData {
        heading: Some(123.45),
    };
    let generated = generate_and_validate("GP", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::HDT(r) = reparsed {
        assert_relative_eq!(r.heading.unwrap(), 123.45);
    } else {
        panic!("Expected HDT");
    }
}

#[test]
fn test_construct_and_roundtrip_hdt_none() {
    use nmea::sentences::HdtData;

    let data = HdtData { heading: None };
    let generated = generate_and_validate("GP", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::HDT(r) = reparsed {
        assert_eq!(r.heading, None);
    } else {
        panic!("Expected HDT");
    }
}

#[test]
fn test_construct_and_roundtrip_hdm() {
    use approx::assert_relative_eq;
    use nmea::sentences::HdmData;

    let data = HdmData {
        heading: Some(45.6),
    };
    let generated = generate_and_validate("HC", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::HDM(r) = reparsed {
        assert_relative_eq!(r.heading.unwrap(), 45.6);
    } else {
        panic!("Expected HDM");
    }
}

#[test]
fn test_construct_and_roundtrip_mtw() {
    use nmea::sentences::MtwData;

    let data = MtwData {
        temperature: Some(22.5),
    };
    let generated = generate_and_validate("IN", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::MTW(r) = reparsed {
        assert_eq!(r.temperature, Some(22.5));
    } else {
        panic!("Expected MTW");
    }
}

#[test]
fn test_construct_and_roundtrip_rot() {
    use approx::assert_relative_eq;
    use nmea::sentences::RotData;

    let data = RotData {
        rate: Some(-5.2),
        valid: Some(true),
    };
    let generated = generate_and_validate("TI", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::ROT(r) = reparsed {
        assert_relative_eq!(r.rate.unwrap(), -5.2);
        assert_eq!(r.valid, Some(true));
    } else {
        panic!("Expected ROT");
    }
}

#[test]
fn test_construct_and_roundtrip_hsc() {
    use approx::assert_relative_eq;
    use nmea::sentences::HscData;

    let data = HscData {
        heading_true: Some(200.5),
        heading_magnetic: Some(210.3),
    };
    let generated = generate_and_validate("GP", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::HSC(r) = reparsed {
        assert_relative_eq!(r.heading_true.unwrap(), 200.5);
        assert_relative_eq!(r.heading_magnetic.unwrap(), 210.3);
    } else {
        panic!("Expected HSC");
    }
}

#[test]
fn test_construct_and_roundtrip_xte() {
    use approx::assert_relative_eq;
    use nmea::sentences::XteData;

    let data = XteData {
        cross_track_error: Some(-1.23),
        status_general: true,
        status_cycle_lock: true,
    };
    let generated = generate_and_validate("GP", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::XTE(r) = reparsed {
        assert_relative_eq!(r.cross_track_error.unwrap(), -1.23);
        assert!(r.status_general);
        assert!(r.status_cycle_lock);
    } else {
        panic!("Expected XTE");
    }
}

#[test]
fn test_construct_and_roundtrip_vdr() {
    use approx::assert_relative_eq;
    use nmea::sentences::VdrData;

    let data = VdrData {
        direction_true: Some(90.0),
        direction_magnetic: Some(85.0),
        speed: Some(2.5),
    };
    let generated = generate_and_validate("II", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::VDR(r) = reparsed {
        assert_relative_eq!(r.direction_true.unwrap(), 90.0);
        assert_relative_eq!(r.direction_magnetic.unwrap(), 85.0);
        assert_relative_eq!(r.speed.unwrap(), 2.5);
    } else {
        panic!("Expected VDR");
    }
}

#[test]
fn test_construct_and_roundtrip_vpw() {
    use approx::assert_relative_eq;
    use nmea::sentences::VpwData;

    let data = VpwData {
        speed_knots: Some(3.5),
        speed_mps: Some(1.8),
    };
    let generated = generate_and_validate("II", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::VPW(r) = reparsed {
        assert_relative_eq!(r.speed_knots.unwrap(), 3.5);
        assert_relative_eq!(r.speed_mps.unwrap(), 1.8);
    } else {
        panic!("Expected VPW");
    }
}

#[test]
fn test_construct_and_roundtrip_rsa() {
    use approx::assert_relative_eq;
    use nmea::sentences::RsaData;

    let data = RsaData {
        starboard: Some(5.0),
        starboard_valid: true,
        port: Some(-3.0),
        port_valid: true,
    };
    let generated = generate_and_validate("II", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::RSA(r) = reparsed {
        assert_relative_eq!(r.starboard.unwrap(), 5.0);
        assert!(r.starboard_valid);
        assert_relative_eq!(r.port.unwrap(), -3.0);
        assert!(r.port_valid);
    } else {
        panic!("Expected RSA");
    }
}

#[test]
fn test_construct_and_roundtrip_dpt() {
    use nmea::sentences::DptData;

    let data = DptData {
        water_depth: Some(25.5),
        offset: Some(1.2),
        max_range_scale: None,
    };
    let generated = generate_and_validate("SD", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::DPT(r) = reparsed {
        assert_eq!(r.water_depth, Some(25.5));
        assert_eq!(r.offset, Some(1.2));
        assert_eq!(r.max_range_scale, None);
    } else {
        panic!("Expected DPT");
    }
}

#[test]
fn test_construct_and_roundtrip_mwd() {
    use approx::assert_relative_eq;
    use nmea::sentences::MwdData;

    let data = MwdData {
        wind_direction_true: Some(180.0),
        wind_direction_magnetic: Some(175.0),
        wind_speed_knots: Some(15.0),
        wind_speed_mps: Some(7.7),
    };
    let generated = generate_and_validate("WI", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::MWD(r) = reparsed {
        assert_relative_eq!(r.wind_direction_true.unwrap(), 180.0);
        assert_relative_eq!(r.wind_direction_magnetic.unwrap(), 175.0);
        assert_relative_eq!(r.wind_speed_knots.unwrap(), 15.0);
        assert_relative_eq!(r.wind_speed_mps.unwrap(), 7.7);
    } else {
        panic!("Expected MWD");
    }
}

#[test]
fn test_construct_and_roundtrip_vtg() {
    use approx::assert_relative_eq;
    use nmea::sentences::VtgData;

    let data = VtgData {
        true_course: Some(270.0),
        speed_over_ground: Some(12.5),
    };
    let generated = generate_and_validate("GP", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::VTG(r) = reparsed {
        assert_relative_eq!(r.true_course.unwrap(), 270.0);
        // Speed gets written as knots and re-read; the knots field is authoritative.
        assert_relative_eq!(r.speed_over_ground.unwrap(), 12.5, epsilon = 0.01);
    } else {
        panic!("Expected VTG");
    }
}

#[test]
fn test_construct_and_roundtrip_pgrmz() {
    use nmea::sentences::rmz::{PgrmzData, PgrmzFixType};

    let data = PgrmzData {
        altitude: 5000,
        fix_type: PgrmzFixType::ThreeDimensional,
    };
    let generated = generate_and_validate("PG", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::PGRMZ(r) = reparsed {
        assert_eq!(r.altitude, 5000);
        assert_eq!(r.fix_type, PgrmzFixType::ThreeDimensional);
    } else {
        panic!("Expected PGRMZ");
    }
}

#[test]
fn test_construct_and_roundtrip_txt() {
    use arrayvec::ArrayString;
    use nmea::sentences::TxtData;

    let data = TxtData {
        count: 1,
        seq: 1,
        text_ident: 7,
        text: ArrayString::from("Hello NMEA").unwrap(),
    };
    let generated = generate_and_validate("GP", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::TXT(r) = reparsed {
        assert_eq!(r.count, 1);
        assert_eq!(r.seq, 1);
        assert_eq!(r.text_ident, 7);
        assert_eq!(r.text.as_str(), "Hello NMEA");
    } else {
        panic!("Expected TXT");
    }
}

// ============================================================================
// Test 6: Edge cases
// ============================================================================

#[test]
fn test_different_talker_ids_produce_valid_output() {
    use nmea::sentences::HdtData;

    let data = HdtData {
        heading: Some(90.0),
    };

    for talker in &["GP", "GN", "GL", "GA", "HC", "HE", "IN"] {
        let generated = generate_and_validate(talker, &data);
        let nmea = parse_nmea_sentence(&generated).unwrap();
        assert_eq!(nmea.talker_id, *talker);
        assert_eq!(nmea.message_id, SentenceType::HDT);
    }
}

#[test]
fn test_generated_sentence_does_not_exceed_max_length() {
    // NMEA 0183 maximum length is 82 characters including $ and \r\n.
    // The crate allows up to 102 characters.
    // Make sure generated sentences stay within the parseable limit.
    use nmea::sentences::HdtData;

    let data = HdtData {
        heading: Some(359.999),
    };
    let generated = generate_and_validate("GP", &data);
    assert!(
        generated.len() <= 102,
        "Generated sentence too long ({} chars): {}",
        generated.len(),
        generated
    );
}

#[test]
fn test_all_none_fields_still_produce_valid_sentence() {
    use nmea::sentences::HdtData;

    let data = HdtData { heading: None };
    let generated = generate_and_validate("GP", &data);
    // Should produce something like $GPHDT,,T*XX
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::HDT(r) = reparsed {
        assert_eq!(r.heading, None);
    } else {
        panic!("Expected HDT");
    }
}

#[test]
fn test_negative_values_roundtrip_correctly() {
    use approx::assert_relative_eq;
    use nmea::sentences::XteData;

    // Test negative cross track error (steer left)
    let data = XteData {
        cross_track_error: Some(-2.5),
        status_general: true,
        status_cycle_lock: true,
    };
    let generated = generate_and_validate("GP", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::XTE(r) = reparsed {
        assert_relative_eq!(r.cross_track_error.unwrap(), -2.5);
    } else {
        panic!("Expected XTE");
    }

    // Test positive cross track error (steer right)
    let data_pos = XteData {
        cross_track_error: Some(1.5),
        status_general: true,
        status_cycle_lock: false,
    };
    let generated_pos = generate_and_validate("GP", &data_pos);
    let reparsed_pos = parse_str(&generated_pos).unwrap();
    if let ParseResult::XTE(r) = reparsed_pos {
        assert_relative_eq!(r.cross_track_error.unwrap(), 1.5);
        assert!(!r.status_cycle_lock);
    } else {
        panic!("Expected XTE");
    }
}

#[test]
fn test_vwr_port_starboard_roundtrip() {
    use approx::assert_relative_eq;
    use nmea::sentences::VwrData;

    // Starboard (positive angle)
    let starboard = VwrData {
        wind_angle: Some(90.0),
        speed_knots: Some(10.0),
        speed_mps: Some(5.14),
        speed_kmph: Some(18.52),
    };
    let generated = generate_and_validate("II", &starboard);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::VWR(r) = reparsed {
        assert_relative_eq!(r.wind_angle.unwrap(), 90.0);
    } else {
        panic!("Expected VWR");
    }

    // Port (negative angle)
    let port = VwrData {
        wind_angle: Some(-45.0),
        speed_knots: Some(5.0),
        speed_mps: None,
        speed_kmph: None,
    };
    let generated_port = generate_and_validate("II", &port);
    let reparsed_port = parse_str(&generated_port).unwrap();
    if let ParseResult::VWR(r) = reparsed_port {
        assert_relative_eq!(r.wind_angle.unwrap(), -45.0);
    } else {
        panic!("Expected VWR");
    }
}

#[test]
fn test_hdg_east_west_variation_roundtrip() {
    use approx::assert_relative_eq;
    use nmea::sentences::HdgData;

    // West variation (negative)
    let data = HdgData {
        heading: Some(180.0),
        deviation: Some(1.5),
        variation: Some(-10.0),
    };
    let generated = generate_and_validate("HC", &data);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::HDG(r) = reparsed {
        assert_relative_eq!(r.heading.unwrap(), 180.0);
        assert_relative_eq!(r.deviation.unwrap(), 1.5);
        assert_relative_eq!(r.variation.unwrap(), -10.0);
    } else {
        panic!("Expected HDG");
    }

    // East variation (positive)
    let data_east = HdgData {
        heading: Some(90.0),
        deviation: None,
        variation: Some(5.0),
    };
    let generated_east = generate_and_validate("HC", &data_east);
    let reparsed_east = parse_str(&generated_east).unwrap();
    if let ParseResult::HDG(r) = reparsed_east {
        assert_relative_eq!(r.heading.unwrap(), 90.0);
        assert_eq!(r.deviation, None);
        assert_relative_eq!(r.variation.unwrap(), 5.0);
    } else {
        panic!("Expected HDG");
    }
}

#[test]
fn test_rpm_engine_and_shaft_roundtrip() {
    use approx::assert_relative_eq;
    use nmea::sentences::{RpmData, RpmSource};

    let engine = RpmData {
        source: Some(RpmSource::Engine),
        source_number: Some(1),
        rpm: Some(3000.0),
        pitch: Some(75.0),
        valid: true,
    };
    let generated = generate_and_validate("II", &engine);
    let reparsed = parse_str(&generated).unwrap();
    if let ParseResult::RPM(r) = reparsed {
        assert_eq!(r.source, Some(RpmSource::Engine));
        assert_eq!(r.source_number, Some(1));
        assert_relative_eq!(r.rpm.unwrap(), 3000.0);
        assert_relative_eq!(r.pitch.unwrap(), 75.0);
        assert!(r.valid);
    } else {
        panic!("Expected RPM");
    }

    let shaft = RpmData {
        source: Some(RpmSource::Shaft),
        source_number: Some(2),
        rpm: Some(1500.0),
        pitch: Some(-25.0),
        valid: false,
    };
    let gen_shaft = generate_and_validate("II", &shaft);
    let reparsed_shaft = parse_str(&gen_shaft).unwrap();
    if let ParseResult::RPM(r) = reparsed_shaft {
        assert_eq!(r.source, Some(RpmSource::Shaft));
        assert_relative_eq!(r.rpm.unwrap(), 1500.0);
        assert_relative_eq!(r.pitch.unwrap(), -25.0);
        assert!(!r.valid);
    } else {
        panic!("Expected RPM");
    }
}
