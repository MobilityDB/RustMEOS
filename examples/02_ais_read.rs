use std::{
    fs::File,
    io::{BufRead, BufReader},
    process,
};

use chrono::{DateTime, NaiveDateTime, Utc};
use meos::{
    meos_initialize,
    temporal::{number::tfloat::TFloatInstant, point::tgeompoint::TGeomPoint},
};

const MAX_LENGTH_HEADER: usize = 1024;

#[derive(Debug)]
struct AISRecord {
    t: DateTime<Utc>,
    mmsi: i64,
    latitude: f64,
    longitude: f64,
    sog: f64,
}

fn main() {
    meos_initialize();
    // https://github.com/MobilityDB/MobilityDB/blob/master/meos/examples/data/ais_instants.csv
    let file = File::open("data/ais_instants.csv").unwrap_or_else(|_| {
        println!("Error opening input file");
        process::exit(1);
    });

    let reader = BufReader::new(file);

    let mut no_records = 0;
    let mut no_nulls = 0;
    let mut lines = reader.lines();

    // Read the first line of the file with the headers
    if let Some(Ok(header)) = lines.next() {
        if header.len() > MAX_LENGTH_HEADER {
            println!("Header length exceeds maximum allowed length");
            process::exit(1);
        }
    }

    // Continue reading the file
    for line in lines.flatten() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 5 {
            let timestamp_buffer = parts[0];
            let mmsi = parts[1].parse::<i64>().unwrap_or_default();
            let latitude = parts[2].parse::<f64>().unwrap_or_default();
            let longitude = parts[3].parse::<f64>().unwrap_or_default();
            let sog = parts[4].parse::<f64>().unwrap_or_default();

            // Transform the string representing the timestamp into a timestamp value
            let t: DateTime<Utc> =
                NaiveDateTime::parse_from_str(timestamp_buffer, "%Y-%m-%d %H:%M:%S")
                    .unwrap()
                    .and_utc();

            let rec = AISRecord {
                t,
                mmsi,
                latitude,
                longitude,
                sog,
            };

            no_records += 1;

            // Print only 1 out of 1000 records
            if no_records % 1000 == 0 {
                let t_out = rec.t.to_string();
                let point_buffer = format!(
                    "SRID=4326;Point({} {})@{}+00",
                    rec.longitude, rec.latitude, t_out
                );
                let TGeomPoint::Instant(inst1) = point_buffer.parse().unwrap() else {
                    panic!()
                };

                let inst2: TFloatInstant = (rec.sog, rec.t).into();
                println!(
                    "MMSI: {}, Location: {:?} SOG : {:?}",
                    rec.mmsi, inst1, inst2
                );

                // Memory management handled automatically in Rust
            }
        } else {
            println!("Record with missing values ignored");
            no_nulls += 1;
        }
    }

    println!(
        "\n{} no_records read.\n{} incomplete records ignored.",
        no_records, no_nulls
    );
}
