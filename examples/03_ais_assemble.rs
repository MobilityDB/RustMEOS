use std::{
    fs::File,
    io::{BufRead, BufReader},
    process,
    time::Instant,
};

use chrono::{DateTime, NaiveDateTime, Utc};
use meos::{
    meos_initialize, TFloatInstant, TFloatSequence, TGeomPoint, TGeomPointInstant,
    TGeomPointSequence, TInstant as _, TNumber as _, TPointTrait as _, Temporal as _,
};

const MAX_INSTANTS: usize = 50000;
const NO_INSTANTS_BATCH: usize = 1000;
const MAX_LENGTH_HEADER: usize = 1024;
const MAX_TRIPS: usize = 5;

#[derive(Debug)]
struct AISRecord {
    t: DateTime<Utc>,
    mmsi: i64,
    latitude: f64,
    longitude: f64,
    sog: f64,
}

#[derive(Debug)]
struct TripRecord {
    mmsi: i64,
    numinstants: usize,
    trip_instants: Vec<TGeomPointInstant>,
    sog_instants: Vec<TFloatInstant>,
    trip: Option<TGeomPointSequence>,
    sog: Option<TFloatSequence>,
}

fn main() {
    meos_initialize();
    let start_time = Instant::now();

    let mut trips: Vec<TripRecord> = Vec::new();
    let mut no_records = 0;
    let mut no_nulls = 0;
    let mut numships = 0;

    // https://github.com/MobilityDB/MobilityDB/blob/master/meos/examples/data/ais_instants.csv
    let file = File::open("data/ais_instants.csv").unwrap_or_else(|_| {
        println!("Error opening input file");
        process::exit(1);
    });

    let reader = BufReader::new(file);

    // Read the first line of the file with the headers
    let mut lines = reader.lines();
    if let Some(Ok(header)) = lines.next() {
        if header.len() > MAX_LENGTH_HEADER {
            println!("Header length exceeds maximum allowed length");
            process::exit(1);
        }
    }

    println!(
        "Reading the instants (one '*' marker every {} instants)",
        NO_INSTANTS_BATCH
    );

    for line in lines.flatten() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 5 {
            let timestamp_str = parts[0];
            let mmsi = parts[1].parse::<i64>().unwrap_or_default();
            let latitude = parts[2].parse::<f64>().unwrap_or_default();
            let longitude = parts[3].parse::<f64>().unwrap_or_default();
            let sog = parts[4].parse::<f64>().unwrap_or_default();

            let t: DateTime<Utc> =
                NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
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
            if no_records % NO_INSTANTS_BATCH == 0 {
                print!("*");
            }

            // Find or create the trip for the current MMSI
            let trip = trips.iter_mut().find(|trip| trip.mmsi == rec.mmsi);
            let trip = match trip {
                Some(trip) => trip,
                None => {
                    if numships == MAX_TRIPS {
                        println!(
                            "The maximum number of ships in the input file is bigger than {}",
                            MAX_TRIPS
                        );
                        process::exit(1);
                    }
                    trips.push(TripRecord {
                        mmsi: rec.mmsi,
                        numinstants: 0,
                        trip_instants: Vec::with_capacity(MAX_INSTANTS),
                        sog_instants: Vec::with_capacity(MAX_INSTANTS),
                        trip: None,
                        sog: None,
                    });
                    numships += 1;
                    trips.last_mut().unwrap()
                }
            };

            let point_str = format!(
                "SRID=4326;Point({} {})@{}+00",
                rec.longitude,
                rec.latitude,
                rec.t.format("%Y-%m-%d %H:%M:%S")
            );
            let TGeomPoint::Instant(inst1) = point_str.parse().unwrap() else {
                panic!()
            };
            trip.trip_instants.push(inst1);

            let inst2: TFloatInstant = TFloatInstant::from_value_and_timestamp(rec.sog, rec.t);
            trip.sog_instants.push(inst2);
            trip.numinstants += 1;
        } else {
            println!("Record with missing values ignored");
            no_nulls += 1;
        }
    }

    println!(
        "\n{} records read.\n{} incomplete records ignored.\n",
        no_records, no_nulls
    );
    println!("{} trips read.", numships);

    for trip in trips.iter_mut() {
        trip.trip = Some(trip.trip_instants.iter().collect());
        trip.sog = Some(trip.sog_instants.iter().collect());

        println!(
            "MMSI: {}, Number of input instants: {}",
            trip.mmsi, trip.numinstants
        );
        println!(
            "  Trip -> Number of instants: {}, Distance travelled {}",
            trip.trip.as_ref().unwrap().num_instants(),
            trip.trip.as_ref().unwrap().length()
        );
        println!(
            "  SOG -> Number of instants: {}, Time-weighted average {}",
            trip.sog.as_ref().unwrap().num_instants(),
            trip.sog.as_ref().unwrap().time_weighted_average()
        );
    }

    let output_file = File::create("data/ais_trips_new.csv").unwrap();
    let mut writer = csv::Writer::from_writer(output_file);

    writer.write_record(&["mmsi", "trip", "sog"]).unwrap();

    for trip in &trips {
        let trip_str = trip.trip.as_ref().unwrap().as_wkt(5);
        let sog_str = format!("{:?}", trip.sog.as_ref().unwrap());
        writer
            .write_record(&[trip.mmsi.to_string(), trip_str, sog_str])
            .unwrap();
    }

    writer.flush().unwrap();

    let elapsed_time = start_time.elapsed();
    println!("The program took {:.2?} to execute", elapsed_time);
}
