//! Serialization utilities for NMEA sentence generation.

use core::fmt::{self, Write};

use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};

/// Write an optional value, or nothing if None.
pub fn write_opt<T: fmt::Display>(f: &mut dyn Write, val: &Option<T>) -> fmt::Result {
    if let Some(v) = val {
        write!(f, "{}", v)?;
    }
    Ok(())
}

/// Write an optional value followed by a comma.
pub fn write_field<T: fmt::Display>(f: &mut dyn Write, val: &Option<T>) -> fmt::Result {
    write_opt(f, val)?;
    f.write_char(',')
}

/// Write time as `hhmmss[.fractional]` in NMEA format.
pub fn write_hms(f: &mut dyn Write, time: &Option<NaiveTime>) -> fmt::Result {
    if let Some(t) = time {
        let nanos = t.nanosecond();
        write!(f, "{:02}{:02}{:02}", t.hour(), t.minute(), t.second())?;
        if nanos > 0 {
            let millis = nanos / 1_000_000;
            if nanos % 1_000_000 == 0 {
                write!(f, ".{:03}", millis)?;
            } else {
                let micros = nanos / 1_000;
                if nanos % 1_000 == 0 {
                    write!(f, ".{:06}", micros)?;
                } else {
                    write!(f, ".{:09}", nanos)?;
                }
            }
        }
    }
    Ok(())
}

/// Write date as `ddmmyy` in NMEA format.
pub fn write_date(f: &mut dyn Write, date: &Option<NaiveDate>) -> fmt::Result {
    if let Some(d) = date {
        write!(f, "{:02}{:02}{:02}", d.day(), d.month(), d.year() % 100)?;
    }
    Ok(())
}

/// Write latitude as `ddmm.mmmmm,N/S` or `,` if None.
pub fn write_lat(f: &mut dyn Write, lat: &Option<f64>) -> fmt::Result {
    match lat {
        Some(lat) => {
            let dir = if *lat >= 0.0 { 'N' } else { 'S' };
            let lat = lat.abs();
            let degrees = num_traits::float::Float::trunc(lat) as u32;
            let minutes = (lat - degrees as f64) * 60.0;
            write!(f, "{:02}{:010.7},{}", degrees, minutes, dir)
        }
        None => f.write_char(','),
    }
}

/// Write longitude as `dddmm.mmmmm,E/W` or `,` if None.
pub fn write_lon(f: &mut dyn Write, lon: &Option<f64>) -> fmt::Result {
    match lon {
        Some(lon) => {
            let dir = if *lon >= 0.0 { 'E' } else { 'W' };
            let lon = lon.abs();
            let degrees = num_traits::float::Float::trunc(lon) as u32;
            let minutes = (lon - degrees as f64) * 60.0;
            write!(f, "{:03}{:010.7},{}", degrees, minutes, dir)
        }
        None => f.write_char(','),
    }
}

/// Write `lat,N/S,lon,E/W` or `,,,,` if both None.
pub fn write_lat_lon(f: &mut dyn Write, lat: &Option<f64>, lon: &Option<f64>) -> fmt::Result {
    write_lat(f, lat)?;
    f.write_char(',')?;
    write_lon(f, lon)
}
