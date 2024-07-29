use std::collections::HashSet;

use crate::BoundingBox;

use super::interpolation::TInterpolation;

pub trait Temporal: Collection {
    /// Creates a temporal object from a base value and a time object.
    ///
    /// # Arguments
    /// * `value` - Base value.
    /// * `base` - Time object to use as the temporal dimension.
    ///
    /// # Returns
    /// A new temporal object.
    fn from_base_time(value: TBase, base: Time) -> Self;

    /// Creates a temporal object from an MF-JSON string.
    ///
    /// # Arguments
    /// * `mfjson` - The MF-JSON string.
    ///
    /// # Returns
    /// A temporal object.
    fn from_mfjson(mfjson: &str) -> Self;

    /// Creates a temporal object from Well-Known Binary (WKB) bytes.
    ///
    /// # Arguments
    /// * `wkb` - The WKB bytes.
    ///
    /// # Returns
    /// A temporal object.
    fn from_wkb(wkb: &[u8]) -> Self;

    /// Creates a temporal object from a hex-encoded WKB string.
    ///
    /// # Arguments
    /// * `hexwkb` - The hex-encoded WKB string.
    ///
    /// # Returns
    /// A temporal object.
    fn from_hexwkb(hexwkb: &str) -> Self;

    /// Creates a temporal object by merging multiple temporal objects.
    ///
    /// # Arguments
    /// * `temporals` - The temporal objects to merge.
    ///
    /// # Returns
    /// A merged temporal object.
    fn from_merge(temporals: &[Self]) -> Self;

    /// Returns the temporal object as a Well-Known Text (WKT) string.
    ///
    /// # Returns
    /// The temporal object as a WKT string.
    fn as_wkt(&self) -> String;

    /// Returns the temporal object as an MF-JSON string.
    ///
    /// # Arguments
    /// * `with_bbox` - Whether to include the bounding box in the output.
    /// * `flags` - The flags to use for the output.
    /// * `precision` - The precision to use for the output.
    /// * `srs` - The spatial reference system (SRS) to use for the output.
    ///
    /// # Returns
    /// The temporal object as an MF-JSON string.
    fn as_mfjson(&self, with_bbox: bool, flags: i32, precision: i32, srs: Option<&str>) -> String;

    /// Returns the temporal object as Well-Known Binary (WKB) bytes.
    ///
    /// # Returns
    /// The temporal object as WKB bytes.
    fn as_wkb(&self) -> Vec<u8>;

    /// Returns the temporal object as a hex-encoded WKB string.
    ///
    /// # Returns
    /// The temporal object as a hex-encoded WKB string.
    fn as_hexwkb(&self) -> String;

    /// Returns the bounding box of the temporal object.
    ///
    /// # Returns
    /// The bounding box of the temporal object.
    fn bounding_box(&self) -> impl BoundingBox;

    /// Returns the interpolation method of the temporal object.
    ///
    /// # Returns
    /// The interpolation method.
    fn interpolation(&self) -> TInterpolation;

    /// Returns the set of unique values in the temporal object.
    ///
    /// # Returns
    /// A set of unique values.
    fn value_set(&self) -> HashSet<TBase>;

    /// Returns the list of values taken by the temporal object.
    ///
    /// # Returns
    /// A list of values.
    fn values(&self) -> Vec<TBase>;

    /// Returns the starting value of the temporal object.
    ///
    /// # Returns
    /// The starting value.
    fn start_value(&self) -> TBase;

    /// Returns the ending value of the temporal object.
    ///
    /// # Returns
    /// The ending value.
    fn end_value(&self) -> TBase;

    /// Returns the minimum value of the temporal object.
    ///
    /// # Returns
    /// The minimum value.
    fn min_value(&self) -> TBase;

    /// Returns the maximum value of the temporal object.
    ///
    /// # Returns
    /// The maximum value.
    fn max_value(&self) -> TBase;

    /// Returns the value of the temporal object at a specific timestamp.
    ///
    /// # Arguments
    /// * `timestamp` - The timestamp.
    ///
    /// # Returns
    /// The value at the given timestamp.
    fn value_at_timestamp(&self, timestamp: DateTime<Utc>) -> TBase;

    /// Returns the time span on which the temporal object is defined.
    ///
    /// # Returns
    /// The time span.
    fn time(&self) -> TsTzSpanSet;

    /// Returns the duration of the temporal object.
    ///
    /// # Arguments
    /// * `ignore_gaps` - Whether to ignore gaps in the temporal value.
    ///
    /// # Returns
    /// The duration of the temporal object.
    fn duration(&self, ignore_gaps: bool) -> Duration;

    /// Returns the number of instants in the temporal object.
    ///
    /// # Returns
    /// The number of instants.
    fn num_instants(&self) -> usize;

    /// Returns the first instant in the temporal object.
    ///
    /// # Returns
    /// The first instant.
    fn start_instant(&self) -> TI;

    /// Returns the last instant in the temporal object.
    ///
    /// # Returns
    /// The last instant.
    fn end_instant(&self) -> TI;

    /// Returns the instant with the minimum value in the temporal object.
    ///
    /// # Returns
    /// The instant with the minimum value.
    fn min_instant(&self) -> TI;

    /// Returns the instant with the maximum value in the temporal object.
    ///
    /// # Returns
    /// The instant with the maximum value.
    fn max_instant(&self) -> TI;

    /// Returns the n-th instant in the temporal object.
    ///
    /// # Arguments
    /// * `n` - The index (0-based).
    ///
    /// # Returns
    /// The n-th instant.
    fn instant_n(&self, n: usize) -> TI;

    /// Returns the list of instants in the temporal object.
    ///
    /// # Returns
    /// A list of instants.
    fn instants(&self) -> Vec<TI>;

    /// Returns the number of timestamps in the temporal object.
    ///
    /// # Returns
    /// The number of timestamps.
    fn num_timestamps(&self) -> usize;

    /// Returns the first timestamp in the temporal object.
    ///
    /// # Returns
    /// The first timestamp.
    fn start_timestamp(&self) -> DateTime<Utc>;

    /// Returns the last timestamp in the temporal object.
    ///
    /// # Returns
    /// The last timestamp.
    fn end_timestamp(&self) -> DateTime<Utc>;

    /// Returns the n-th timestamp in the temporal object.
    ///
    /// # Arguments
    /// * `n` - The index (0-based).
    ///
    /// # Returns
    /// The n-th timestamp.
    fn timestamp_n(&self, n: usize) -> DateTime<Utc>;

    /// Returns the list of timestamps in the temporal object.
    ///
    /// # Returns
    /// A list of timestamps.
    fn timestamps(&self) -> Vec<DateTime<Utc>>;

    /// Returns the list of segments in the temporal object.
    ///
    /// # Returns
    /// A list of segments.
    fn segments(&self) -> Vec<TS>;
}
