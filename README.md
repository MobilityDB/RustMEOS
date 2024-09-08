# meos

Rust bindings for [meos](https://libmeos.org/) C API.

The supported meos version is >= 1.1

## Disclaimer

The crate is still in alpha, this means it is not advised for production usage as some tests are still to be added and not proper CI workflows are inplace. This project is checked with valgrind, but if you stumble on a crash feel free to open an issue explaining the problem.

## Usage example

You can check the examples in the `examples/` directory.

### Constructing trajectories from text:

```rust
use meos::{meos_initialize, TPoint};

meos_initialize("UTC");

let trajectory: TGeomPoint = "[POINT(1 1)@2000-01-01 08:00, POINT(2 2)@2000-01-01 08:01]".parse().unwrap();
```

### Get the shortest distance ever between two temporal points

```rust
use meos::{meos_initialize, TPoint};

meos_initialize("UTC");

let tpoint1: TGeomPoint = "[Point(0 0 0)@2001-01-01, Point(1 1 1)@2001-01-03, Point(0 0 0)@2001-01-05)".parse().unwrap();
let tpoint2: TGeomPoint = "[Point(2 0 0)@2001-01-02, Point(1 1 1)@2001-01-04, Point(2 2 2)@2001-01-06)".parse().unwrap();

let distance = tpoint1.nearest_approach_distance(tpoint2);
println!("{distance}"); // Prints 0.5
```

### Check if a trajectory ever goes through a point (using `geos`)

```rust
use meos::{meos_initialize, TPoint};
use geos::{Geometry}

meos_initialize("UTC");

let trajectory: TGeomPoint = "[Point(0 0 0)@2001-01-01, Point(2 2 2)@2001-01-05)".parse().unwrap();
let geom = Geometry::new_from_wkt("Point (1 1 1)").expect("Invalid geometry");

println!("Does go through `geom`: {}", tpoint1.ever_equal_than_value(geom).unwrap()); // `true`
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