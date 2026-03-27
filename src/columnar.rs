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
//!     println!("t={:.1}s spd={:.1}kts", cols.sys_time_s(i), cols.ins_gnd_spd_kts(i));
//! }
//! ```

use crate::{OnFlightFile, FileHeader, Frame};

/// Columnar flight data storing raw encoded values.
/// Field names match the Bolder Flight Systems dev manual.
/// Use accessor methods to get engineering units.
#[derive(Debug, Clone)]
pub struct Columns {
    pub header: FileHeader,
    pub len: usize,

    /// Per-frame status bitfield (6 bytes per frame)
    pub status: Vec<[u8; 6]>,

    // System
    pub sys_time_ms: Vec<u32>,
    pub input_volt: Vec<u8>,          // /25 → V
    pub filt_input_volt: Vec<u8>,     // /25 → V
    pub cpu_die_temp_c: Vec<i8>,
    pub imu_die_temp_c: Vec<i8>,
    pub mag_die_temp_c: Vec<i8>,
    pub pres_die_temp_c: Vec<i8>,

    // IMU (raw, body frame)
    pub imu_accel_x: Vec<i16>,       // /1000 → G
    pub imu_accel_y: Vec<i16>,
    pub imu_accel_z: Vec<i16>,
    pub imu_gyro_x: Vec<i16>,        // /10 → deg/s
    pub imu_gyro_y: Vec<i16>,
    pub imu_gyro_z: Vec<i16>,

    // Magnetometer
    pub mag_x: Vec<i16>,             // /80 → µT
    pub mag_y: Vec<i16>,
    pub mag_z: Vec<i16>,

    // Static pressure
    pub pres: Vec<u16>,              // *2 → Pa

    // GNSS
    pub gnss_fix: Vec<u8>,
    pub gnss_num_sv: Vec<u8>,
    pub gnss_utc_year: Vec<u16>,
    pub gnss_utc_month: Vec<u8>,
    pub gnss_utc_day: Vec<u8>,
    pub gnss_utc_hour: Vec<u8>,
    pub gnss_utc_min: Vec<u8>,
    pub gnss_utc_sec: Vec<u8>,
    pub gnss_horz_pos_acc: Vec<u8>,   // /10 → ft
    pub gnss_vert_pos_acc: Vec<u8>,   // /10 → ft
    pub gnss_vel_acc: Vec<u8>,        // /10 → kts
    pub gnss_ned_vel_x: Vec<i16>,     // /10 → kts
    pub gnss_ned_vel_y: Vec<i16>,
    pub gnss_ned_vel_z: Vec<i16>,     // /100 → kts
    pub gnss_alt_wgs84: Vec<u16>,     // -10000 → ft
    pub gnss_geoid_height: Vec<i16>,  // /10 → ft
    pub gnss_lat: Vec<i32>,           // /1e7 → deg
    pub gnss_lon: Vec<i32>,

    // INS (EKF-fused)
    pub ins_pitch: Vec<i16>,          // /100 → deg (+up)
    pub ins_roll: Vec<i16>,           // /100 → deg (+right)
    pub ins_mag_var: Vec<i16>,        // /100 → deg (+east)
    pub ins_heading_true: Vec<u16>,   // /100 → deg (0-360)
    pub ins_heading_mag: Vec<u16>,    // /100 → deg (0-360)
    pub ins_climb_rate: Vec<i16>,     // 1:1 → ft/min
    pub ins_load_factor: Vec<i16>,    // /1000 → G
    pub ins_accel_x: Vec<i16>,        // /1000 → G
    pub ins_accel_y: Vec<i16>,
    pub ins_accel_z: Vec<i16>,
    pub ins_gyro_x: Vec<i16>,         // /10 → deg/s
    pub ins_gyro_y: Vec<i16>,
    pub ins_gyro_z: Vec<i16>,
    pub ins_mag_x: Vec<i16>,          // /80 → µT
    pub ins_mag_y: Vec<i16>,
    pub ins_mag_z: Vec<i16>,
    pub ins_ned_vel_x: Vec<i16>,      // /10 → kts
    pub ins_ned_vel_y: Vec<i16>,
    pub ins_ned_vel_z: Vec<i16>,      // /100 → kts
    pub ins_gnd_spd: Vec<u16>,        // /100 → kts
    pub ins_gnd_track_true: Vec<u16>, // /100 → deg (0-360)
    pub ins_gnd_track_mag: Vec<u16>,  // /100 → deg (0-360)
    pub ins_flt_path: Vec<i16>,       // /100 → deg
    pub ins_alt_wgs84: Vec<u16>,      // -10000 → ft
    pub ins_lat: Vec<i32>,            // /1e7 → deg
    pub ins_lon: Vec<i32>,

    // ADC
    pub adc_pres: Vec<u16>,           // *2 → Pa
    pub adc_pres_alt: Vec<u16>,       // -10000 → ft

    // External airdata (v1+)
    pub airdata_die_temp_c: Vec<i8>,
    pub airdata_static_pres: Vec<u16>,
    pub airdata_diff_pres: Vec<u16>,
    pub airdata_oat: Vec<i16>,         // /100 → °C
    pub airdata_ias: Vec<u16>,         // /100 → kts
    pub airdata_cas: Vec<u16>,         // /100 → kts
    pub airdata_tas: Vec<u16>,         // /100 → kts
    pub airdata_pres_alt: Vec<u16>,    // -10000 → ft
    pub airdata_density_alt: Vec<u16>, // -10000 → ft
    pub airdata_aoa: Vec<i16>,         // /100
    pub airdata_wind_spd: Vec<u16>,    // /100 → kts
    pub airdata_wind_dir_true: Vec<u16>, // /100 → deg
    pub airdata_wind_dir_mag: Vec<u16>,  // /100 → deg

    // External AGL altimeter (v2)
    pub agl_die_temp_c: Vec<i8>,
    pub agl_alt_in: Vec<i16>,         // 1:1 → inches

    // Heart rate (v2)
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
        macro_rules! v {
            () => { Vec::with_capacity(n) };
        }
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

    // -- Accessors: engineering units --

    pub fn sys_time_s(&self, i: usize) -> f64 { self.sys_time_ms[i] as f64 / 1000.0 }
    pub fn input_volt_v(&self, i: usize) -> f32 { self.input_volt[i] as f32 / 25.0 }
    pub fn filt_input_volt_v(&self, i: usize) -> f32 { self.filt_input_volt[i] as f32 / 25.0 }

    // Status
    pub fn ins_initialized(&self, i: usize) -> bool { self.status[i][1] & 0x40 != 0 }
    pub fn ins_healthy(&self, i: usize) -> bool { self.status[i][1] & 0x80 != 0 }
    pub fn gnss_new_data(&self, i: usize) -> bool { self.status[i][1] & 0x10 != 0 }
    pub fn gnss_healthy(&self, i: usize) -> bool { self.status[i][1] & 0x20 != 0 }
    pub fn imu_healthy(&self, i: usize) -> bool { self.status[i][0] & 0x10 != 0 }
    pub fn airdata_connected(&self, i: usize) -> bool { self.status[i][2] & 0x02 != 0 }
    pub fn agl_connected(&self, i: usize) -> bool { self.status[i][4] & 0x02 != 0 }

    // IMU
    pub fn imu_accel_x_g(&self, i: usize) -> f32 { self.imu_accel_x[i] as f32 / 1000.0 }
    pub fn imu_accel_y_g(&self, i: usize) -> f32 { self.imu_accel_y[i] as f32 / 1000.0 }
    pub fn imu_accel_z_g(&self, i: usize) -> f32 { self.imu_accel_z[i] as f32 / 1000.0 }
    pub fn imu_gyro_x_dps(&self, i: usize) -> f32 { self.imu_gyro_x[i] as f32 / 10.0 }
    pub fn imu_gyro_y_dps(&self, i: usize) -> f32 { self.imu_gyro_y[i] as f32 / 10.0 }
    pub fn imu_gyro_z_dps(&self, i: usize) -> f32 { self.imu_gyro_z[i] as f32 / 10.0 }

    // Magnetometer
    pub fn mag_x_ut(&self, i: usize) -> f32 { self.mag_x[i] as f32 / 80.0 }
    pub fn mag_y_ut(&self, i: usize) -> f32 { self.mag_y[i] as f32 / 80.0 }
    pub fn mag_z_ut(&self, i: usize) -> f32 { self.mag_z[i] as f32 / 80.0 }

    // Pressure
    pub fn pres_pa(&self, i: usize) -> u32 { self.pres[i] as u32 * 2 }

    // GNSS
    pub fn gnss_horz_acc_ft(&self, i: usize) -> f32 { self.gnss_horz_pos_acc[i] as f32 / 10.0 }
    pub fn gnss_vert_acc_ft(&self, i: usize) -> f32 { self.gnss_vert_pos_acc[i] as f32 / 10.0 }
    pub fn gnss_vel_acc_kts(&self, i: usize) -> f32 { self.gnss_vel_acc[i] as f32 / 10.0 }
    pub fn gnss_north_vel_kts(&self, i: usize) -> f32 { self.gnss_ned_vel_x[i] as f32 / 10.0 }
    pub fn gnss_east_vel_kts(&self, i: usize) -> f32 { self.gnss_ned_vel_y[i] as f32 / 10.0 }
    pub fn gnss_down_vel_kts(&self, i: usize) -> f32 { self.gnss_ned_vel_z[i] as f32 / 100.0 }
    pub fn gnss_alt_wgs84_ft(&self, i: usize) -> i32 { self.gnss_alt_wgs84[i] as i32 - 10000 }
    pub fn gnss_geoid_height_ft(&self, i: usize) -> f32 { self.gnss_geoid_height[i] as f32 / 10.0 }
    pub fn gnss_alt_msl_ft(&self, i: usize) -> f32 { self.gnss_alt_wgs84_ft(i) as f32 - self.gnss_geoid_height_ft(i) }
    pub fn gnss_lat_deg(&self, i: usize) -> f64 { self.gnss_lat[i] as f64 / 1e7 }
    pub fn gnss_lon_deg(&self, i: usize) -> f64 { self.gnss_lon[i] as f64 / 1e7 }

    // INS
    pub fn ins_pitch_deg(&self, i: usize) -> f32 { self.ins_pitch[i] as f32 / 100.0 }
    pub fn ins_roll_deg(&self, i: usize) -> f32 { self.ins_roll[i] as f32 / 100.0 }
    pub fn ins_mag_var_deg(&self, i: usize) -> f32 { self.ins_mag_var[i] as f32 / 100.0 }
    pub fn ins_heading_true_deg(&self, i: usize) -> f32 { self.ins_heading_true[i] as f32 / 100.0 }
    pub fn ins_heading_mag_deg(&self, i: usize) -> f32 { self.ins_heading_mag[i] as f32 / 100.0 }
    pub fn ins_climb_rate_fpm(&self, i: usize) -> i16 { self.ins_climb_rate[i] }
    pub fn ins_load_factor_g(&self, i: usize) -> f32 { self.ins_load_factor[i] as f32 / 1000.0 }
    pub fn ins_accel_x_g(&self, i: usize) -> f32 { self.ins_accel_x[i] as f32 / 1000.0 }
    pub fn ins_accel_y_g(&self, i: usize) -> f32 { self.ins_accel_y[i] as f32 / 1000.0 }
    pub fn ins_accel_z_g(&self, i: usize) -> f32 { self.ins_accel_z[i] as f32 / 1000.0 }
    pub fn ins_gyro_x_dps(&self, i: usize) -> f32 { self.ins_gyro_x[i] as f32 / 10.0 }
    pub fn ins_gyro_y_dps(&self, i: usize) -> f32 { self.ins_gyro_y[i] as f32 / 10.0 }
    pub fn ins_gyro_z_dps(&self, i: usize) -> f32 { self.ins_gyro_z[i] as f32 / 10.0 }
    pub fn ins_mag_x_ut(&self, i: usize) -> f32 { self.ins_mag_x[i] as f32 / 80.0 }
    pub fn ins_mag_y_ut(&self, i: usize) -> f32 { self.ins_mag_y[i] as f32 / 80.0 }
    pub fn ins_mag_z_ut(&self, i: usize) -> f32 { self.ins_mag_z[i] as f32 / 80.0 }
    pub fn ins_north_vel_kts(&self, i: usize) -> f32 { self.ins_ned_vel_x[i] as f32 / 10.0 }
    pub fn ins_east_vel_kts(&self, i: usize) -> f32 { self.ins_ned_vel_y[i] as f32 / 10.0 }
    pub fn ins_down_vel_kts(&self, i: usize) -> f32 { self.ins_ned_vel_z[i] as f32 / 100.0 }
    pub fn ins_gnd_spd_kts(&self, i: usize) -> f32 { self.ins_gnd_spd[i] as f32 / 100.0 }
    pub fn ins_gnd_track_true_deg(&self, i: usize) -> f32 { self.ins_gnd_track_true[i] as f32 / 100.0 }
    pub fn ins_gnd_track_mag_deg(&self, i: usize) -> f32 { self.ins_gnd_track_mag[i] as f32 / 100.0 }
    pub fn ins_flt_path_deg(&self, i: usize) -> f32 { self.ins_flt_path[i] as f32 / 100.0 }
    pub fn ins_alt_wgs84_ft(&self, i: usize) -> i32 { self.ins_alt_wgs84[i] as i32 - 10000 }
    pub fn ins_alt_msl_ft(&self, i: usize) -> f32 { self.ins_alt_wgs84_ft(i) as f32 - self.gnss_geoid_height_ft(i) }
    pub fn ins_lat_deg(&self, i: usize) -> f64 { self.ins_lat[i] as f64 / 1e7 }
    pub fn ins_lon_deg(&self, i: usize) -> f64 { self.ins_lon[i] as f64 / 1e7 }

    // ADC
    pub fn adc_pres_pa(&self, i: usize) -> u32 { self.adc_pres[i] as u32 * 2 }
    pub fn adc_pres_alt_ft(&self, i: usize) -> i32 { self.adc_pres_alt[i] as i32 - 10000 }

    // External airdata
    pub fn airdata_ias_kts(&self, i: usize) -> f32 { self.airdata_ias[i] as f32 / 100.0 }
    pub fn airdata_cas_kts(&self, i: usize) -> f32 { self.airdata_cas[i] as f32 / 100.0 }
    pub fn airdata_tas_kts(&self, i: usize) -> f32 { self.airdata_tas[i] as f32 / 100.0 }
    pub fn airdata_oat_c(&self, i: usize) -> f32 { self.airdata_oat[i] as f32 / 100.0 }
    pub fn airdata_wind_spd_kts(&self, i: usize) -> f32 { self.airdata_wind_spd[i] as f32 / 100.0 }
    pub fn airdata_wind_dir_true_deg(&self, i: usize) -> f32 { self.airdata_wind_dir_true[i] as f32 / 100.0 }
    pub fn airdata_pres_alt_ft(&self, i: usize) -> i32 { self.airdata_pres_alt[i] as i32 - 10000 }
    pub fn airdata_density_alt_ft(&self, i: usize) -> i32 { self.airdata_density_alt[i] as i32 - 10000 }

    // AGL
    pub fn agl_alt_ft(&self, i: usize) -> f32 { self.agl_alt_in[i] as f32 / 12.0 }

    // -- Convenience --

    /// Duration in seconds.
    pub fn duration_s(&self) -> f64 {
        if self.len > 0 { self.sys_time_s(self.len - 1) - self.sys_time_s(0) } else { 0.0 }
    }

    /// Time offset from start of recording for sample i.
    pub fn t(&self, i: usize) -> f64 {
        self.sys_time_s(i) - self.sys_time_s(0)
    }

    /// Index of first sample with valid GNSS fix (>= 3D).
    pub fn first_fix_index(&self) -> Option<usize> {
        (0..self.len).find(|&i| self.gnss_fix[i] >= 3)
    }
}
