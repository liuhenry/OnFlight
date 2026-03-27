use onflight::{self, ParseOptions};

const TEST_META: [u8; 60] = [
    0x4d, 0x44, 0x00, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x45, 0x53, 0x54, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x65, 0x73, 0x74, 0x20, 0x50, 0x69, 0x6c, 0x6f, 0x74,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x43, 0x31,
    0x37, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb2, 0xc9,
];

const TEST_FRAME: [u8; 190] = [
    0x42, 0x46, 0x02, 0xb8, 0xfc, 0xef, 0x00, 0x00, 0x00, 0x00, 0x26, 0xe4, 0x2d, 0x00, 0x62, 0x62,
    0x32, 0x36, 0xcc, 0xff, 0x28, 0x00, 0x9f, 0xfa, 0x1c, 0x00, 0x29, 0x00, 0xc8, 0xff, 0x31, 0x36,
    0xf9, 0x68, 0xea, 0xac, 0x09, 0x35, 0xc6, 0xb3, 0x94, 0x38, 0x03, 0x11, 0x15, 0x29, 0x02, 0x0d,
    0x18, 0x05, 0x5c, 0xfd, 0xb0, 0x03, 0x85, 0x04, 0x7c, 0x31, 0x3d, 0xfc, 0x60, 0x15, 0xb0, 0x16,
    0x87, 0xab, 0x02, 0xb7, 0x36, 0xff, 0x82, 0xf7, 0x0e, 0x05, 0xab, 0x32, 0x9d, 0x2d, 0x34, 0xfa,
    0x2c, 0x05, 0xe1, 0xff, 0xdb, 0xff, 0xc5, 0xfa, 0x46, 0x00, 0x22, 0x00, 0xc5, 0xff, 0xff, 0xf8,
    0x42, 0xea, 0x78, 0x09, 0x72, 0xfd, 0xb0, 0x03, 0xc5, 0x04, 0xe1, 0x2c, 0xb8, 0x30, 0xaa, 0x2b,
    0xa2, 0xfd, 0x7c, 0x31, 0x3d, 0x15, 0xb0, 0x16, 0x6e, 0xac, 0x02, 0xb7, 0xc1, 0xb3, 0x5d, 0x31,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x27, 0x10,
    0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xcb, 0xcc,
];

fn build_test_file(frames: &[&[u8]]) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&TEST_META);
    for frame in frames {
        data.extend_from_slice(frame);
    }
    data
}

fn parse_test(data: &[u8]) -> onflight::OnFlightFile {
    onflight::parse(data, &ParseOptions::default()).unwrap()
}

#[test]
fn test_fletcher16() {
    let ck_offset = 190 - 2;
    let expected = u16::from_le_bytes([TEST_FRAME[ck_offset], TEST_FRAME[ck_offset + 1]]);
    let computed = onflight::fletcher16(&TEST_FRAME[..ck_offset]);
    assert_eq!(expected, computed);
}

#[test]
fn test_meta_checksum() {
    let ck_offset = 60 - 2;
    let expected = u16::from_le_bytes([TEST_META[ck_offset], TEST_META[ck_offset + 1]]);
    let computed = onflight::fletcher16(&TEST_META[..ck_offset]);
    assert_eq!(expected, computed);
}

#[test]
fn test_parse_header() {
    let file = parse_test(&build_test_file(&[&TEST_FRAME]));
    assert_eq!(file.header.tail_number, "TEST");
    assert_eq!(file.header.pilot_name, "Test Pilot");
    assert_eq!(file.header.aircraft_type, "C172");
}

#[test]
fn test_parse_single_frame() {
    let file = parse_test(&build_test_file(&[&TEST_FRAME]));
    assert_eq!(file.frames.len(), 1);
    let f = &file.frames[0];

    assert_eq!(f.sys_time_ms, 3007526);
    assert_eq!(f.cpu_die_temp_c, 50);
    assert_eq!(f.imu_die_temp_c, 54);
    assert_eq!(f.mag_die_temp_c, 49);
    assert_eq!(f.pres_die_temp_c, 53);
    assert_eq!(f.imu_accel_x, -52);
    assert_eq!(f.imu_accel_y, 40);
    assert_eq!(f.imu_accel_z, -1377);
    assert_eq!(f.imu_gyro_x, 28);
    assert_eq!(f.imu_gyro_y, 41);
    assert_eq!(f.imu_gyro_z, -56);
    assert_eq!(f.gnss_fix, 4);
    assert_eq!(f.gnss_num_sv, 18);
    assert_eq!(f.gnss_utc_year, 2026);
    assert_eq!(f.gnss_utc_month, 3);
    assert_eq!(f.gnss_utc_day, 17);
    assert_eq!(f.pres, 46022);
    assert_eq!(f.ins_gnd_spd, 11489);
    assert_eq!(f.ins_alt_wgs84, 12668);
    assert_eq!(f.ins_load_factor, 1324);
    assert_eq!(f.ins_lat, 380638525);
    assert_eq!(f.ins_lon, -1224561554);
}

#[test]
fn test_multiple_frames() {
    let file = parse_test(&build_test_file(&[&TEST_FRAME, &TEST_FRAME, &TEST_FRAME]));
    assert_eq!(file.frames.len(), 3);
    for f in &file.frames {
        assert_eq!(f.sys_time_ms, 3007526);
    }
}

#[test]
fn test_status_bitfield() {
    let file = parse_test(&build_test_file(&[&TEST_FRAME]));
    let s = &file.frames[0].status;
    assert_ne!(s[1] & 0x40, 0, "INS initialized");
    assert_ne!(s[1] & 0x80, 0, "INS healthy");
    assert_ne!(s[1] & 0x20, 0, "GNSS healthy");
}

#[test]
fn test_bad_meta_magic() {
    let mut data = build_test_file(&[&TEST_FRAME]);
    data[0] = b'X';
    assert!(onflight::parse(&data, &ParseOptions::default()).is_err());
}

#[test]
fn test_empty_file() {
    assert!(onflight::parse(&[], &ParseOptions::default()).is_err());
}

#[test]
fn test_corrupt_frame_header() {
    let mut data = build_test_file(&[&TEST_FRAME]);
    data[TEST_META.len()] = b'X';
    assert!(onflight::parse(&data, &ParseOptions::default()).is_err());
}

#[test]
fn test_checksum_error_counted() {
    let mut data = build_test_file(&[&TEST_FRAME]);
    data[TEST_META.len() + 10] ^= 0xFF; // corrupt payload byte
    let file = onflight::parse(&data, &ParseOptions::default()).unwrap();
    assert_eq!(file.checksum_errors, 1);
    assert_eq!(file.frames.len(), 1); // still included
}

#[test]
fn test_skip_bad_checksums() {
    let mut data = build_test_file(&[&TEST_FRAME]);
    data[TEST_META.len() + 10] ^= 0xFF;
    let result = onflight::parse(&data, &ParseOptions {
        validate_checksums: true,
        skip_bad_checksums: true,
    });
    // Only frame is skipped → NoDataFrames error
    assert!(result.is_err());
}
