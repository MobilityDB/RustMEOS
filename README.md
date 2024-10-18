# RustMEOS

RustMEOS is a Rust library providing bindings for the [MEOS](https://libmeos.org/) C library, designed for spatiotemporal data management and analysis. It enables handling of temporal and spatial data, making it ideal for applications that need to work with moving objects, trajectories, and time-varying geographical data.

It supports MEOS version >= 1.2

## Overview

The primary goal of this library is to facilitate the creation and manipulation of temporal types, such as time-stamped geographic points, sequences, and numeric values. These temporal data structures can be used for various use cases including:

- **Tracking Movement:** Efficiently manage and analyze the movement of objects (e.g., vehicles, ships, animals) over time.
- **Spatiotemporal Queries**:
    - **Distance Calculations:** Compute the shortest distance between trajectories, which can be useful for determining when two moving objects were closest to each other.
    - **Time-Weighted Averages:** Analyze time-dependent data, like averaging speeds or temperatures over a period.
    - **Intersection Queries:** Check if a trajectory passes through specific points or regions, enabling location-based analysis of movement.

This library provides access to advanced spatiotemporal data handling capabilities of MEOS while maintaining Rust’s memory safety, concurrency, and performance benefits.

## Installation

Add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
meos = "0.2"
```
Ensure that the `meos` C library is installed on your system. Follow the installation instructions on the [MEOS website](https://github.com/MobilityDB/MobilityDB/?tab=readme-ov-file#requirements).

## Key Features

The library offers a range of temporal data types, including:

- **Temporal Geometric Points (`TGeomPoint`):** These represent geometric points that change over time (e.g., location data of moving objects).
- **Temporal Float (`TFloat`):** These store numeric values associated with time, such as speed or temperature over time.
- **Temporal Boolean (`TBool`):** Represents true/false values that vary over time, useful for tracking binary states such as whether an object is within a specific area at given times.

The type hierarchy is the following, the main types (`TGeomPoint`, `TFloat`, etc.) are enums that encapsulate the different kinds of temporal subtypes, **Instant**, **Sequence**, and **SequenceSet**, to learn more about these, refer to the [`meos` documentation](https://libmeos.org/documentation/datamodel/). Users can almost seamlessly use either the enums or the concrete structs (e.g. `TGeomPointSequence`). Some users may benefit from using the concrete structs since more concrete types can be inferred in some functions.

## Usage example

You can check more examples in the `examples/` directory.

### Constructing trajectories from text:

```rust
use meos::{meos_initialize, TGeomPoint};

meos_initialize();

let trajectory: TGeomPoint = "[POINT(1 1)@2000-01-01 08:00, POINT(2 2)@2000-01-01 08:01]".parse().unwrap();
```

### Constructing trajectories from a list of pairs (point, timestamp):

```rust
    use chrono::{DateTime, TimeZone, Utc};
    use geos::Geometry;
    use meos::{meos_initialize, TGeomPointSequence};

    meos_initialize();

    let geometries_with_time: Vec<(Geometry, DateTime<Utc>)> = vec![
        (
            Geometry::new_from_wkt("POINT(1 1)").unwrap(),
            Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        ),
        (
            Geometry::new_from_wkt("POINT(3 2)").unwrap(),
            Utc.with_ymd_and_hms(2020, 1, 1, 0, 1, 0).unwrap(),
        ),
        (
            Geometry::new_from_wkt("POINT(3 3)").unwrap(),
            Utc.with_ymd_and_hms(2020, 1, 1, 0, 2, 0).unwrap(),
        ),
    ];

    let tpoint: TGeomPointSequence = geometries_with_time.into_iter().collect();

    println!("{tpoint:?}");
```

### Get the shortest distance ever between two temporal points

```rust
use meos::{meos_initialize, TGeomPoint, TPointTrait};

meos_initialize();

let tpoint1: TGeomPoint =
    "[Point(0 0 0)@2001-01-01, Point(1 1 1)@2001-01-03, Point(0 0 0)@2001-01-05)"
        .parse()
        .unwrap();
let tpoint2: TGeomPoint =
    "[Point(2 0 0)@2001-01-02, Point(1 1 1)@2001-01-04, Point(2 2 2)@2001-01-06)"
        .parse()
        .unwrap();

let distance = tpoint1.nearest_approach_distance(&tpoint2);
println!("{distance}"); // Prints 0.5
```

### Check if a trajectory ever goes through a point (using `geos`)

```rust
use geos::Geometry;
use meos::{meos_initialize, TGeomPoint, Temporal};

meos_initialize();

let trajectory: TGeomPoint = "[Point(0 0 0)@2001-01-01, Point(2 2 2)@2001-01-05)"
    .parse()
    .unwrap();
let geom = Geometry::new_from_wkt("Point (1 1 1)").expect("Invalid geometry");

println!(
    "Does go through `geom`: {}",
    trajectory.ever_equal_than_value(geom).unwrap()
); // `true`
```

## Multithreading
Right now it should only be used in single-threaded applications. In the foreseeable future this could change.

## Build

This crate links dynamically to your system-installed `meos`. See [sys/README.md](./sys/README.md) for
more information.

## Contributing

Only a subset of `meos` has been implemented, feel free to add wrappers for missing features.

All added features needs to be tested and this being a C wrapper, valgrind runs on all examples/tests to check that
no bugs / memory leaks are lurking.

## Acknowledgments
This wrapper has been deeply influenced by the Rust wrapper of [`geos`](https://github.com/georust/geos)
