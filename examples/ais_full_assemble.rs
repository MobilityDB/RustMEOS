use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    process,
    time::Instant,
};

use chrono::{DateTime, Utc};
use meos::{
    init,
    temporal::{
        number::{
            tfloat::{TFloatInstant, TFloatSequence},
            tnumber::TNumber,
        },
        point::{
            tgeompoint::{TGeomPoint, TGeomPointInstant, TGeomPointSequence},
            tpoint::TPointTrait,
        },
        temporal::Temporal,
        tinstant::TInstant,
        tsequence::TSequence,
    },
};

// Constants
const MAX_NO_RECORDS: usize = 10_000_000;
const MAX_SHIPS: usize = 6500;
const NO_RECORDS_BATCH: usize = 100_000;
const INITIAL_INSTANTS: usize = 64;
const MAX_LENGTH_HEADER: usize = 1024;

#[derive(Debug)]
struct AISRecord {
    t: DateTime<Utc>,
    mmsi: i64,
    latitude: Option<f64>,
    longitude: Option<f64>,
    sog: Option<f64>,
}

#[derive(Debug)]
struct TripRecord {
    mmsi: i64,
    num_instants: usize,
    trip_instants: Vec<TGeomPointInstant>,
    sog_instants: Vec<TFloatInstant>,
    trip: Option<TGeomPointSequence>,
    sog: Option<TFloatSequence>,
}

fn main() {
    init();
    let start_time = Instant::now();

    let mut trips: Vec<TripRecord> = Vec::with_capacity(MAX_SHIPS);
    let mut no_records = 0;
    let mut no_nulls = 0;
    let mut num_ships = 0;

    // Open the CSV file
    let file = File::open("data/aisdk-2023-08-01.csv").unwrap_or_else(|_| {
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
        NO_RECORDS_BATCH
    );

    for line in lines.flatten() {
        let rec = parse_ais_record(&line);
        if rec.is_none() {
            no_nulls += 1;
            continue;
        }
        let rec = rec.unwrap();

        no_records += 1;
        if no_records % NO_RECORDS_BATCH == 0 {
            print!("*");
            std::io::stdout().flush().unwrap();
        }

        if no_records == MAX_NO_RECORDS {
            break;
        }

        // Find or create the trip for the current MMSI
        let trip = trips.iter_mut().find(|trip| trip.mmsi == rec.mmsi);
        let trip = match trip {
            Some(trip) => trip,
            None => {
                if num_ships == MAX_SHIPS {
                    println!(
                        "The maximum number of ships in the input file is bigger than {}",
                        MAX_SHIPS
                    );
                    process::exit(1);
                }
                trips.push(TripRecord {
                    mmsi: rec.mmsi,
                    num_instants: 0,
                    trip_instants: Vec::with_capacity(INITIAL_INSTANTS),
                    sog_instants: Vec::with_capacity(INITIAL_INSTANTS),
                    trip: None,
                    sog: None,
                });
                num_ships += 1;
                trips.last_mut().unwrap()
            }
        };

        if let (Some(lat), Some(long)) = (rec.latitude, rec.longitude) {
            let point_str = format!(
                "SRID=4326;Point({} {})@{}+00",
                lat,
                long,
                rec.t.format("%Y-%m-%d %H:%M:%S")
            );
            let TGeomPoint::Instant(instant) = point_str.parse().unwrap() else {
                panic!()
            };
            trip.trip_instants.push(instant);
        }

        if let Some(sog) = rec.sog {
            let instant = TInstant::from_value_and_timestamp(sog, rec.t);
            trip.sog_instants.push(instant);
        }

        trip.num_instants += 1;
    }

    println!(
        "\n{} records read.\n{} incomplete records ignored.\n",
        no_records, no_nulls
    );
    println!("{} trips read.", num_ships);

    for trip in trips.iter_mut() {
        trip.trip = Some(TSequence::new(
            &trip.trip_instants.iter().collect::<Vec<_>>(),
            meos::temporal::interpolation::TInterpolation::Linear,
        ));
        trip.sog = Some(TSequence::new(
            &trip.sog_instants.iter().collect::<Vec<_>>(),
            meos::temporal::interpolation::TInterpolation::Linear,
        ));

        println!(
            "MMSI: {}, Number of input instants: {}",
            trip.mmsi, trip.num_instants
        );
        println!(
            "  Trip -> Number of instants: {}, Distance travelled: {:.2}",
            trip.trip.as_ref().unwrap().num_instants(),
            trip.trip.as_ref().unwrap().length()
        );
        println!(
            "  SOG -> Number of instants: {}, Time-weighted average: {:.2}",
            trip.sog.as_ref().unwrap().num_instants(),
            trip.sog.as_ref().unwrap().time_weighted_average()
        );
    }

    // Writing output to a file
    let output_file = File::create("data/ais_trips_new.csv").unwrap();
    let mut writer = csv::Writer::from_writer(output_file);

    writer.write_record(&["mmsi", "trip", "sog"]).unwrap();

    for trip in &trips {
        let trip_str = trip.trip.as_ref().unwrap().as_wkt(5); // Adjust as per actual method in `pymeos`
        let sog_str = format!("{:?}", trip.sog.as_ref().unwrap());
        writer
            .write_record(&[trip.mmsi.to_string(), trip_str, sog_str])
            .unwrap();
    }

    writer.flush().unwrap();

    let elapsed_time = start_time.elapsed();
    println!("The program took {:.2?} to execute", elapsed_time);
}

// Helper function to parse a line into an AISRecord
fn parse_ais_record(line: &str) -> Option<AISRecord> {
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() < 8 {
        return None;
    }

    let timestamp = DateTime::parse_from_rfc3339(fields[0])
        .ok()?
        .with_timezone(&Utc);
    let mmsi = fields[2].parse::<i64>().ok()?;
    let latitude = fields[3].parse::<f64>().ok();
    let longitude = fields[4].parse::<f64>().ok();
    let sog = fields[7].parse::<f64>().ok();

    Some(AISRecord {
        t: timestamp,
        mmsi,
        latitude,
        longitude,
        sog,
    })
}
