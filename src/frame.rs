//! Data frame definition matching the OnFlight Hub Developers Manual.
//!
//! All fields store raw encoded values. Use the scale factors documented
//! in the manual (and noted in comments) to convert to engineering units.
//!
//! Field names match the dev manual. Types use Rust equivalents:
//! U1→u8, I1→i8, U2→u16, I2→i16, U4→u32, I4→i32, F4→f32.

/// A single 50 Hz data frame from the OnFlight Hub.
///
/// Fields are raw encoded values as stored in the binary format.
/// See the OnFlight Hub Developers Manual for scale factors and units.
#[derive(Debug, Clone, Default)]
pub struct Frame {
    // -- Status (6 bytes) --
    /// Per-frame status bitfield. See manual Section 3.3.
    pub status: [u8; 6],

    // -- System --
    /// Time since boot in milliseconds.
    pub sys_time_ms: u32,
    /// Input voltage, raw. Divide by 25 for volts.
    pub input_volt: u8,
    /// Filtered input voltage, raw. Divide by 25 for volts.
    pub filt_input_volt: u8,
    /// CPU die temperature in °C.
    pub cpu_die_temp_c: i8,

    // -- IMU (uncompensated, unfiltered, body frame) --
    /// IMU die temperature in °C.
    pub imu_die_temp_c: i8,
    /// X-axis accelerometer. Divide by 1000 for G.
    pub imu_accel_x: i16,
    /// Y-axis accelerometer. Divide by 1000 for G.
    pub imu_accel_y: i16,
    /// Z-axis accelerometer. Divide by 1000 for G.
    pub imu_accel_z: i16,
    /// X-axis gyro. Divide by 10 for deg/s.
    pub imu_gyro_x: i16,
    /// Y-axis gyro. Divide by 10 for deg/s.
    pub imu_gyro_y: i16,
    /// Z-axis gyro. Divide by 10 for deg/s.
    pub imu_gyro_z: i16,

    // -- Magnetometer --
    /// Magnetometer die temperature in °C.
    pub mag_die_temp_c: i8,
    /// X-axis magnetometer. Divide by 80 for µT.
    pub mag_x: i16,
    /// Y-axis magnetometer. Divide by 80 for µT.
    pub mag_y: i16,
    /// Z-axis magnetometer. Divide by 80 for µT.
    pub mag_z: i16,

    // -- Static pressure --
    /// Static pressure die temperature in °C.
    pub pres_die_temp_c: i8,
    /// Static pressure, raw. Multiply by 2 for Pa.
    pub pres: u16,

    // -- GNSS --
    /// GNSS fix type (0=none, 2=2D, 3=3D, 4=differential).
    pub gnss_fix: u8,
    /// Number of satellite vehicles used in solution.
    pub gnss_num_sv: u8,
    /// UTC year (absolute, e.g. 2026).
    pub gnss_utc_year: u16,
    pub gnss_utc_month: u8,
    pub gnss_utc_day: u8,
    pub gnss_utc_hour: u8,
    pub gnss_utc_min: u8,
    pub gnss_utc_sec: u8,
    /// Estimated horizontal position accuracy. Divide by 10 for ft.
    pub gnss_horz_pos_acc: u8,
    /// Estimated vertical position accuracy. Divide by 10 for ft.
    pub gnss_vert_pos_acc: u8,
    /// Estimated velocity accuracy. Divide by 10 for kts.
    pub gnss_vel_acc: u8,
    /// GNSS North velocity. Divide by 10 for kts.
    pub gnss_ned_vel_x: i16,
    /// GNSS East velocity. Divide by 10 for kts.
    pub gnss_ned_vel_y: i16,
    /// GNSS Down velocity. Divide by 100 for kts.
    pub gnss_ned_vel_z: i16,
    /// GNSS altitude above WGS84 ellipsoid, biased +10000. Subtract 10000 for ft.
    pub gnss_alt_wgs84: u16,
    /// GNSS geoid height. Divide by 10 for ft. MSL = WGS84 - geoid.
    pub gnss_geoid_height: i16,
    /// GNSS latitude. Divide by 1e7 for degrees.
    pub gnss_lat: i32,
    /// GNSS longitude. Divide by 1e7 for degrees.
    pub gnss_lon: i32,

    // -- INS (EKF-compensated, filtered) --
    /// Pitch angle (+up). Divide by 100 for degrees.
    pub ins_pitch: i16,
    /// Roll angle (+right). Divide by 100 for degrees.
    pub ins_roll: i16,
    /// Magnetic variation (+east). Divide by 100 for degrees.
    pub ins_mag_var: i16,
    /// True heading (0-360). Divide by 100 for degrees.
    pub ins_heading_true: u16,
    /// Magnetic heading (0-360). Divide by 100 for degrees.
    pub ins_heading_mag: u16,
    /// Climb rate in ft/min (1:1 scale).
    pub ins_climb_rate: i16,
    /// Load factor. Divide by 1000 for G.
    pub ins_load_factor: i16,
    /// INS X-axis accelerometer (+forward). Divide by 1000 for G.
    pub ins_accel_x: i16,
    /// INS Y-axis accelerometer (+right). Divide by 1000 for G.
    pub ins_accel_y: i16,
    /// INS Z-axis accelerometer (+down). Divide by 1000 for G.
    pub ins_accel_z: i16,
    /// INS X-axis gyro. Divide by 10 for deg/s.
    pub ins_gyro_x: i16,
    /// INS Y-axis gyro. Divide by 10 for deg/s.
    pub ins_gyro_y: i16,
    /// INS Z-axis gyro. Divide by 10 for deg/s.
    pub ins_gyro_z: i16,
    /// INS X-axis magnetometer. Divide by 80 for µT.
    pub ins_mag_x: i16,
    /// INS Y-axis magnetometer. Divide by 80 for µT.
    pub ins_mag_y: i16,
    /// INS Z-axis magnetometer. Divide by 80 for µT.
    pub ins_mag_z: i16,
    /// INS North velocity. Divide by 10 for kts.
    pub ins_ned_vel_x: i16,
    /// INS East velocity. Divide by 10 for kts.
    pub ins_ned_vel_y: i16,
    /// INS Down velocity. Divide by 100 for kts.
    pub ins_ned_vel_z: i16,
    /// INS ground speed. Divide by 100 for kts.
    pub ins_gnd_spd: u16,
    /// INS ground track, true (0-360). Divide by 100 for degrees.
    pub ins_gnd_track_true: u16,
    /// INS ground track, magnetic (0-360). Divide by 100 for degrees.
    pub ins_gnd_track_mag: u16,
    /// INS flight path angle (+/- 90). Divide by 100 for degrees.
    pub ins_flt_path: i16,
    /// INS altitude above WGS84 ellipsoid, biased +10000. Subtract 10000 for ft.
    pub ins_alt_wgs84: u16,
    /// INS latitude. Divide by 1e7 for degrees.
    pub ins_lat: i32,
    /// INS longitude. Divide by 1e7 for degrees.
    pub ins_lon: i32,

    // -- ADC (internal air data computer) --
    /// ADC static pressure, raw. Multiply by 2 for Pa.
    pub adc_pres: u16,
    /// ADC pressure altitude, biased +10000. Subtract 10000 for ft.
    pub adc_pres_alt: u16,

    // -- External airdata (firmware v1+, zero when not connected) --
    /// External airdata board die temperature in °C.
    pub airdata_die_temp_c: i8,
    /// External airdata static pressure. Multiply by 2 for Pa.
    pub airdata_static_pres: u16,
    /// External airdata differential pressure in Pa (1:1 scale).
    pub airdata_diff_pres: u16,
    /// External airdata OAT. Divide by 100 for °C.
    pub airdata_oat: i16,
    /// External airdata IAS. Divide by 100 for kts.
    pub airdata_ias: u16,
    /// External airdata CAS. Divide by 100 for kts.
    pub airdata_cas: u16,
    /// External airdata TAS. Divide by 100 for kts.
    pub airdata_tas: u16,
    /// External airdata pressure altitude, biased +10000. Subtract 10000 for ft.
    pub airdata_pres_alt: u16,
    /// External airdata density altitude, biased +10000. Subtract 10000 for ft.
    pub airdata_density_alt: u16,
    /// External airdata AOA. Divide by 100 for degrees (or pressure ratio per status bit).
    pub airdata_aoa: i16,
    /// External airdata wind speed. Divide by 100 for kts.
    pub airdata_wind_spd: u16,
    /// External airdata wind direction, true. Divide by 100 for degrees.
    pub airdata_wind_dir_true: u16,
    /// External airdata wind direction, magnetic. Divide by 100 for degrees.
    pub airdata_wind_dir_mag: u16,

    // -- External AGL altimeter (firmware v2, zero when not connected) --
    /// AGL altimeter board die temperature in °C.
    pub agl_die_temp_c: i8,
    /// AGL altitude in inches (1:1 scale). Divide by 12 for feet.
    pub agl_alt_in: i16,

    // -- Heart rate (firmware v2, zero when not connected) --
    /// Heart rate in BPM.
    pub heart_rate_bpm: u8,
}
