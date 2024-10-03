pub mod tgeogpoint;
pub mod tgeompoint;
pub mod tpoint;

#[cfg(test)]
#[serial_test::serial]
mod tests {
    use crate::meos_initialize;

    use super::*;

    #[test]
    fn instant_tgeompoint() {
        meos_initialize("UTC");
        let string = "POINT(0 0)@2018-01-01 08:00:00+00";
        let result: tgeompoint::TGeomPoint = string.parse().unwrap();
        assert_eq!(
            format!("{result:?}"),
            format!("Instant({})", string.to_owned())
        );
    }

    #[test]
    fn instant_tgeogpoint() {
        meos_initialize("UTC");
        let string = "POINT(0 0)@2018-01-01 08:00:00+00";
        let result: tgeogpoint::TGeogPoint = string.parse().unwrap();
        assert_eq!(
            format!("{result:?}"),
            format!("Instant({})", string.to_owned())
        );
    }

    #[test]
    fn sequence_tgeompoint() {
        meos_initialize("UTC");
        let string = "[POINT(0 0)@2018-01-01 08:00:00+00]";
        let result: tgeompoint::TGeomPoint = string.parse().unwrap();
        assert_eq!(
            format!("{result:?}"),
            format!("Sequence({})", string.to_owned())
        );
    }

    #[test]
    fn sequence_tgeogpoint() {
        meos_initialize("UTC");
        let string = "[POINT(0 0)@2018-01-01 08:00:00+00]";
        let result: tgeogpoint::TGeogPoint = string.parse().unwrap();
        assert_eq!(
            format!("{result:?}"),
            format!("Sequence({})", string.to_owned())
        );
    }

    #[test]
    fn sequence_set_tgeompoint() {
        meos_initialize("UTC");
        let string = "{[POINT(0 0)@2018-01-01 08:00:00+00]}";
        let result: tgeompoint::TGeomPoint = string.parse().unwrap();
        assert_eq!(
            format!("{result:?}"),
            format!("SequenceSet({})", string.to_owned())
        );
    }

    #[test]
    fn sequence_set_tgeogpoint() {
        meos_initialize("UTC");
        let string = "{[POINT(0 0)@2018-01-01 08:00:00+00]}";
        let result: tgeogpoint::TGeogPoint = string.parse().unwrap();
        assert_eq!(
            format!("{result:?}"),
            format!("SequenceSet({})", string.to_owned())
        );
    }
}
