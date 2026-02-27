//! Property-based tests for NMEA parsing and generation roundtrips.
//!
//! Uses QuickCheck to verify that:
//! - `write_X(value)` -> `parse_X(output)` ≈ `value` for utility functions
//! - `generate(data)` -> `parse(output)` ≈ `data` for sentence types

use quickcheck::{QuickCheck, TestResult};

// ──────────────────────────────────────────────
// Lat/Lon roundtrip via gen_utils::write_lat/write_lon + utils::do_parse_lat_lon
// ──────────────────────────────────────────────

fn check_lat_lon_roundtrip(lat: f64, lon: f64) -> TestResult {
    if !lat.is_finite() || !lon.is_finite() {
        return TestResult::discard();
    }
    let lat = lat % 90.0;
    let lon = lon % 180.0;

    let mut buf = String::new();
    nmea::sentences::gen_utils::write_lat(&mut buf, &Some(lat)).unwrap();
    buf.push(',');
    nmea::sentences::gen_utils::write_lon(&mut buf, &Some(lon)).unwrap();

    let result = nmea::sentences::utils::do_parse_lat_lon(&buf);
    match result {
        Ok((_, (parsed_lat, parsed_lon))) => {
            const MAX_DIFF: f64 = 1e-6;
            TestResult::from_bool(
                (parsed_lat - lat).abs() < MAX_DIFF && (parsed_lon - lon).abs() < MAX_DIFF,
            )
        }
        Err(_) => TestResult::failed(),
    }
}

#[test]
fn prop_lat_lon_roundtrip() {
    // Regression cases
    assert!(!check_lat_lon_roundtrip(0.0, 57.89528).is_failure());
    assert!(!check_lat_lon_roundtrip(0.0, -43.33031).is_failure());
    assert!(!check_lat_lon_roundtrip(89.999, 179.999).is_failure());
    assert!(!check_lat_lon_roundtrip(-89.999, -179.999).is_failure());
    assert!(!check_lat_lon_roundtrip(0.0, 0.0).is_failure());

    QuickCheck::new()
        .tests(100_000)
        .quickcheck(check_lat_lon_roundtrip as fn(f64, f64) -> TestResult);
}

// ──────────────────────────────────────────────
// HMS roundtrip via gen_utils::write_hms + utils::parse_hms
// ──────────────────────────────────────────────

fn check_hms_roundtrip(h: u8, m: u8, s: u8, ms: u16) -> TestResult {
    if h >= 24 || m >= 60 || s >= 60 || ms >= 1000 {
        return TestResult::discard();
    }

    let time = match chrono::NaiveTime::from_hms_milli_opt(h as u32, m as u32, s as u32, ms as u32)
    {
        Some(t) => t,
        None => return TestResult::discard(),
    };

    let mut buf = String::new();
    nmea::sentences::gen_utils::write_hms(&mut buf, &Some(time)).unwrap();
    buf.push(','); // parse_hms expects trailing comma

    let result = nmea::sentences::utils::parse_hms(&buf);
    match result {
        Ok((_, parsed)) => {
            // Millisecond precision is all we can guarantee through NMEA format
            let orig_ms = time.format("%H%M%S%.3f").to_string();
            let parsed_ms = parsed.format("%H%M%S%.3f").to_string();
            TestResult::from_bool(orig_ms == parsed_ms)
        }
        Err(_) => TestResult::failed(),
    }
}

#[test]
fn prop_hms_roundtrip() {
    assert!(!check_hms_roundtrip(0, 0, 0, 0).is_failure());
    assert!(!check_hms_roundtrip(23, 59, 59, 999).is_failure());
    assert!(!check_hms_roundtrip(12, 30, 0, 500).is_failure());

    QuickCheck::new()
        .tests(100_000)
        .quickcheck(check_hms_roundtrip as fn(u8, u8, u8, u16) -> TestResult);
}

// ──────────────────────────────────────────────
// Date roundtrip through ZDA sentence
// ──────────────────────────────────────────────

fn check_date_through_zda(day: u8, month: u8, year: u16) -> TestResult {
    if !(1..=12).contains(&month) || !(1..=28).contains(&day) {
        return TestResult::discard();
    }
    let year = year % 9000 + 1000;

    let time = chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap();
    let data = nmea::sentences::ZdaData {
        utc_time: Some(time),
        day: Some(day),
        month: Some(month),
        year: Some(year),
        local_zone_hours: Some(0),
        local_zone_minutes: Some(0),
    };

    let mut buf = String::new();
    nmea::generate::generate_sentence("GP", &data, &mut buf).unwrap();

    match nmea::parse_str(&buf) {
        Ok(nmea::ParseResult::ZDA(p)) => TestResult::from_bool(
            p.day == Some(day) && p.month == Some(month) && p.year == Some(year),
        ),
        _ => TestResult::failed(),
    }
}

#[test]
#[cfg(feature = "ZDA")]
fn prop_date_through_zda() {
    assert!(!check_date_through_zda(1, 1, 2000).is_failure());
    assert!(!check_date_through_zda(28, 12, 2024).is_failure());
    assert!(!check_date_through_zda(15, 6, 1983).is_failure());

    QuickCheck::new()
        .tests(10_000)
        .quickcheck(check_date_through_zda as fn(u8, u8, u16) -> TestResult);
}

// ──────────────────────────────────────────────
// HDT roundtrip (f32 heading)
// ──────────────────────────────────────────────

fn check_hdt_roundtrip(heading: f32) -> TestResult {
    if !heading.is_finite() || heading < 0.0 || heading >= 360.0 {
        return TestResult::discard();
    }

    let data = nmea::sentences::HdtData {
        heading: Some(heading),
    };
    let mut buf = String::new();
    nmea::generate::generate_sentence("GP", &data, &mut buf).unwrap();

    match nmea::parse_str(&buf) {
        Ok(nmea::ParseResult::HDT(p)) => {
            TestResult::from_bool((p.heading.unwrap() - heading).abs() < 0.01)
        }
        _ => TestResult::failed(),
    }
}

#[test]
#[cfg(feature = "HDT")]
fn prop_hdt_roundtrip() {
    assert!(!check_hdt_roundtrip(0.0).is_failure());
    assert!(!check_hdt_roundtrip(359.99).is_failure());
    assert!(!check_hdt_roundtrip(180.5).is_failure());

    QuickCheck::new()
        .tests(10_000)
        .quickcheck(check_hdt_roundtrip as fn(f32) -> TestResult);
}

// ──────────────────────────────────────────────
// MTW roundtrip (f64 temperature)
// ──────────────────────────────────────────────

fn check_mtw_roundtrip(temperature: f64) -> TestResult {
    if !temperature.is_finite() || temperature.abs() > 999.0 {
        return TestResult::discard();
    }

    let data = nmea::sentences::MtwData {
        temperature: Some(temperature),
    };
    let mut buf = String::new();
    nmea::generate::generate_sentence("YX", &data, &mut buf).unwrap();

    match nmea::parse_str(&buf) {
        Ok(nmea::ParseResult::MTW(p)) => {
            TestResult::from_bool((p.temperature.unwrap() - temperature).abs() < 0.01)
        }
        _ => TestResult::failed(),
    }
}

#[test]
#[cfg(feature = "MTW")]
fn prop_mtw_roundtrip() {
    assert!(!check_mtw_roundtrip(0.0).is_failure());
    assert!(!check_mtw_roundtrip(-5.5).is_failure());
    assert!(!check_mtw_roundtrip(35.2).is_failure());

    QuickCheck::new()
        .tests(10_000)
        .quickcheck(check_mtw_roundtrip as fn(f64) -> TestResult);
}

// ──────────────────────────────────────────────
// DPT roundtrip (f64 depth + offset)
// ──────────────────────────────────────────────

fn check_dpt_roundtrip(depth: f64, offset: f64) -> TestResult {
    if !depth.is_finite() || !offset.is_finite() {
        return TestResult::discard();
    }
    // DPT parser uses nom::float which rejects negative zero display "-0"
    if depth <= 0.0 || depth > 9999.0 || offset.abs() > 999.0 {
        return TestResult::discard();
    }

    let data = nmea::sentences::DptData {
        water_depth: Some(depth),
        offset: Some(offset),
        max_range_scale: None,
    };
    let mut buf = String::new();
    nmea::generate::generate_sentence("SD", &data, &mut buf).unwrap();

    match nmea::parse_str(&buf) {
        Ok(nmea::ParseResult::DPT(p)) => TestResult::from_bool(
            (p.water_depth.unwrap() - depth).abs() < 0.01
                && (p.offset.unwrap() - offset).abs() < 0.01,
        ),
        _ => TestResult::failed(),
    }
}

#[test]
#[cfg(feature = "DPT")]
fn prop_dpt_roundtrip() {
    assert!(!check_dpt_roundtrip(0.0, 0.0).is_failure());
    assert!(!check_dpt_roundtrip(100.5, 1.5).is_failure());
    assert!(!check_dpt_roundtrip(4.1, 0.0).is_failure());

    QuickCheck::new()
        .tests(10_000)
        .quickcheck(check_dpt_roundtrip as fn(f64, f64) -> TestResult);
}

// ──────────────────────────────────────────────
// XTE roundtrip (f32 signed cross track error)
// ──────────────────────────────────────────────

fn check_xte_roundtrip(error: f32) -> TestResult {
    if !error.is_finite() || error.abs() > 999.0 || error == 0.0 {
        return TestResult::discard();
    }

    let data = nmea::sentences::XteData {
        cross_track_error: Some(error),
        status_general: true,
        status_cycle_lock: true,
    };
    let mut buf = String::new();
    nmea::generate::generate_sentence("GP", &data, &mut buf).unwrap();

    match nmea::parse_str(&buf) {
        Ok(nmea::ParseResult::XTE(p)) => {
            TestResult::from_bool((p.cross_track_error.unwrap() - error).abs() < 0.01)
        }
        _ => TestResult::failed(),
    }
}

#[test]
#[cfg(feature = "XTE")]
fn prop_xte_roundtrip() {
    assert!(!check_xte_roundtrip(0.5).is_failure());
    assert!(!check_xte_roundtrip(-0.67).is_failure());
    assert!(!check_xte_roundtrip(12.34).is_failure());

    QuickCheck::new()
        .tests(10_000)
        .quickcheck(check_xte_roundtrip as fn(f32) -> TestResult);
}

// ──────────────────────────────────────────────
// Lat/Lon roundtrip through GGA (full sentence)
// ──────────────────────────────────────────────

#[cfg(feature = "GGA")]
fn check_lat_lon_through_gga(lat: f64, lon: f64) -> TestResult {
    if !lat.is_finite() || !lon.is_finite() {
        return TestResult::discard();
    }
    let lat = lat % 90.0;
    let lon = lon % 180.0;

    let data = nmea::sentences::GgaData {
        fix_time: chrono::NaiveTime::from_hms_opt(12, 0, 0),
        latitude: Some(lat),
        longitude: Some(lon),
        fix_type: Some(nmea::sentences::FixType::Gps),
        fix_satellites: Some(8),
        hdop: Some(1.0),
        altitude: Some(100.0),
        geoid_separation: Some(0.0),
    };

    let mut buf = String::new();
    nmea::generate::generate_sentence("GP", &data, &mut buf).unwrap();

    match nmea::parse_str(&buf) {
        Ok(nmea::ParseResult::GGA(p)) => {
            const MAX_DIFF: f64 = 1e-6;
            let lat_ok = (p.latitude.unwrap() - lat).abs() < MAX_DIFF;
            let lon_ok = (p.longitude.unwrap() - lon).abs() < MAX_DIFF;
            TestResult::from_bool(lat_ok && lon_ok)
        }
        _ => TestResult::failed(),
    }
}

#[test]
#[cfg(feature = "GGA")]
fn prop_lat_lon_through_gga() {
    assert!(!check_lat_lon_through_gga(0.0, 57.89528).is_failure());
    assert!(!check_lat_lon_through_gga(0.0, -43.33031).is_failure());
    assert!(!check_lat_lon_through_gga(48.117, 11.522).is_failure());

    QuickCheck::new()
        .tests(100_000)
        .quickcheck(check_lat_lon_through_gga as fn(f64, f64) -> TestResult);
}

// ──────────────────────────────────────────────
// VHW roundtrip (multiple f64 fields)
// ──────────────────────────────────────────────

fn check_vhw_roundtrip(heading_t: f64, heading_m: f64, knots: f64, kmph: f64) -> TestResult {
    if [heading_t, heading_m, knots, kmph]
        .iter()
        .any(|v| !v.is_finite())
    {
        return TestResult::discard();
    }
    if heading_t < 0.0
        || heading_t >= 360.0
        || heading_m < 0.0
        || heading_m >= 360.0
        || knots < 0.0
        || knots > 999.0
        || kmph < 0.0
        || kmph > 999.0
    {
        return TestResult::discard();
    }

    let data = nmea::sentences::VhwData {
        heading_true: Some(heading_t),
        heading_magnetic: Some(heading_m),
        relative_speed_knots: Some(knots),
        relative_speed_kmph: Some(kmph),
    };
    let mut buf = String::new();
    nmea::generate::generate_sentence("II", &data, &mut buf).unwrap();

    match nmea::parse_str(&buf) {
        Ok(nmea::ParseResult::VHW(p)) => TestResult::from_bool(
            (p.heading_true.unwrap() - heading_t).abs() < 0.01
                && (p.heading_magnetic.unwrap() - heading_m).abs() < 0.01
                && (p.relative_speed_knots.unwrap() - knots).abs() < 0.01
                && (p.relative_speed_kmph.unwrap() - kmph).abs() < 0.01,
        ),
        _ => TestResult::failed(),
    }
}

#[test]
#[cfg(feature = "VHW")]
fn prop_vhw_roundtrip() {
    assert!(!check_vhw_roundtrip(0.0, 0.0, 0.0, 0.0).is_failure());
    assert!(!check_vhw_roundtrip(182.5, 181.8, 6.12, 11.33).is_failure());

    QuickCheck::new()
        .tests(10_000)
        .quickcheck(check_vhw_roundtrip as fn(f64, f64, f64, f64) -> TestResult);
}

// ──────────────────────────────────────────────
// HDG roundtrip (f32 heading + deviation + variation with E/W sign)
// ──────────────────────────────────────────────

fn check_hdg_roundtrip(heading: f32, deviation: f32, variation: f32) -> TestResult {
    if [heading, deviation, variation]
        .iter()
        .any(|v| !v.is_finite())
    {
        return TestResult::discard();
    }
    if heading < 0.0 || heading >= 360.0 || deviation.abs() > 180.0 || variation.abs() > 180.0 {
        return TestResult::discard();
    }
    // Avoid 0.0 for deviation/variation - sign (E/W) is ambiguous
    if deviation == 0.0 || variation == 0.0 {
        return TestResult::discard();
    }

    let data = nmea::sentences::HdgData {
        heading: Some(heading),
        deviation: Some(deviation),
        variation: Some(variation),
    };
    let mut buf = String::new();
    nmea::generate::generate_sentence("HC", &data, &mut buf).unwrap();

    match nmea::parse_str(&buf) {
        Ok(nmea::ParseResult::HDG(p)) => TestResult::from_bool(
            (p.heading.unwrap() - heading).abs() < 0.01
                && (p.deviation.unwrap() - deviation).abs() < 0.01
                && (p.variation.unwrap() - variation).abs() < 0.01,
        ),
        _ => TestResult::failed(),
    }
}

#[test]
#[cfg(feature = "HDG")]
fn prop_hdg_roundtrip() {
    assert!(!check_hdg_roundtrip(180.0, 5.0, -10.0).is_failure());
    assert!(!check_hdg_roundtrip(0.1, -5.0, 10.0).is_failure());

    QuickCheck::new()
        .tests(10_000)
        .quickcheck(check_hdg_roundtrip as fn(f32, f32, f32) -> TestResult);
}

// ──────────────────────────────────────────────
// ROT roundtrip (f32 signed rate of turn)
// ──────────────────────────────────────────────

fn check_rot_roundtrip(rate: f32) -> TestResult {
    if !rate.is_finite() || rate.abs() > 720.0 {
        return TestResult::discard();
    }

    let data = nmea::sentences::RotData {
        rate: Some(rate),
        valid: Some(true),
    };
    let mut buf = String::new();
    nmea::generate::generate_sentence("TI", &data, &mut buf).unwrap();

    match nmea::parse_str(&buf) {
        Ok(nmea::ParseResult::ROT(p)) => {
            TestResult::from_bool((p.rate.unwrap() - rate).abs() < 0.01)
        }
        _ => TestResult::failed(),
    }
}

#[test]
#[cfg(feature = "ROT")]
fn prop_rot_roundtrip() {
    assert!(!check_rot_roundtrip(0.0).is_failure());
    assert!(!check_rot_roundtrip(-35.6).is_failure());
    assert!(!check_rot_roundtrip(720.0).is_failure());

    QuickCheck::new()
        .tests(10_000)
        .quickcheck(check_rot_roundtrip as fn(f32) -> TestResult);
}
