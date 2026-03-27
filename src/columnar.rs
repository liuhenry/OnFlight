//! Columnar (struct-of-arrays) representation of OnFlight data.
//!
//! [`Columns`] stores each field as a contiguous array, transposed from
//! the row-oriented [`Frame`] representation. This layout is efficient
//! for time-series analysis where you iterate one field across many samples.
//!
//! ```ignore
//! let file = onflight::parse(&data, &onflight::ParseOptions::default())?;
//! let cols = onflight::Columns::from_file(file);
//! for i in 0..cols.len {
//!     let speed_kts = cols.ins_gnd_spd[i] as f32 / 100.0;
//!     println!("t={}ms  {:.1} kts", cols.sys_time_ms[i], speed_kts);
//! }
//! ```

use crate::{OnFlightFile, FileHeader, Frame};

/// Columnar flight data storing raw encoded values.
/// Field names and types match the `Frame` struct.
/// Scale factors are documented on the corresponding `Frame` fields.
#[derive(Debug, Clone)]
pub struct Columns {
    pub header: FileHeader,
    pub len: usize,

    pub status: Vec<[u8; 6]>,

    // System
    pub sys_time_ms: Vec<u32>,
    pub input_volt: Vec<u8>,
    pub filt_input_volt: Vec<u8>,
    pub cpu_die_temp_c: Vec<i8>,
    pub imu_die_temp_c: Vec<i8>,
    pub mag_die_temp_c: Vec<i8>,
    pub pres_die_temp_c: Vec<i8>,

    // IMU
    pub imu_accel_x: Vec<i16>,
    pub imu_accel_y: Vec<i16>,
    pub imu_accel_z: Vec<i16>,
    pub imu_gyro_x: Vec<i16>,
    pub imu_gyro_y: Vec<i16>,
    pub imu_gyro_z: Vec<i16>,

    // Magnetometer
    pub mag_x: Vec<i16>,
    pub mag_y: Vec<i16>,
    pub mag_z: Vec<i16>,

    // Static pressure
    pub pres: Vec<u16>,

    // GNSS
    pub gnss_fix: Vec<u8>,
    pub gnss_num_sv: Vec<u8>,
    pub gnss_utc_year: Vec<u16>,
    pub gnss_utc_month: Vec<u8>,
    pub gnss_utc_day: Vec<u8>,
    pub gnss_utc_hour: Vec<u8>,
    pub gnss_utc_min: Vec<u8>,
    pub gnss_utc_sec: Vec<u8>,
    pub gnss_horz_pos_acc: Vec<u8>,
    pub gnss_vert_pos_acc: Vec<u8>,
    pub gnss_vel_acc: Vec<u8>,
    pub gnss_ned_vel_x: Vec<i16>,
    pub gnss_ned_vel_y: Vec<i16>,
    pub gnss_ned_vel_z: Vec<i16>,
    pub gnss_alt_wgs84: Vec<u16>,
    pub gnss_geoid_height: Vec<i16>,
    pub gnss_lat: Vec<i32>,
    pub gnss_lon: Vec<i32>,

    // INS
    pub ins_pitch: Vec<i16>,
    pub ins_roll: Vec<i16>,
    pub ins_mag_var: Vec<i16>,
    pub ins_heading_true: Vec<u16>,
    pub ins_heading_mag: Vec<u16>,
    pub ins_climb_rate: Vec<i16>,
    pub ins_load_factor: Vec<i16>,
    pub ins_accel_x: Vec<i16>,
    pub ins_accel_y: Vec<i16>,
    pub ins_accel_z: Vec<i16>,
    pub ins_gyro_x: Vec<i16>,
    pub ins_gyro_y: Vec<i16>,
    pub ins_gyro_z: Vec<i16>,
    pub ins_mag_x: Vec<i16>,
    pub ins_mag_y: Vec<i16>,
    pub ins_mag_z: Vec<i16>,
    pub ins_ned_vel_x: Vec<i16>,
    pub ins_ned_vel_y: Vec<i16>,
    pub ins_ned_vel_z: Vec<i16>,
    pub ins_gnd_spd: Vec<u16>,
    pub ins_gnd_track_true: Vec<u16>,
    pub ins_gnd_track_mag: Vec<u16>,
    pub ins_flt_path: Vec<i16>,
    pub ins_alt_wgs84: Vec<u16>,
    pub ins_lat: Vec<i32>,
    pub ins_lon: Vec<i32>,

    // ADC
    pub adc_pres: Vec<u16>,
    pub adc_pres_alt: Vec<u16>,

    // External airdata
    pub airdata_die_temp_c: Vec<i8>,
    pub airdata_static_pres: Vec<u16>,
    pub airdata_diff_pres: Vec<u16>,
    pub airdata_oat: Vec<i16>,
    pub airdata_ias: Vec<u16>,
    pub airdata_cas: Vec<u16>,
    pub airdata_tas: Vec<u16>,
    pub airdata_pres_alt: Vec<u16>,
    pub airdata_density_alt: Vec<u16>,
    pub airdata_aoa: Vec<i16>,
    pub airdata_wind_spd: Vec<u16>,
    pub airdata_wind_dir_true: Vec<u16>,
    pub airdata_wind_dir_mag: Vec<u16>,

    // AGL
    pub agl_die_temp_c: Vec<i8>,
    pub agl_alt_in: Vec<i16>,

    // Heart rate
    pub heart_rate_bpm: Vec<u8>,
}

impl Columns {
    /// Transpose an [`OnFlightFile`] (row-oriented) into columnar storage.
    pub fn from_file(file: OnFlightFile) -> Self {
        let n = file.frames.len();
        let mut cols = Self::with_capacity(file.header, n);
        for f in &file.frames {
            cols.push_frame(f);
        }
        cols
    }

    fn with_capacity(header: FileHeader, n: usize) -> Self {
        macro_rules! v { () => { Vec::with_capacity(n) }; }
        Self {
            header, len: 0,
            status: v!(),
            sys_time_ms: v!(), input_volt: v!(), filt_input_volt: v!(),
            cpu_die_temp_c: v!(), imu_die_temp_c: v!(), mag_die_temp_c: v!(), pres_die_temp_c: v!(),
            imu_accel_x: v!(), imu_accel_y: v!(), imu_accel_z: v!(),
            imu_gyro_x: v!(), imu_gyro_y: v!(), imu_gyro_z: v!(),
            mag_x: v!(), mag_y: v!(), mag_z: v!(),
            pres: v!(),
            gnss_fix: v!(), gnss_num_sv: v!(),
            gnss_utc_year: v!(), gnss_utc_month: v!(), gnss_utc_day: v!(),
            gnss_utc_hour: v!(), gnss_utc_min: v!(), gnss_utc_sec: v!(),
            gnss_horz_pos_acc: v!(), gnss_vert_pos_acc: v!(), gnss_vel_acc: v!(),
            gnss_ned_vel_x: v!(), gnss_ned_vel_y: v!(), gnss_ned_vel_z: v!(),
            gnss_alt_wgs84: v!(), gnss_geoid_height: v!(), gnss_lat: v!(), gnss_lon: v!(),
            ins_pitch: v!(), ins_roll: v!(), ins_mag_var: v!(),
            ins_heading_true: v!(), ins_heading_mag: v!(),
            ins_climb_rate: v!(), ins_load_factor: v!(),
            ins_accel_x: v!(), ins_accel_y: v!(), ins_accel_z: v!(),
            ins_gyro_x: v!(), ins_gyro_y: v!(), ins_gyro_z: v!(),
            ins_mag_x: v!(), ins_mag_y: v!(), ins_mag_z: v!(),
            ins_ned_vel_x: v!(), ins_ned_vel_y: v!(), ins_ned_vel_z: v!(),
            ins_gnd_spd: v!(), ins_gnd_track_true: v!(), ins_gnd_track_mag: v!(),
            ins_flt_path: v!(), ins_alt_wgs84: v!(), ins_lat: v!(), ins_lon: v!(),
            adc_pres: v!(), adc_pres_alt: v!(),
            airdata_die_temp_c: v!(), airdata_static_pres: v!(), airdata_diff_pres: v!(),
            airdata_oat: v!(), airdata_ias: v!(), airdata_cas: v!(), airdata_tas: v!(),
            airdata_pres_alt: v!(), airdata_density_alt: v!(), airdata_aoa: v!(),
            airdata_wind_spd: v!(), airdata_wind_dir_true: v!(), airdata_wind_dir_mag: v!(),
            agl_die_temp_c: v!(), agl_alt_in: v!(),
            heart_rate_bpm: v!(),
        }
    }

    fn push_frame(&mut self, f: &Frame) {
        self.len += 1;
        self.status.push(f.status);
        self.sys_time_ms.push(f.sys_time_ms);
        self.input_volt.push(f.input_volt);
        self.filt_input_volt.push(f.filt_input_volt);
        self.cpu_die_temp_c.push(f.cpu_die_temp_c);
        self.imu_die_temp_c.push(f.imu_die_temp_c);
        self.mag_die_temp_c.push(f.mag_die_temp_c);
        self.pres_die_temp_c.push(f.pres_die_temp_c);
        self.imu_accel_x.push(f.imu_accel_x);
        self.imu_accel_y.push(f.imu_accel_y);
        self.imu_accel_z.push(f.imu_accel_z);
        self.imu_gyro_x.push(f.imu_gyro_x);
        self.imu_gyro_y.push(f.imu_gyro_y);
        self.imu_gyro_z.push(f.imu_gyro_z);
        self.mag_x.push(f.mag_x);
        self.mag_y.push(f.mag_y);
        self.mag_z.push(f.mag_z);
        self.pres.push(f.pres);
        self.gnss_fix.push(f.gnss_fix);
        self.gnss_num_sv.push(f.gnss_num_sv);
        self.gnss_utc_year.push(f.gnss_utc_year);
        self.gnss_utc_month.push(f.gnss_utc_month);
        self.gnss_utc_day.push(f.gnss_utc_day);
        self.gnss_utc_hour.push(f.gnss_utc_hour);
        self.gnss_utc_min.push(f.gnss_utc_min);
        self.gnss_utc_sec.push(f.gnss_utc_sec);
        self.gnss_horz_pos_acc.push(f.gnss_horz_pos_acc);
        self.gnss_vert_pos_acc.push(f.gnss_vert_pos_acc);
        self.gnss_vel_acc.push(f.gnss_vel_acc);
        self.gnss_ned_vel_x.push(f.gnss_ned_vel_x);
        self.gnss_ned_vel_y.push(f.gnss_ned_vel_y);
        self.gnss_ned_vel_z.push(f.gnss_ned_vel_z);
        self.gnss_alt_wgs84.push(f.gnss_alt_wgs84);
        self.gnss_geoid_height.push(f.gnss_geoid_height);
        self.gnss_lat.push(f.gnss_lat);
        self.gnss_lon.push(f.gnss_lon);
        self.ins_pitch.push(f.ins_pitch);
        self.ins_roll.push(f.ins_roll);
        self.ins_mag_var.push(f.ins_mag_var);
        self.ins_heading_true.push(f.ins_heading_true);
        self.ins_heading_mag.push(f.ins_heading_mag);
        self.ins_climb_rate.push(f.ins_climb_rate);
        self.ins_load_factor.push(f.ins_load_factor);
        self.ins_accel_x.push(f.ins_accel_x);
        self.ins_accel_y.push(f.ins_accel_y);
        self.ins_accel_z.push(f.ins_accel_z);
        self.ins_gyro_x.push(f.ins_gyro_x);
        self.ins_gyro_y.push(f.ins_gyro_y);
        self.ins_gyro_z.push(f.ins_gyro_z);
        self.ins_mag_x.push(f.ins_mag_x);
        self.ins_mag_y.push(f.ins_mag_y);
        self.ins_mag_z.push(f.ins_mag_z);
        self.ins_ned_vel_x.push(f.ins_ned_vel_x);
        self.ins_ned_vel_y.push(f.ins_ned_vel_y);
        self.ins_ned_vel_z.push(f.ins_ned_vel_z);
        self.ins_gnd_spd.push(f.ins_gnd_spd);
        self.ins_gnd_track_true.push(f.ins_gnd_track_true);
        self.ins_gnd_track_mag.push(f.ins_gnd_track_mag);
        self.ins_flt_path.push(f.ins_flt_path);
        self.ins_alt_wgs84.push(f.ins_alt_wgs84);
        self.ins_lat.push(f.ins_lat);
        self.ins_lon.push(f.ins_lon);
        self.adc_pres.push(f.adc_pres);
        self.adc_pres_alt.push(f.adc_pres_alt);
        self.airdata_die_temp_c.push(f.airdata_die_temp_c);
        self.airdata_static_pres.push(f.airdata_static_pres);
        self.airdata_diff_pres.push(f.airdata_diff_pres);
        self.airdata_oat.push(f.airdata_oat);
        self.airdata_ias.push(f.airdata_ias);
        self.airdata_cas.push(f.airdata_cas);
        self.airdata_tas.push(f.airdata_tas);
        self.airdata_pres_alt.push(f.airdata_pres_alt);
        self.airdata_density_alt.push(f.airdata_density_alt);
        self.airdata_aoa.push(f.airdata_aoa);
        self.airdata_wind_spd.push(f.airdata_wind_spd);
        self.airdata_wind_dir_true.push(f.airdata_wind_dir_true);
        self.airdata_wind_dir_mag.push(f.airdata_wind_dir_mag);
        self.agl_die_temp_c.push(f.agl_die_temp_c);
        self.agl_alt_in.push(f.agl_alt_in);
        self.heart_rate_bpm.push(f.heart_rate_bpm);
    }
}
