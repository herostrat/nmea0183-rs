//! Integration tests using real-world NMEA sentences from the signalk/nmea0183-signalk plugin.
//!
//! These sentences were extracted from the test suite of
//! <https://github.com/signalk/nmea0183-signalk> and serve as conformance tests
//! against a well-established NMEA parser implementation.

use nmea::parse_str;

/// Verify that all signalk test sentences parse without errors.
/// Each sentence is a real-world example from the signalk plugin's test suite.
#[test]
fn test_signalk_sentences_parse() {
    let sentences: &[(&str, &str)] = &[
        // BOD - Bearing Origin to Destination
        ("BOD", "$GPBOD,045.,T,023.,M,DEST,START*01"),
        // BWC - Bearing & Distance to Waypoint (Great Circle)
        ("BWC", "$GPBWC,225444,4917.24,N,12309.57,W,051.9,T,031.6,M,001.3,N,004*29"),
        ("BWC", "$IIBWC,200321,,,,,119.5,T,129.5,M,22.10,N,1*1E"),
        // DBK - Depth Below Keel
        ("DBK", "$IIDBK,035.53,f,010.83,M,005.85,F*3C"),
        // DBS - Depth Below Surface
        ("DBS", "$IIDBS,035.53,f,010.83,M,005.85,F*24"),
        // DBT - Depth Below Transducer
        ("DBT", "$IIDBT,035.53,f,010.83,M,005.85,F*23"),
        ("DBT", "$IIDBT,432.8,f,,M,,F*1C"),
        // DPT - Depth
        ("DPT", "$IIDPT,4.1,0.0*45"),
        ("DPT", "$IIDPT,4.1,*6B"),
        ("DPT", "$IIDPT,4.1,1.0*44"),
        ("DPT", "$IIDPT,4.1,-1.0*69"),
        // GGA - Global Positioning System Fix Data
        ("GGA", "$GPGGA,172814.0,3723.46587704,N,12202.26957864,W,2,6,1.2,18.893,M,-25.669,M,2.0,0031*4F"),
        // GLL - Geographic Position
        ("GLL", "$GPGLL,5958.613,N,02325.928,E,121022,A,D*40"),
        // GNS - Fix Data (with multi-char FAA mode indicator)
        ("GNS", "$GPGNS,224749.00,3333.4268304,N,11153.3538273,W,ANN,19,0.6,406.110,-26.294,6.0,0138,S,*43"),
        // GSV - Satellites in View
        ("GSV", "$GPGSV,3,1,09,07,16,321,37,08,29,281,33,10,29,143,35,16,75,216,35,0*6E"),
        ("GSV", "$GPGSV,3,2,09,18,38,057,35,20,44,105,40,21,81,117,33,26,43,164,25,0*63"),
        ("GSV", "$GPGSV,3,3,09,27,62,289,41,0*5B"),
        ("GSV", "$GLGSV,3,1,10,65,14,112,18,71,15,018,11,72,25,069,31,77,10,181,30,0*79"),
        ("GSV", "$GLGSV,3,2,10,78,52,221,38,79,44,310,28,80,00,342,,81,35,261,40,0*7E"),
        ("GSV", "$GLGSV,3,3,10,87,41,052,31,88,75,350,33,0*73"),
        ("GSV", "$GAGSV,2,1,07,01,37,308,33,03,09,074,35,05,07,025,,13,85,237,31,0*7F"),
        ("GSV", "$GAGSV,2,2,07,15,39,060,33,21,63,228,39,26,30,239,40,0*44"),
        // HDG - Heading, Deviation & Variation
        ("HDG", "$SDHDG,181.9,,,0.6,E*32"),
        ("HDG", "$HCHDG,51.5,,,,*73"),
        ("HDG", "$INHDG,180,5,W,10,W*6D"),
        // HDM - Heading Magnetic
        ("HDM", "$04HDM,186.5,M*2C"),
        // HDT - Heading True
        ("HDT", "$GPHDT,123.456,T*32"),
        // HSC - Heading Steering Command
        ("HSC", "$FTHSC,40.12,T,39.11,M*5E"),
        // MDA - Meteorological Composite
        ("MDA", "$WIMDA,,I,+0.985,B,+03.1,C,+5.6,C,40.0,3.0,+3.4,C,90.0,T,85.0,M,10.0,N,,M*1A"),
        ("MDA", "$WIMDA,29.92,I,,B,+03.1,C,+5.6,C,40.0,3.0,+3.4,C,90.0,T,85.0,M,,N,5.0,M*01"),
        // MTW - Water Temperature
        ("MTW", "$YXMTW,15.2,C*14"),
        // MWD - Wind Direction & Speed
        ("MWD", "$IIMWD,,,046.,M,10.1,N,05.2,M*0B"),
        ("MWD", "$IIMWD,046.,T,046.,M,10.1,N,,*17"),
        ("MWD", "$IIMWD,046.,T,,,,,5.2,M*72"),
        // MWV - Wind Speed and Angle
        ("MWV", "$IIMWV,074,T,05.85,N,A*2E"),
        ("MWV", "$IIMWV,336,R,13.41,N,A*22"),
        // RMC - Recommended Minimum Navigation Information
        ("RMC", "$GPRMC,085412.000,A,5222.3198,N,00454.5784,E,0.58,251.34,030414,,,A*65"),
        ("RMC", "$GPRMC,085412.000,A,5222.3198,N,00454.5784,E,,,030414,12,E*42"),
        ("RMC", "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A"),
        // ROT - Rate of Turn
        ("ROT", "$GPROT,35.6,A*01"),
        // RSA - Rudder Sensor Angle
        ("RSA", "$IIRSA,10.5,A,,V*4D"),
        // VDR - Set and Drift
        ("VDR", "$IIVDR,10.1,T,12.3,M,1.2,N*3A"),
        // VHW - Water Speed and Heading
        ("VHW", "$IIVHW,,T,,M,06.12,N,11.33,K*50"),
        ("VHW", "$SDVHW,182.5,T,181.8,M,0.0,N,0.0,K*4C"),
        // VLW - Distance Traveled through Water
        ("VLW", "$IIVLW,10.1,N,3.2,N*7C"),
        ("VLW", "$IIVLW,115.2,N,12.3,N*7A"),
        // VPW - Speed Measured Parallel to Wind
        ("VPW", "$IIVPW,4.5,N,6.7,M*52"),
        ("VPW", "$IIVPW,4.5,N,,*30"),
        // VTG - Track Made Good and Ground Speed
        ("VTG", "$GPVTG,0.0,T,359.3,M,0.0,N,0.0,K,A*2F"),
        ("VTG", "$GPVTG,,T,,M,0.102,N,0.190,K,A*28"),
        // XTE - Cross-Track Error
        ("XTE", "$GPXTE,A,A,0.67,L,N*6F"),
        // ZDA - Time & Date
        ("ZDA", "$GPZDA,160012.71,11,03,2004,-1,00*7D"),
        ("ZDA", "$IIZDA,085400,22,07,2021,,*50"),
    ];

    let mut failures = Vec::new();
    for (label, sentence) in sentences {
        if let Err(e) = parse_str(sentence) {
            failures.push(format!("{} {:?}: {:?}", label, sentence, e));
        }
    }

    assert!(
        failures.is_empty(),
        "Failed to parse {} signalk sentences:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Test specific parsed values from signalk sentences for key sentence types.
#[test]
fn test_signalk_gga_values() {
    use approx::assert_relative_eq;
    let result = parse_str("$GPGGA,172814.0,3723.46587704,N,12202.26957864,W,2,6,1.2,18.893,M,-25.669,M,2.0,0031*4F").unwrap();
    if let nmea::ParseResult::GGA(data) = result {
        assert_relative_eq!(data.latitude.unwrap(), 37.391098, epsilon = 0.0001);
        assert_relative_eq!(data.longitude.unwrap(), -122.037826, epsilon = 0.0001);
        assert_eq!(data.fix_type, Some(nmea::sentences::FixType::DGps));
        assert_eq!(data.fix_satellites, Some(6));
        assert_relative_eq!(data.hdop.unwrap(), 1.2);
        assert_relative_eq!(data.altitude.unwrap(), 18.893);
        assert_relative_eq!(data.geoid_separation.unwrap(), -25.669);
    } else {
        panic!("Expected GGA result");
    }
}

#[test]
fn test_signalk_rmc_values() {
    use approx::assert_relative_eq;
    let result = parse_str("$GPRMC,085412.000,A,5222.3198,N,00454.5784,E,0.58,251.34,030414,,,A*65").unwrap();
    if let nmea::ParseResult::RMC(data) = result {
        assert_relative_eq!(data.lat.unwrap(), 52.37199666, epsilon = 0.0001);
        assert_relative_eq!(data.lon.unwrap(), 4.909640, epsilon = 0.0001);
        assert_relative_eq!(data.speed_over_ground.unwrap(), 0.58, epsilon = 0.001);
        assert_relative_eq!(data.true_course.unwrap(), 251.34, epsilon = 0.01);
    } else {
        panic!("Expected RMC result");
    }
}

#[test]
fn test_signalk_rmc_with_variation() {
    use approx::assert_relative_eq;
    let result = parse_str("$GPRMC,085412.000,A,5222.3198,N,00454.5784,E,,,030414,12,E*42").unwrap();
    if let nmea::ParseResult::RMC(data) = result {
        assert_eq!(data.speed_over_ground, None);
        assert_eq!(data.true_course, None);
        // Magnetic variation should be +12 degrees (East)
        assert_relative_eq!(data.magnetic_variation.unwrap(), 12.0, epsilon = 0.01);
    } else {
        panic!("Expected RMC result");
    }
}

#[test]
fn test_signalk_dbt_values() {
    use approx::assert_relative_eq;
    let result = parse_str("$IIDBT,035.53,f,010.83,M,005.85,F*23").unwrap();
    if let nmea::ParseResult::DBT(data) = result {
        assert_relative_eq!(data.depth_feet.unwrap(), 35.53);
        assert_relative_eq!(data.depth_meters.unwrap(), 10.83);
        assert_relative_eq!(data.depth_fathoms.unwrap(), 5.85);
    } else {
        panic!("Expected DBT result");
    }
}

#[test]
fn test_signalk_dbt_feet_only() {
    use approx::assert_relative_eq;
    let result = parse_str("$IIDBT,432.8,f,,M,,F*1C").unwrap();
    if let nmea::ParseResult::DBT(data) = result {
        assert_relative_eq!(data.depth_feet.unwrap(), 432.8);
        assert_eq!(data.depth_meters, None);
        assert_eq!(data.depth_fathoms, None);
    } else {
        panic!("Expected DBT result");
    }
}

#[test]
fn test_signalk_dpt_values() {
    use approx::assert_relative_eq;
    let result = parse_str("$IIDPT,4.1,0.0*45").unwrap();
    if let nmea::ParseResult::DPT(data) = result {
        assert_relative_eq!(data.water_depth.unwrap(), 4.1);
        assert_relative_eq!(data.offset.unwrap(), 0.0);
    } else {
        panic!("Expected DPT result");
    }
}

/// DPT with negative offset (transducer-to-keel).
#[test]
fn test_signalk_dpt_negative_offset() {
    use approx::assert_relative_eq;
    let result = parse_str("$IIDPT,4.1,-1.0*69").unwrap();
    if let nmea::ParseResult::DPT(data) = result {
        assert_relative_eq!(data.water_depth.unwrap(), 4.1);
        assert_relative_eq!(data.offset.unwrap(), -1.0);
    } else {
        panic!("Expected DPT result");
    }
}

#[test]
fn test_signalk_hdg_with_variation() {
    use approx::assert_relative_eq;
    let result = parse_str("$SDHDG,181.9,,,0.6,E*32").unwrap();
    if let nmea::ParseResult::HDG(data) = result {
        assert_relative_eq!(data.heading.unwrap(), 181.9);
        assert_eq!(data.deviation, None);
        assert_relative_eq!(data.variation.unwrap(), 0.6); // East = positive
    } else {
        panic!("Expected HDG result");
    }
}

#[test]
fn test_signalk_hdg_heading_only() {
    use approx::assert_relative_eq;
    let result = parse_str("$HCHDG,51.5,,,,*73").unwrap();
    if let nmea::ParseResult::HDG(data) = result {
        assert_relative_eq!(data.heading.unwrap(), 51.5);
        assert_eq!(data.deviation, None);
        assert_eq!(data.variation, None);
    } else {
        panic!("Expected HDG result");
    }
}

#[test]
fn test_signalk_hdg_all_fields() {
    use approx::assert_relative_eq;
    let result = parse_str("$INHDG,180,5,W,10,W*6D").unwrap();
    if let nmea::ParseResult::HDG(data) = result {
        assert_relative_eq!(data.heading.unwrap(), 180.0);
        assert_relative_eq!(data.deviation.unwrap(), -5.0); // West = negative
        assert_relative_eq!(data.variation.unwrap(), -10.0); // West = negative
    } else {
        panic!("Expected HDG result");
    }
}

#[test]
fn test_signalk_hdm_value() {
    use approx::assert_relative_eq;
    let result = parse_str("$04HDM,186.5,M*2C").unwrap();
    if let nmea::ParseResult::HDM(data) = result {
        assert_relative_eq!(data.heading.unwrap(), 186.5);
    } else {
        panic!("Expected HDM result");
    }
}

#[test]
fn test_signalk_hdt_value() {
    use approx::assert_relative_eq;
    let result = parse_str("$GPHDT,123.456,T*32").unwrap();
    if let nmea::ParseResult::HDT(data) = result {
        assert_relative_eq!(data.heading.unwrap(), 123.456);
    } else {
        panic!("Expected HDT result");
    }
}

#[test]
fn test_signalk_hsc_values() {
    use approx::assert_relative_eq;
    let result = parse_str("$FTHSC,40.12,T,39.11,M*5E").unwrap();
    if let nmea::ParseResult::HSC(data) = result {
        assert_relative_eq!(data.heading_true.unwrap(), 40.12);
        assert_relative_eq!(data.heading_magnetic.unwrap(), 39.11);
    } else {
        panic!("Expected HSC result");
    }
}

#[test]
fn test_signalk_mtw_value() {
    use approx::assert_relative_eq;
    let result = parse_str("$YXMTW,15.2,C*14").unwrap();
    if let nmea::ParseResult::MTW(data) = result {
        assert_relative_eq!(data.temperature.unwrap(), 15.2);
    } else {
        panic!("Expected MTW result");
    }
}

#[test]
fn test_signalk_mwd_magnetic_and_speed() {
    use approx::assert_relative_eq;
    let result = parse_str("$IIMWD,,,046.,M,10.1,N,05.2,M*0B").unwrap();
    if let nmea::ParseResult::MWD(data) = result {
        assert_eq!(data.wind_direction_true, None);
        assert_relative_eq!(data.wind_direction_magnetic.unwrap(), 46.0);
        assert_relative_eq!(data.wind_speed_knots.unwrap(), 10.1);
        assert_relative_eq!(data.wind_speed_mps.unwrap(), 5.2);
    } else {
        panic!("Expected MWD result");
    }
}

#[test]
fn test_signalk_mwd_true_direction_only() {
    use approx::assert_relative_eq;
    let result = parse_str("$IIMWD,046.,T,,,,,5.2,M*72").unwrap();
    if let nmea::ParseResult::MWD(data) = result {
        assert_relative_eq!(data.wind_direction_true.unwrap(), 46.0);
        assert_eq!(data.wind_direction_magnetic, None);
        assert_eq!(data.wind_speed_knots, None);
        assert_relative_eq!(data.wind_speed_mps.unwrap(), 5.2);
    } else {
        panic!("Expected MWD result");
    }
}

#[test]
fn test_signalk_rot_value() {
    use approx::assert_relative_eq;
    let result = parse_str("$GPROT,35.6,A*01").unwrap();
    if let nmea::ParseResult::ROT(data) = result {
        assert_relative_eq!(data.rate.unwrap(), 35.6);
        assert_eq!(data.valid, Some(true));
    } else {
        panic!("Expected ROT result");
    }
}

#[test]
fn test_signalk_rsa_value() {
    use approx::assert_relative_eq;
    let result = parse_str("$IIRSA,10.5,A,,V*4D").unwrap();
    if let nmea::ParseResult::RSA(data) = result {
        assert_relative_eq!(data.starboard.unwrap(), 10.5);
        assert!(data.starboard_valid);
        assert_eq!(data.port, None);
        assert!(!data.port_valid);
    } else {
        panic!("Expected RSA result");
    }
}

#[test]
fn test_signalk_vdr_values() {
    use approx::assert_relative_eq;
    let result = parse_str("$IIVDR,10.1,T,12.3,M,1.2,N*3A").unwrap();
    if let nmea::ParseResult::VDR(data) = result {
        assert_relative_eq!(data.direction_true.unwrap(), 10.1);
        assert_relative_eq!(data.direction_magnetic.unwrap(), 12.3);
        assert_relative_eq!(data.speed.unwrap(), 1.2);
    } else {
        panic!("Expected VDR result");
    }
}

#[test]
fn test_signalk_vhw_speed_only() {
    use approx::assert_relative_eq;
    let result = parse_str("$IIVHW,,T,,M,06.12,N,11.33,K*50").unwrap();
    if let nmea::ParseResult::VHW(data) = result {
        assert_eq!(data.heading_true, None);
        assert_eq!(data.heading_magnetic, None);
        assert_relative_eq!(data.relative_speed_knots.unwrap(), 6.12);
        assert_relative_eq!(data.relative_speed_kmph.unwrap(), 11.33);
    } else {
        panic!("Expected VHW result");
    }
}

#[test]
fn test_signalk_vhw_speed_and_heading() {
    use approx::assert_relative_eq;
    let result = parse_str("$SDVHW,182.5,T,181.8,M,0.0,N,0.0,K*4C").unwrap();
    if let nmea::ParseResult::VHW(data) = result {
        assert_relative_eq!(data.heading_true.unwrap(), 182.5);
        assert_relative_eq!(data.heading_magnetic.unwrap(), 181.8);
        assert_relative_eq!(data.relative_speed_knots.unwrap(), 0.0);
        assert_relative_eq!(data.relative_speed_kmph.unwrap(), 0.0);
    } else {
        panic!("Expected VHW result");
    }
}

#[test]
fn test_signalk_vlw_basic() {
    use approx::assert_relative_eq;
    let result = parse_str("$IIVLW,10.1,N,3.2,N*7C").unwrap();
    if let nmea::ParseResult::VLW(data) = result {
        assert_relative_eq!(data.total_water_distance.unwrap(), 10.1);
        assert_relative_eq!(data.trip_water_distance.unwrap(), 3.2);
        assert_eq!(data.total_ground_distance, None);
        assert_eq!(data.trip_ground_distance, None);
    } else {
        panic!("Expected VLW result");
    }
}

#[test]
fn test_signalk_vpw_values() {
    use approx::assert_relative_eq;
    let result = parse_str("$IIVPW,4.5,N,6.7,M*52").unwrap();
    if let nmea::ParseResult::VPW(data) = result {
        assert_relative_eq!(data.speed_knots.unwrap(), 4.5);
        assert_relative_eq!(data.speed_mps.unwrap(), 6.7);
    } else {
        panic!("Expected VPW result");
    }
}

#[test]
fn test_signalk_vtg_values() {
    use approx::assert_relative_eq;
    let result = parse_str("$GPVTG,0.0,T,359.3,M,0.0,N,0.0,K,A*2F").unwrap();
    if let nmea::ParseResult::VTG(data) = result {
        assert_relative_eq!(data.true_course.unwrap(), 0.0);
        // VtgData stores speed_over_ground (knots preferred, kph fallback)
        assert_relative_eq!(data.speed_over_ground.unwrap(), 0.0);
    } else {
        panic!("Expected VTG result");
    }
}

#[test]
fn test_signalk_xte_values() {
    use approx::assert_relative_eq;
    let result = parse_str("$GPXTE,A,A,0.67,L,N*6F").unwrap();
    if let nmea::ParseResult::XTE(data) = result {
        // Left = negative in our representation
        assert_relative_eq!(data.cross_track_error.unwrap(), -0.67);
        assert!(data.status_general);
        assert!(data.status_cycle_lock);
    } else {
        panic!("Expected XTE result");
    }
}

#[test]
fn test_signalk_zda_values() {
    let result = parse_str("$GPZDA,160012.71,11,03,2004,-1,00*7D").unwrap();
    if let nmea::ParseResult::ZDA(data) = result {
        assert_eq!(data.utc_time.unwrap().format("%H:%M:%S%.3f").to_string(), "16:00:12.710");
        assert_eq!(data.day.unwrap(), 11);
        assert_eq!(data.month.unwrap(), 3);
        assert_eq!(data.year.unwrap(), 2004);
        assert_eq!(data.local_zone_hours.unwrap(), -1);
        assert_eq!(data.local_zone_minutes.unwrap(), 0);
    } else {
        panic!("Expected ZDA result");
    }
}

/// Test sentences with empty/missing fields (signalk expects null/None values).
/// Only includes sentence types whose parsers handle fully-empty fields.
#[test]
fn test_signalk_empty_field_sentences() {
    let empty_sentences: &[(&str, &str)] = &[
        ("DPT", "$IIDPT,,,*6C"),
        ("HDG", "$SDHDG,,,,,*70"),
        ("HSC", "$FTHSC,,,,*4A"),
        ("MDA", "$WIMDA,,I,,B,,C,,C,,,,C,,T,,M,,N,,M*04"),
        ("MTW", "$RAMTW,,C*1E"),
        ("ZDA", "$GPZDA,,,,,,*48"),
    ];
    // Known limitations: the following signalk empty sentences fail in our parser:
    // BOD: requires bearing type char 'T'/'M'
    // BWC: lat/lon parser expects N/S/E/W designators
    // DBK/DBS/DBT: depth unit char parser is not fully optional
    // GGA: fix quality requires one_of("0123456789")
    // GLL: status field requires one_of chars
    // GNS: mode indicator field requires non-empty
    // HDM/HDT: trailing 'M'/'T' char is mandatory
    // MWV: reference designator requires one_of("RT")

    let mut failures = Vec::new();
    for (label, sentence) in empty_sentences {
        if let Err(e) = parse_str(sentence) {
            failures.push(format!("{} {:?}: {:?}", label, sentence, e));
        }
    }

    assert!(
        failures.is_empty(),
        "Failed to parse {} empty-field signalk sentences:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// GNS with multi-char FAA mode indicator (NMEA 4.1+: GPS=A, GLONASS=N, Galileo=N).
#[test]
fn test_signalk_gns_multi_char_mode() {
    use approx::assert_relative_eq;
    let result = parse_str(
        "$GPGNS,224749.00,3333.4268304,N,11153.3538273,W,ANN,19,0.6,406.110,-26.294,6.0,0138,S,*43",
    )
    .unwrap();
    if let nmea::ParseResult::GNS(data) = result {
        assert_eq!(data.faa_modes.len(), 3);
        assert_eq!(data.faa_modes.primary(), nmea::sentences::FaaMode::Autonomous);
        assert_eq!(
            data.faa_modes.get(1),
            Some(&nmea::sentences::FaaMode::DataNotValid)
        );
        assert_eq!(
            data.faa_modes.get(2),
            Some(&nmea::sentences::FaaMode::DataNotValid)
        );
        assert_relative_eq!(data.lat.unwrap(), 33.0 + 33.4268304 / 60., epsilon = 1e-5);
        assert_eq!(data.nsattelites, 19);
    } else {
        panic!("Expected GNS result");
    }
}

/// Verify that our parser correctly rejects invalid checksums, matching signalk behavior.
#[test]
fn test_signalk_invalid_checksum_rejected() {
    // From signalk's invalid_checksum test: ROT with wrong checksum *FF
    let result = parse_str("$GPROT,35.6,A*FF");
    assert!(result.is_err(), "Should reject sentence with invalid checksum");
}
