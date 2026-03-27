//! Parser for Bolder Flight Systems OnFlight Hub `.onflight` binary data logs.
//!
//! Based on the OnFlight Hub Developers Manual. Returns row-oriented [`Frame`]
//! structs matching the wire format. Each frame is one 50 Hz sample.
//!
//! # Example
//! ```ignore
//! let data = std::fs::read("data0.onflight")?;
//! let file = onflight::parse(&data, &onflight::ParseOptions::default())?;
//! println!("Tail: {}, {} frames", file.header.tail_number, file.frames.len());
//! for frame in &file.frames {
//!     println!("t={}ms alt={}ft", frame.sys_time_ms, frame.ins_alt_wgs84 as i32 - 10000);
//! }
//! ```

mod frame;
mod checksum;

pub use frame::Frame;
pub use checksum::fletcher16;

use core::fmt;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ParseError {
    FileTooSmall,
    BadMagic,
    NoDataFrames,
    UnsupportedVersion { version: u8 },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileTooSmall => write!(f, "File too small to contain header"),
            Self::BadMagic => write!(f, "Bad magic: expected 'MD' header"),
            Self::NoDataFrames => write!(f, "No valid data frames found"),
            Self::UnsupportedVersion { version } =>
                write!(f, "Unsupported data frame version: {}", version),
        }
    }
}

impl std::error::Error for ParseError {}

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// Options controlling parse behavior.
pub struct ParseOptions {
    /// Validate Fletcher 16 checksums on each frame. Default: true.
    pub validate_checksums: bool,
    /// If true, skip frames with bad checksums instead of including them. Default: false.
    pub skip_bad_checksums: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            validate_checksums: true,
            skip_bad_checksums: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Parsed `.onflight` file: metadata header + sequence of data frames.
pub struct OnFlightFile {
    pub header: FileHeader,
    pub frames: Vec<Frame>,
    /// Number of frames that failed checksum validation (0 if not validated).
    pub checksum_errors: usize,
}

/// File-level metadata written once at the start of the log.
#[derive(Debug, Clone)]
pub struct FileHeader {
    /// Meta data format version.
    pub version: u8,
    /// OnFlight Hub serial number (6 bytes).
    pub serial_number: [u8; 6],
    /// Aircraft tail number (from configuration).
    pub tail_number: String,
    /// Pilot name (from configuration).
    pub pilot_name: String,
    /// Aircraft type (from configuration).
    pub aircraft_type: String,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const META_PAYLOAD_LEN: usize = 54;
const META_TOTAL_LEN: usize = 2 + 1 + 1 + META_PAYLOAD_LEN + 2;

const PAYLOAD_V0: usize = 143;
const PAYLOAD_V1: usize = 155;
const PAYLOAD_V2: usize = 184;
const FRAME_OVERHEAD: usize = 2 + 1 + 1 + 2; // BF + version + length + checksum

// ---------------------------------------------------------------------------
// Parse
// ---------------------------------------------------------------------------

/// Parse an `.onflight` binary data log.
pub fn parse(data: &[u8], opts: &ParseOptions) -> Result<OnFlightFile, ParseError> {
    if data.len() < META_TOTAL_LEN {
        return Err(ParseError::FileTooSmall);
    }
    if &data[0..2] != b"MD" {
        return Err(ParseError::BadMagic);
    }

    // Meta data
    let mut serial = [0u8; 6];
    serial.copy_from_slice(&data[4..10]);
    let header = FileHeader {
        version: data[2],
        serial_number: serial,
        tail_number: read_cstring(&data[10..22]),
        pilot_name: read_cstring(&data[22..46]),
        aircraft_type: read_cstring(&data[46..58]),
    };

    // Find first data frame
    let frame_start = find_first_frame(data).ok_or(ParseError::NoDataFrames)?;
    let version = data[frame_start + 2];
    let payload_len = match version {
        0 => PAYLOAD_V0,
        1 => PAYLOAD_V1,
        2 => PAYLOAD_V2,
        v => return Err(ParseError::UnsupportedVersion { version: v }),
    };
    let frame_size = FRAME_OVERHEAD + payload_len;

    // Parse frames
    let data_section = &data[frame_start..];
    let max_frames = data_section.len() / frame_size;
    let mut frames = Vec::with_capacity(max_frames);
    let mut checksum_errors = 0;

    for i in 0..max_frames {
        let raw = &data_section[i * frame_size..(i + 1) * frame_size];

        if raw[0] != b'B' || raw[1] != b'F' {
            break;
        }

        if opts.validate_checksums {
            let ck_offset = frame_size - 2;
            let expected = u16::from_le_bytes([raw[ck_offset], raw[ck_offset + 1]]);
            let computed = fletcher16(&raw[..ck_offset]);
            if expected != computed {
                checksum_errors += 1;
                if opts.skip_bad_checksums {
                    continue;
                }
            }
        }

        let p = &raw[4..];
        frames.push(parse_frame(p, version));
    }

    if frames.is_empty() {
        return Err(ParseError::NoDataFrames);
    }

    Ok(OnFlightFile { header, frames, checksum_errors })
}

// ---------------------------------------------------------------------------
// Internal
// ---------------------------------------------------------------------------

fn find_first_frame(data: &[u8]) -> Option<usize> {
    let start = if data.len() > META_TOTAL_LEN { META_TOTAL_LEN } else { 0 };
    for i in start..data.len().saturating_sub(4) {
        if data[i] == b'B' && data[i + 1] == b'F' {
            let version = data[i + 2];
            let payload_len = data[i + 3] as usize;
            let expected = match version {
                0 => PAYLOAD_V0,
                1 => PAYLOAD_V1,
                2 => PAYLOAD_V2,
                _ => continue,
            };
            if payload_len == expected {
                return Some(i);
            }
        }
    }
    None
}

fn read_cstring(data: &[u8]) -> String {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..end]).to_string()
}

fn u16_le(data: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([data[off], data[off + 1]])
}

fn i16_le(data: &[u8], off: usize) -> i16 {
    i16::from_le_bytes([data[off], data[off + 1]])
}

fn u32_le(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(data[off..off + 4].try_into().unwrap())
}

fn i32_le(data: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(data[off..off + 4].try_into().unwrap())
}

fn parse_frame(p: &[u8], version: u8) -> Frame {
    let mut status = [0u8; 6];
    status.copy_from_slice(&p[0..6]);

    let mut f = Frame {
        status,
        sys_time_ms: u32_le(p, 6),
        input_volt: p[10],
        filt_input_volt: p[11],
        cpu_die_temp_c: p[12] as i8,
        imu_die_temp_c: p[13] as i8,
        imu_accel_x: i16_le(p, 14),
        imu_accel_y: i16_le(p, 16),
        imu_accel_z: i16_le(p, 18),
        imu_gyro_x: i16_le(p, 20),
        imu_gyro_y: i16_le(p, 22),
        imu_gyro_z: i16_le(p, 24),
        mag_die_temp_c: p[26] as i8,
        mag_x: i16_le(p, 27),
        mag_y: i16_le(p, 29),
        mag_z: i16_le(p, 31),
        pres_die_temp_c: p[33] as i8,
        pres: u16_le(p, 34),
        gnss_fix: p[36] & 0x07,
        gnss_num_sv: p[36] >> 3,
        gnss_utc_year: p[37] as u16 + 1970,
        gnss_utc_month: p[38],
        gnss_utc_day: p[39],
        gnss_utc_hour: p[40],
        gnss_utc_min: p[41],
        gnss_utc_sec: p[42],
        gnss_horz_pos_acc: p[43],
        gnss_vert_pos_acc: p[44],
        gnss_vel_acc: p[45],
        gnss_ned_vel_x: i16_le(p, 46),
        gnss_ned_vel_y: i16_le(p, 48),
        gnss_ned_vel_z: i16_le(p, 50),
        gnss_alt_wgs84: u16_le(p, 52),
        gnss_geoid_height: i16_le(p, 54),
        gnss_lat: i32_le(p, 56),
        gnss_lon: i32_le(p, 60),
        ins_pitch: i16_le(p, 64),
        ins_roll: i16_le(p, 66),
        ins_mag_var: i16_le(p, 68),
        ins_heading_true: u16_le(p, 70),
        ins_heading_mag: u16_le(p, 72),
        ins_climb_rate: i16_le(p, 74),
        ins_load_factor: i16_le(p, 76),
        ins_accel_x: i16_le(p, 78),
        ins_accel_y: i16_le(p, 80),
        ins_accel_z: i16_le(p, 82),
        ins_gyro_x: i16_le(p, 84),
        ins_gyro_y: i16_le(p, 86),
        ins_gyro_z: i16_le(p, 88),
        ins_mag_x: i16_le(p, 90),
        ins_mag_y: i16_le(p, 92),
        ins_mag_z: i16_le(p, 94),
        ins_ned_vel_x: i16_le(p, 96),
        ins_ned_vel_y: i16_le(p, 98),
        ins_ned_vel_z: i16_le(p, 100),
        ins_gnd_spd: u16_le(p, 102),
        ins_gnd_track_true: u16_le(p, 104),
        ins_gnd_track_mag: u16_le(p, 106),
        ins_flt_path: i16_le(p, 108),
        ins_alt_wgs84: u16_le(p, 110),
        ins_lat: i32_le(p, 112),
        ins_lon: i32_le(p, 116),
        adc_pres: u16_le(p, 120),
        adc_pres_alt: u16_le(p, 122),
        ..Frame::default()
    };

    if version >= 1 && p.len() >= PAYLOAD_V1 {
        f.airdata_die_temp_c = p[124] as i8;
        f.airdata_static_pres = u16_le(p, 125);
        f.airdata_diff_pres = u16_le(p, 127);
        f.airdata_oat = i16_le(p, 129);
        f.airdata_ias = u16_le(p, 131);
        f.airdata_cas = u16_le(p, 133);
        f.airdata_tas = u16_le(p, 135);
        f.airdata_pres_alt = u16_le(p, 137);
        f.airdata_density_alt = u16_le(p, 139);
        f.airdata_aoa = i16_le(p, 141);
        f.airdata_wind_spd = u16_le(p, 143);
        f.airdata_wind_dir_true = u16_le(p, 145);
        f.airdata_wind_dir_mag = u16_le(p, 147);
    }

    if version >= 2 && p.len() >= PAYLOAD_V2 {
        f.agl_die_temp_c = p[149] as i8;
        f.agl_alt_in = i16_le(p, 150);
        f.heart_rate_bpm = p[152];
    }

    f
}
