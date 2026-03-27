# onflight

Rust parser for [Bolder Flight Systems OnFlight Hub](https://bolderflight.com/onflight.html) `.onflight` binary data logs.

## Usage

```rust
let bytes = std::fs::read("data0.onflight")?;
let file = onflight::parse(&bytes, &onflight::ParseOptions::default())?;

println!("Tail: {}", file.header.tail_number);
println!("Frames: {}", file.frames.len());

// Row access with raw values — apply scale factors yourself
for frame in &file.frames {
    let speed_kts = frame.ins_gnd_spd as f32 / 100.0;
    let alt_ft = frame.ins_alt_wgs84 as i32 - 10000;
    println!("t={}ms  {:.1} kts  {} ft", frame.sys_time_ms, speed_kts, alt_ft);
}

// Or use columnar access with built-in unit conversion
let cols = onflight::Columns::from_file(file);
for i in 0..cols.len {
    let speed_kts = cols.ins_gnd_spd[i] as f32 / 100.0;
    let alt_ft = cols.ins_alt_wgs84[i] as i32 - 10000;
    println!("{:.1} kts  {} ft", speed_kts, alt_ft);
}
```

## Data

Each 50 Hz frame contains:

- **System**: boot time, input voltage, temperatures (CPU, IMU, magnetometer, pressure sensor)
- **IMU**: 3-axis accelerometer and gyroscope (raw, body frame)
- **Magnetometer**: 3-axis magnetic field
- **Static pressure**: barometric pressure
- **GNSS**: fix type, satellite count, UTC date/time, lat/lon, WGS84 altitude, geoid height, NED velocity, position and velocity accuracy estimates
- **INS** (EKF-fused): pitch, roll, heading (true and magnetic), magnetic variation, ground speed, ground track, climb rate, load factor, flight path angle, NED velocity, lat/lon, WGS84 altitude, filtered accelerometer/gyroscope/magnetometer
- **ADC**: pressure altitude, static pressure
- **External airdata** *(when connected)*: IAS, CAS, TAS, OAT, wind speed and direction, angle of attack, pressure and density altitude
- **AGL altimeter** *(when connected)*: height above ground in inches
- **Heart rate** *(when connected)*: BPM

`Frame` and `Columns` store all values as packed integers (i8, u8, i16, u16, i32, u32) with documented scale factors. No floating-point conversion is performed by the parser — consumers apply scale factors at their display/analysis boundary. `Columns` is a struct-of-arrays transpose of `Vec<Frame>` for cache-friendly column iteration.

## Features

- Data frame versions 0, 1, and 2
- Fletcher 16 checksum validation (configurable)
- Row-oriented (`Frame`) and columnar (`Columns`) access
- Handles truncated end-of-file gracefully
- Zero dependencies
